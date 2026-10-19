// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//! Actual allocation-routing caller; never fabricates IRQ or collector state.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::{bus::SystemBus, system::cortex_m::configure_cortex_m, Bus, Machine};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const ENGINE: &str = "79071f69cd6035a820cbdad93da7343d7d501e4d";
const NAMES: [&str; 12] = [
    "runtime_status",
    "routing_stage",
    "routing_checks",
    "routing_irq_count",
    "routing_irq_done",
    "routing_scan_count",
    "routing_thread_pointer",
    "routing_irq_pointer",
    "routing_scan_pointer",
    "routing_irq_exception",
    "routing_irq_mask",
    "routing_scan_in_gc",
];

fn terminal(v: [u32; 12], stages: [bool; 4], routes: &[(u32, u32)], irq: u32, scan: u32) {
    assert_eq!(&v[..6], &[0x600d, 4, 23, 1, 1, 1]);
    assert_eq!(v[9], 16);
    assert_eq!(v[10], 0);
    assert_ne!(v[11], 0);
    assert!(v[6..9]
        .iter()
        .all(|p| (0x20000000..0x2002e000).contains(p) && p % 4 == 0));
    assert!(stages.iter().all(|seen| *seen));
    assert_eq!(routes, &[(1, v[6]), (2, v[7]), (3, v[8])]);
    assert_eq!((irq, scan), (1, 1));
}

#[test]
fn manufactured_routing_terminal_controls_refuse_missing_observations() {
    let good = [
        0x600d, 4, 23, 1, 1, 1, 0x20001004, 0x20002000, 0x20002000, 16, 0, 1,
    ];
    let routes = [(1, good[6]), (2, good[7]), (3, good[8])];
    terminal(good, [true; 4], &routes, 1, 1);
    for i in 0..good.len() {
        let mut bad = good;
        bad[i] = if i == 10 { 1 } else { 0 };
        assert!(std::panic::catch_unwind(|| terminal(bad, [true; 4], &routes, 1, 1)).is_err());
    }
    for i in 0..4 {
        let mut stages = [true; 4];
        stages[i] = false;
        assert!(std::panic::catch_unwind(|| terminal(good, stages, &routes, 1, 1)).is_err());
    }
    for i in 0..3 {
        let mut bad = routes;
        bad[i].1 = 0;
        assert!(std::panic::catch_unwind(|| terminal(good, [true; 4], &bad, 1, 1)).is_err());
    }
    assert!(std::panic::catch_unwind(|| terminal(good, [true; 4], &routes, 0, 1)).is_err());
    assert!(std::panic::catch_unwind(|| terminal(good, [true; 4], &routes, 1, 0)).is_err());
}

struct Allocation {
    stage: u32,
    size: u32,
    exception: u32,
    in_gc: u32,
    return_pc: u32,
    selected: bool,
}

#[test]
#[ignore = "hosted source-bound allocation-routing execution only"]
fn actual_irq_and_gc_scan_select_raw_allocator() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_ROUTING_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["status"], "GC_ALLOCATION_ROUTING_LINK_ONLY");
    assert_eq!(report["case"], "allocation-routing");
    assert_eq!(report["probe_source"], "pxt-gc-runtime/probe-routing.cpp");
    assert_eq!(report["startup_source"], "pxt-gc-runtime/routing-startup.S");
    assert_eq!(report["core"], "2ed19d64ec3359143409a4b0a9758b4aeef3ee7d");
    for flag in [
        "guest_qualified",
        "native_arcade",
        "performance_measured",
        "board_qualified",
        "licence_clearance",
    ] {
        assert_eq!(report[flag], false);
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
    let mut unique = addresses.to_vec();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), NAMES.len());
    let init_entry = symbol("gc_static_init_entry") & !1;
    let probe_entry = symbol("runtime_probe") & !1;
    let malloc_entry = (symbol("malloc") & !1) as u32;
    let app_entry = (symbol("app_alloc") & !1) as u32;
    let raw_entry = (symbol("device_malloc") & !1) as u32;
    assert_ne!(app_entry, raw_entry);
    assert!(malloc_entry != app_entry && malloc_entry != raw_entry);
    let in_gc_address = symbol("_ZN3pxt4inGCE");
    let irq_entry = (symbol("authored_gc_routing_irq0") & !1) as u32;
    let scan_entry = (symbol("authored_gc_allocating_scan") & !1) as u32;
    let constructors = symbol("_init_array_start")..symbol("_init_array_end");
    assert!(
        constructors.start < constructors.end && (constructors.end - constructors.start) % 4 == 0
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let chip = ChipDescriptor::from_file(root.join("configs/chips/atsamd51.yaml")).unwrap();
    assert_eq!(chip.cpu_hz, 120000000);
    let manifest: SystemManifest = serde_yaml::from_str(
        "name: pxt-gc-allocation-routing\nchip: ../chips/atsamd51.yaml\nexternal_devices: []\n",
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
    for vector in 1..256 {
        let word = machine.bus.read_u32(vector * 4).unwrap();
        let expected = match vector {
            1 => "reset_entry",
            15 => "runtime_systick",
            16 => "authored_gc_routing_irq0",
            _ => "runtime_fault",
        };
        assert_eq!(word & 1, 1);
        assert_eq!(u64::from(word & !1), symbol(expected) & !1);
    }
    let data = symbol("_data_start")..symbol("_data_end");
    let bss = symbol("_bss_start")..symbol("_bss_end");
    let expected_data: Vec<_> = (0..data.end - data.start)
        .step_by(4)
        .map(|offset| machine.bus.read_u32(symbol("_data_load") + offset).unwrap())
        .collect();
    for address in data.clone().step_by(4) {
        machine.bus.write_u32(address, 0x5a5a5a5a).unwrap();
    }
    for address in bss.clone().step_by(4) {
        machine.bus.write_u32(address, 0xa5a5a5a5).unwrap();
    }
    let constructor_entries: Vec<_> = constructors
        .step_by(4)
        .map(|a| machine.bus.read_u32(a).unwrap() & !1)
        .collect();
    let mut entered = std::collections::BTreeSet::new();
    let mut initialized = false;
    let mut startup = false;
    let mut stages = [false; 4];
    let mut previous = 0;
    let mut active: Option<Allocation> = None;
    let mut routes = Vec::new();
    let mut irq_calls = 0;
    let mut scan_calls = 0;
    let bound = 12_000_000;
    for step in 1..=bound {
        let pc = machine.cpu.pc & !1;
        let in_gc =
            (machine.bus.read_u32(in_gc_address & !3).unwrap() >> ((in_gc_address & 3) * 8)) & 0xff;
        let stage = machine.bus.read_u32(addresses[1]).unwrap();
        if u64::from(pc) == init_entry && !initialized {
            for address in bss.clone().step_by(4) {
                assert_eq!(machine.bus.read_u32(address).unwrap(), 0);
            }
            for (address, expected) in data.clone().step_by(4).zip(&expected_data) {
                assert_eq!(machine.bus.read_u32(address).unwrap(), *expected);
            }
            assert!(machine.cpu.primask && machine.cpu.active_exception == 0);
            initialized = true;
        }
        if constructor_entries.contains(&pc) {
            entered.insert(pc);
        }
        if u64::from(pc) == probe_entry && !startup {
            assert!(initialized && constructor_entries.iter().all(|x| entered.contains(x)));
            startup = true;
        }
        if startup {
            if pc == irq_entry {
                assert_eq!(machine.cpu.active_exception, 16);
                irq_calls += 1;
            }
            if pc == scan_entry {
                assert_eq!(machine.cpu.active_exception, 0);
                assert_ne!(in_gc, 0);
                scan_calls += 1;
            }
            if pc == malloc_entry {
                assert!(active.is_none(), "unexpected nested public allocation");
                assert_eq!(
                    routes.len() as u32 + 1,
                    stage,
                    "one ordered public allocation per context"
                );
                let size = match stage {
                    1 => 17,
                    2 => 29,
                    3 => 37,
                    _ => panic!("unexpected allocation stage"),
                };
                assert_eq!(machine.cpu.r0, size);
                assert_eq!(
                    machine.cpu.active_exception,
                    if stage == 2 { 16 } else { 0 }
                );
                assert_eq!(in_gc != 0, stage == 3);
                active = Some(Allocation {
                    stage,
                    size,
                    exception: machine.cpu.active_exception,
                    in_gc,
                    return_pc: machine.cpu.lr & !1,
                    selected: false,
                });
            }
            if let Some(call) = active.as_mut() {
                if !call.selected && (pc == app_entry || pc == raw_entry) {
                    assert_eq!(machine.cpu.r0, call.size);
                    assert_eq!(machine.cpu.active_exception, call.exception);
                    assert_eq!(in_gc, call.in_gc);
                    let expected = if call.stage == 1 {
                        app_entry
                    } else {
                        raw_entry
                    };
                    println!("PXT_GC_ROUTING_CALLEE source={source} elf_sha256={hash} step={step} stage={} bytes={} exception={} in_gc={} actual={pc:#x} expected={expected:#x} guest_inputs_unchanged=true", call.stage, call.size, call.exception, call.in_gc);
                    let reason = match call.stage {
                        1 => "thread allocation must select actual app_alloc",
                        2 => "IRQ allocation must select actual device_malloc",
                        _ => "in-GC allocation must select actual device_malloc",
                    };
                    assert_eq!(pc, expected, "{reason}");
                    call.selected = true;
                }
                if pc == call.return_pc {
                    assert!(
                        call.selected,
                        "public allocation returned without observed route"
                    );
                    assert_eq!(machine.cpu.active_exception, call.exception);
                    assert_eq!(in_gc, call.in_gc);
                    let pointer = machine.cpu.r0;
                    assert!((0x20000000..0x2002e000).contains(&pointer) && pointer % 4 == 0);
                    println!("PXT_GC_ROUTING_RETURN source={source} elf_sha256={hash} step={step} stage={} pointer={pointer} exception={} in_gc={} guest_inputs_unchanged=true", call.stage, machine.cpu.active_exception, in_gc);
                    routes.push((call.stage, pointer));
                    active = None;
                }
            }
        }
        machine
            .step()
            .expect("actual routing caller architectural fault");
        assert!([0, 16].contains(&machine.cpu.active_exception));
        if !startup {
            continue;
        }
        let v = addresses.map(|a| machine.bus.read_u32(a).unwrap());
        assert!(
            [0, 0x100, 0x600d].contains(&v[0]),
            "routing guest panic status={:#x} stage={} checks={}",
            v[0],
            v[1],
            v[2]
        );
        if v[1] != previous {
            assert_eq!(v[1], previous + 1);
            stages[v[1] as usize - 1] = true;
            previous = v[1];
            println!("PXT_GC_ROUTING_STAGE source={source} elf_sha256={hash} step={step} stage={} checks={}", v[1], v[2]);
        }
        if v[0] == 0x600d {
            assert!(active.is_none() && machine.cpu.primask && machine.cpu.active_exception == 0);
            assert_eq!(
                machine.bus.read_u32(in_gc_address & !3).unwrap() >> ((in_gc_address & 3) * 8)
                    & 0xff,
                0
            );
            assert_eq!(machine.bus.read_u32(0xe000ed28).unwrap(), 0);
            assert_eq!(machine.bus.read_u32(0xe000ed2c).unwrap(), 0);
            terminal(v, stages, &routes, irq_calls, scan_calls);
            println!("PXT_GC_ROUTING_CALLER engine={ENGINE} source={source} elf_sha256={hash} steps={step} bound={bound} witnesses={v:?} irq_entries={irq_calls} scan_entries={scan_calls} constructors={} native_arcade=false performance_measured=false", entered.len());
            return;
        }
    }
    let final_values = addresses.map(|a| machine.bus.read_u32(a).unwrap());
    println!("PXT_GC_ROUTING_BOUND source={source} elf_sha256={hash} bound={bound} witnesses={final_values:?} pc={:#x} exception={} irq_entries={irq_calls} scan_entries={scan_calls} routes={routes:?} qualified=false", machine.cpu.pc, machine.cpu.active_exception);
    panic!("actual allocation-routing caller did not reach terminal within {bound} steps");
}
