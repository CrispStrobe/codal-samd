// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//! Actual finite allocation churn; no fabricated return values or heap failures.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::{bus::SystemBus, system::cortex_m::configure_cortex_m, Bus, Machine};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const ENGINE: &str = "79071f69cd6035a820cbdad93da7343d7d501e4d";
const NAMES: [&str; 11] = [
    "runtime_status",
    "churn_checks",
    "churn_round",
    "churn_sequence",
    "churn_operation",
    "churn_slot",
    "churn_request",
    "churn_old",
    "churn_prior_size",
    "churn_return",
    "churn_boundaries",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Event {
    round: u32,
    operation: u32,
    slot: usize,
    size: u32,
}

fn events() -> Vec<Event> {
    let mut v = Vec::new();
    for operation in 6..=9 {
        v.push(Event {
            round: 0,
            operation,
            slot: 8,
            size: if operation % 2 == 0 { 0 } else { u32::MAX },
        });
    }
    for round in 0..8 {
        let sizes = [1, 7, 17, 33];
        for operation in [1, 4] {
            for slot in if operation == 4 { 4..8 } else { 0..8 } {
                v.push(Event {
                    round,
                    operation,
                    slot,
                    size: if operation == 4 {
                        u32::MAX
                    } else {
                        sizes[slot % 4] + round
                    },
                });
            }
        }
        for shrink in [false, true] {
            for slot in 0..8 {
                let grown = sizes[slot % 4] + round + 9;
                v.push(Event {
                    round,
                    operation: 2,
                    slot,
                    size: if shrink { (grown + 1) / 2 } else { grown },
                });
            }
        }
        v.push(Event {
            round,
            operation: 5,
            slot: 8,
            size: 0,
        });
        for i in 0..4 {
            for slot in [i, i + 4] {
                v.push(Event {
                    round,
                    operation: 3,
                    slot,
                    size: 0,
                });
            }
        }
        v.push(Event {
            round,
            operation: 5,
            slot: 8,
            size: 0,
        });
    }
    assert_eq!(v.len(), 308);
    v
}

fn terminal(v: [u32; 11], completed: usize, live: &[[u32; 3]; 8], constructors: usize) {
    assert_eq!((v[0], v[1], v[2], v[3], v[10]), (0x600d, 662, 7, 308, 308));
    assert_eq!(completed, 308);
    assert_eq!(*live, [[0; 3]; 8]);
    assert!(constructors > 0);
}

#[test]
fn manufactured_churn_controls_require_complete_inventory() {
    let e = events();
    for (op, count) in [(1, 64), (2, 128), (3, 64), (4, 32), (5, 16)] {
        assert_eq!(e.iter().filter(|e| e.operation == op).count(), count);
    }
    let good = [0x600d, 662, 7, 308, 5, 8, 0, 0, 0, 0, 308];
    terminal(good, 308, &[[0; 3]; 8], 1);
    for i in [0, 1, 2, 3, 10] {
        let mut bad = good;
        bad[i] = 0;
        assert!(std::panic::catch_unwind(|| terminal(bad, 308, &[[0; 3]; 8], 1)).is_err());
    }
    assert!(std::panic::catch_unwind(|| terminal(good, 307, &[[0; 3]; 8], 1)).is_err());
    let mut live = [[0; 3]; 8];
    live[0] = [0x20001000, 1, 32];
    assert!(std::panic::catch_unwind(|| terminal(good, 308, &live, 1)).is_err());
    assert!(std::panic::catch_unwind(|| terminal(good, 308, &[[0; 3]; 8], 0)).is_err());
}

struct Call {
    return_pc: u32,
    event: Event,
    old: u32,
    bytes: Vec<u8>,
}

#[test]
#[ignore = "hosted source-bound finite churn execution only"]
fn actual_churn_preserves_payloads_and_native_refusals() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|c| c.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_CHURN_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["status"], "GC_ALLOCATION_CHURN_LINK_ONLY");
    assert_eq!(report["case"], "allocation-churn");
    assert_eq!(report["probe_source"], "pxt-gc-runtime/probe-churn.cpp");
    assert_eq!(report["startup_source"], "pxt-gc-runtime/startup.S");
    assert_eq!(report["core"], "2ed19d64ec3359143409a4b0a9758b4aeef3ee7d");
    for key in [
        "guest_qualified",
        "native_arcade",
        "performance_measured",
        "board_qualified",
        "licence_clearance",
    ] {
        assert_eq!(report[key], false);
    }
    let path = folder.join("gc-control.elf");
    let bytes = std::fs::read(&path).unwrap();
    let hash = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(report["elf_sha256"].as_str(), Some(hash.as_str()));
    let elf = goblin::elf::Elf::parse(&bytes).unwrap();
    let symbol = |name| {
        let found: Vec<_> = elf
            .syms
            .iter()
            .filter(|s| s.st_shndx != 0 && elf.strtab.get_at(s.st_name) == Some(name))
            .collect();
        assert_eq!(found.len(), 1, "unique defined symbol {name}");
        found[0].st_value
    };
    let addresses = NAMES.map(symbol);
    let live_base = symbol("churn_live");
    let boundary = (symbol("authored_churn_boundary") & !1) as u32;
    let init_entry = symbol("gc_static_init_entry") & !1;
    let probe_entry = symbol("runtime_probe") & !1;
    let in_gc = symbol("_ZN3pxt4inGCE");
    let apis = [
        "malloc",
        "realloc",
        "free",
        "bw_native_alloc",
        "bw_native_realloc",
        "bw_native_free",
        "_ZN3pxt2gcEi",
    ]
    .map(|n| (symbol(n) & !1) as u32);
    let mut unique = apis.to_vec();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), 7);
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let chip = ChipDescriptor::from_file(root.join("configs/chips/atsamd51.yaml")).unwrap();
    assert_eq!(chip.cpu_hz, 120000000);
    let manifest: SystemManifest = serde_yaml::from_str(
        "name: pxt-allocation-churn\nchip: ../chips/atsamd51.yaml\nexternal_devices: []\n",
    )
    .unwrap();
    let mut bus = SystemBus::from_config(&chip, &manifest).unwrap();
    let (cpu, _) = configure_cortex_m(&mut bus);
    let mut machine = Machine::new(cpu, bus);
    machine
        .load_firmware(&labwired_loader::load_elf(&path).unwrap())
        .unwrap();
    assert_eq!(symbol("runtime_vectors"), 0);
    assert_eq!(symbol("runtime_vectors_end"), 1024);
    assert_eq!(machine.bus.read_u32(0).unwrap(), 0x20030000);
    for i in 1..256 {
        let expected = match i {
            1 => "reset_entry",
            15 => "runtime_systick",
            _ => "runtime_fault",
        };
        assert_eq!(
            u64::from(machine.bus.read_u32(i * 4).unwrap()),
            symbol(expected) | 1
        );
    }
    let data = symbol("_data_start")..symbol("_data_end");
    let bss = symbol("_bss_start")..symbol("_bss_end");
    let expected_data: Vec<_> = (0..data.end - data.start)
        .step_by(4)
        .map(|o| machine.bus.read_u32(symbol("_data_load") + o).unwrap())
        .collect();
    for address in data.clone().step_by(4) {
        machine.bus.write_u32(address, 0x5a5a5a5a).unwrap();
    }
    for address in bss.clone().step_by(4) {
        machine.bus.write_u32(address, 0xa5a5a5a5).unwrap();
    }
    let constructors: Vec<_> = (symbol("_init_array_start")..symbol("_init_array_end"))
        .step_by(4)
        .map(|a| machine.bus.read_u32(a).unwrap() & !1)
        .collect();
    assert!(!constructors.is_empty());
    let mut entered = std::collections::BTreeSet::new();
    let (mut initialized, mut startup) = (false, false);
    let expected = events();
    let mut completed = 0;
    let mut live = [[0u32; 3]; 8];
    let mut active: Option<Call> = None;
    let mut finished: Option<u32> = None;
    let bound = 12_000_000;
    for step in 1..=bound {
        let pc = machine.cpu.pc & !1;
        if u64::from(pc) == init_entry && !initialized {
            for a in bss.clone().step_by(4) {
                assert_eq!(machine.bus.read_u32(a).unwrap(), 0);
            }
            for (a, v) in data.clone().step_by(4).zip(&expected_data) {
                assert_eq!(machine.bus.read_u32(a).unwrap(), *v);
            }
            initialized = true;
        }
        if constructors.contains(&pc) {
            entered.insert(pc);
        }
        if u64::from(pc) == probe_entry && !startup {
            assert!(initialized && constructors.iter().all(|a| entered.contains(a)));
            startup = true;
        }
        if startup {
            assert_eq!(machine.cpu.active_exception, 0);
            let v = addresses.map(|a| machine.bus.read_u32(a).unwrap());
            if active.is_none() && finished.is_none() && apis.contains(&pc) {
                assert!(machine.cpu.primask);
                let e = expected[completed];
                assert_eq!(
                    (v[2], v[3], v[4], v[5], v[6]),
                    (
                        e.round,
                        completed as u32 + 1,
                        e.operation,
                        e.slot as u32,
                        e.size
                    )
                );
                let api = match e.operation {
                    1 => {
                        if e.slot < 4 {
                            0
                        } else {
                            3
                        }
                    }
                    2 => {
                        if e.slot < 4 {
                            1
                        } else {
                            4
                        }
                    }
                    3 => {
                        if e.slot < 4 {
                            2
                        } else {
                            5
                        }
                    }
                    4 | 8 | 9 => 4,
                    5 => 6,
                    6 | 7 => 3,
                    _ => unreachable!(),
                };
                assert_eq!(pc, apis[api], "actual outer allocator/collector ownership");
                assert_eq!(
                    (machine.bus.read_u32(in_gc & !3).unwrap() >> ((in_gc & 3) * 8)) & 0xff,
                    0
                );
                let old = if e.slot < 8 { live[e.slot] } else { [0; 3] };
                assert_eq!((v[7], v[8]), (old[0], old[1]));
                if [2, 4, 8, 9].contains(&e.operation) {
                    assert_eq!((machine.cpu.r0, machine.cpu.r1), (old[0], e.size));
                } else {
                    assert_eq!(
                        machine.cpu.r0,
                        if e.operation == 3 { old[0] } else { e.size }
                    );
                }
                let mut content = Vec::new();
                for i in 0..old[1] {
                    let a = u64::from(old[0] + i);
                    content.push(
                        ((machine.bus.read_u32(a & !3).unwrap() >> ((a & 3) * 8)) & 255) as u8,
                    );
                }
                assert!(content
                    .iter()
                    .enumerate()
                    .all(|(i, b)| *b == old[2].wrapping_add(i as u32 * 13) as u8));
                println!("PXT_GC_CHURN_ENTRY source={source} elf_sha256={hash} step={step} sequence={} op={} slot={} bytes={} old={} prior={} actual={pc:#x} guest_inputs_unchanged=true",completed+1,e.operation,e.slot,e.size,old[0],old[1]);
                active = Some(Call {
                    return_pc: machine.cpu.lr & !1,
                    event: e,
                    old: old[0],
                    bytes: content,
                });
            }
            if let Some(call) = active.as_ref() {
                if pc == call.return_pc {
                    assert!(machine.cpu.primask);
                    let e = call.event;
                    let pointer = if [3, 5].contains(&e.operation) {
                        0
                    } else {
                        machine.cpu.r0
                    };
                    assert_eq!(
                        (machine.bus.read_u32(in_gc & !3).unwrap() >> ((in_gc & 3) * 8)) & 0xff,
                        0
                    );
                    println!("PXT_GC_CHURN_RETURN source={source} elf_sha256={hash} step={step} sequence={} op={} slot={} bytes={} old={} pointer={pointer} guest_inputs_unchanged=true",completed+1,e.operation,e.slot,e.size,call.old);
                    if [4, 7, 9].contains(&e.operation) {
                        assert_eq!(pointer, 0, "native arithmetic refusal must return null");
                    } else if [6, 8].contains(&e.operation) {
                        assert_eq!(pointer, 0, "native zero request must return null");
                    } else if [1, 2].contains(&e.operation) {
                        assert!(
                            (0x20000000..0x2002e000).contains(&pointer)
                                && pointer % if e.slot < 4 { 4 } else { 8 } == 0
                        );
                        if e.operation == 2 {
                            let length = (call.bytes.len() as u32).min(e.size);
                            let mut actual = Vec::new();
                            for i in 0..length {
                                let a = u64::from(pointer + i);
                                actual.push(
                                    ((machine.bus.read_u32(a & !3).unwrap() >> ((a & 3) * 8)) & 255)
                                        as u8,
                                );
                            }
                            let wanted = &call.bytes[..length as usize];
                            println!("PXT_GC_CHURN_COPY source={source} elf_sha256={hash} step={step} sequence={} bytes={length} old={} pointer={pointer} expected_sha256={:x} actual_sha256={:x} guest_inputs_unchanged=true",completed+1,call.old,Sha256::digest(wanted),Sha256::digest(&actual));
                            if e.slot < 4 {
                                assert_eq!(
                                    actual, wanted,
                                    "public realloc must preserve every old payload byte"
                                );
                            } else {
                                assert_eq!(
                                    actual, wanted,
                                    "native realloc must preserve every old payload byte"
                                );
                            }
                        }
                    }
                    finished = Some(pointer);
                    active = None;
                }
            }
            if pc == boundary {
                assert!(active.is_none() && machine.cpu.primask);
                let pointer = finished
                    .take()
                    .expect("actual API return must precede each boundary");
                let e = expected[completed];
                assert_eq!(
                    (v[2], v[3], v[4], v[5], v[6]),
                    (
                        e.round,
                        completed as u32 + 1,
                        e.operation,
                        e.slot as u32,
                        e.size
                    )
                );
                let old = if e.slot < 8 { live[e.slot] } else { [0; 3] };
                assert_eq!((v[7], v[8]), (old[0], old[1]));
                assert_eq!(v[9], pointer);
                assert_eq!(v[10], completed as u32);
                match e.operation {
                    1 => live[e.slot] = [pointer, e.size, 0x20 + e.slot as u32 * 19 + e.round * 7],
                    2 => {
                        live[e.slot][0] = pointer;
                        live[e.slot][1] = e.size;
                    }
                    3 => live[e.slot] = [0; 3],
                    _ => {}
                }
                let mut count = 0;
                for (i, s) in live.iter().enumerate() {
                    for (j, w) in s.iter().enumerate() {
                        assert_eq!(
                            machine
                                .bus
                                .read_u32(live_base + (i * 12 + j * 4) as u64)
                                .unwrap(),
                            *w
                        );
                    }
                    if s[0] == 0 {
                        assert_eq!(s[1], 0);
                        continue;
                    }
                    count += 1;
                    for k in 0..s[1] {
                        let a = u64::from(s[0] + k);
                        let b =
                            ((machine.bus.read_u32(a & !3).unwrap() >> ((a & 3) * 8)) & 255) as u8;
                        assert_eq!(
                            b,
                            s[2].wrapping_add(k * 13) as u8,
                            "every live payload byte after disturbance"
                        );
                    }
                    for other in &live[..i] {
                        if other[0] != 0 {
                            assert!(s[0] + s[1] <= other[0] || other[0] + other[1] <= s[0]);
                        }
                    }
                }
                completed += 1;
                println!("PXT_GC_CHURN_BOUNDARY source={source} elf_sha256={hash} step={step} sequence={completed} op={} slot={} bytes={} pointer={pointer} checks={} live={count} contents_verified=true guest_inputs_unchanged=true",e.operation,e.slot,e.size,v[1]);
            }
        }
        machine
            .step()
            .expect("actual allocation churn architectural fault");
        if startup {
            let v = addresses.map(|a| machine.bus.read_u32(a).unwrap());
            assert!(
                [0, 0x100, 0x600d].contains(&v[0]),
                "churn guest panic status={:#x} sequence={} checks={}",
                v[0],
                v[3],
                v[1]
            );
            if v[0] == 0x600d {
                assert!(active.is_none() && finished.is_none());
                assert!(machine.cpu.primask && machine.cpu.active_exception == 0);
                assert_eq!(
                    (machine.bus.read_u32(in_gc & !3).unwrap() >> ((in_gc & 3) * 8)) & 0xff,
                    0
                );
                for a in [0xe000ed28, 0xe000ed2c] {
                    assert_eq!(machine.bus.read_u32(a).unwrap(), 0);
                }
                terminal(v, completed, &live, entered.len());
                println!("PXT_GC_CHURN_CALLER engine={ENGINE} source={source} elf_sha256={hash} steps={step} bound={bound} checks=662 boundaries=308 rounds=8 copies=128 zero_refusals=2 arithmetic_refusals=34 constructors={} native_arcade=false performance_measured=false real_oom_qualified=false allocated_dma_qualified=false",entered.len());
                return;
            }
        }
    }
    println!("PXT_GC_CHURN_BOUND source={source} elf_sha256={hash} bound={bound} completed={completed} qualified=false");
    panic!("actual allocation churn did not reach terminal within {bound} steps");
}
