// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//! Separate actual fatal native exhaustion. Never inject allocator failures.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::{bus::SystemBus, system::cortex_m::configure_cortex_m, Bus, Machine};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const ENGINE: &str = "79071f69cd6035a820cbdad93da7343d7d501e4d";
const NAMES: [&str; 7] = [
    "runtime_status",
    "oom_stage",
    "oom_old",
    "oom_guard",
    "oom_request",
    "oom_checks",
    "codal_heap_start",
];

fn fatal(v: [u32; 7], outer: usize, raw: usize, inner: &[u32], boundaries: usize) {
    assert_eq!(
        v[0], 0xe0000014,
        "actual fatal OOM must retain DEVICE_OOM code 20"
    );
    assert_eq!((v[1], v[4], v[5]), (3, 131072, 3));
    assert_eq!((outer, raw, boundaries), (3, 3, 2));
    assert_eq!(inner.len(), 3);
    assert!(inner[0] != 0 && inner[1] != 0 && inner[2] == 0);
    assert!(v[2] >= v[6] + 12 && v[3] >= v[2] + 32 && v[3] + 131072 <= 0x2002e000);
    assert_eq!((v[2] | v[3]) & 7, 0);
}

#[test]
fn manufactured_fatal_admission_refuses_wrong_or_missing_witnesses() {
    let good = [0xe0000014, 3, 0x20002000, 0x20002040, 131072, 3, 0x20001000];
    fatal(good, 3, 3, &[0x20001ff4, 0x20002034, 0], 2);
    for i in 0..7 {
        let mut bad = good;
        bad[i] = if i == 6 { u32::MAX - 12 } else { 0 };
        assert!(std::panic::catch_unwind(|| fatal(bad, 3, 3, &[1, 1, 0], 2)).is_err());
    }
    for (outer, raw, inner, boundaries) in [
        (2, 3, vec![1, 1, 0], 2),
        (3, 2, vec![1, 1, 0], 2),
        (3, 3, vec![1, 1], 2),
        (3, 3, vec![1, 1, 1], 2),
        (3, 3, vec![1, 1, 0], 1),
    ] {
        assert!(std::panic::catch_unwind(|| fatal(good, outer, raw, &inner, boundaries)).is_err());
    }
}

#[test]
#[ignore = "hosted source-bound actual fatal exhaustion only"]
fn actual_native_exhaustion_preserves_live_payload_and_fatal_policy() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|c| c.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_NATIVE_OOM_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["status"], "GC_NATIVE_REAL_OOM_LINK_ONLY");
    assert_eq!(report["case"], "native-real-oom");
    assert_eq!(
        report["probe_source"],
        "pxt-gc-runtime/probe-native-oom.cpp"
    );
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
    let entry = |name| (symbol(name) & !1) as u32;
    let alloc = entry("bw_native_alloc");
    let realloc = entry("bw_native_realloc");
    let raw_entry = entry("device_malloc");
    let inner_entry = entry("_Z16device_malloc_injR14HeapDefinition");
    let panic_entry = entry("target_panic");
    let boundary = entry("authored_oom_boundary");
    let probe_entry = entry("runtime_probe");
    let init_entry = entry("gc_static_init_entry");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let chip = ChipDescriptor::from_file(root.join("configs/chips/atsamd51.yaml")).unwrap();
    assert_eq!(chip.cpu_hz, 120000000);
    let manifest: SystemManifest = serde_yaml::from_str(
        "name: pxt-native-real-oom\nchip: ../chips/atsamd51.yaml\nexternal_devices: []\n",
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
        let name = match i {
            1 => "reset_entry",
            15 => "runtime_systick",
            _ => "runtime_fault",
        };
        assert_eq!(
            u64::from(machine.bus.read_u32(i * 4).unwrap()),
            symbol(name) | 1
        );
    }
    let data = symbol("_data_start")..symbol("_data_end");
    let bss = symbol("_bss_start")..symbol("_bss_end");
    let initial: Vec<_> = (0..data.end - data.start)
        .step_by(4)
        .map(|o| machine.bus.read_u32(symbol("_data_load") + o).unwrap())
        .collect();
    for a in data.clone().step_by(4) {
        machine.bus.write_u32(a, 0x5a5a5a5a).unwrap();
    }
    for a in bss.clone().step_by(4) {
        machine.bus.write_u32(a, 0xa5a5a5a5).unwrap();
    }
    let constructors: Vec<_> = (symbol("_init_array_start")..symbol("_init_array_end"))
        .step_by(4)
        .map(|a| machine.bus.read_u32(a).unwrap() & !1)
        .collect();
    assert!(!constructors.is_empty());
    let mut entered = std::collections::BTreeSet::new();
    let (mut initialized, mut startup) = (false, false);
    let (mut outer, mut raw, mut boundaries) = (0usize, 0usize, 0usize);
    let mut inner = Vec::new();
    let (mut outer_return, mut inner_return) = (None, None);
    let mut panic_observed = None;
    let bound = 12_000_000;
    for step in 1..=bound {
        let pc = machine.cpu.pc & !1;
        if pc == init_entry && !initialized {
            for a in bss.clone().step_by(4) {
                assert_eq!(machine.bus.read_u32(a).unwrap(), 0);
            }
            for (a, v) in data.clone().step_by(4).zip(&initial) {
                assert_eq!(machine.bus.read_u32(a).unwrap(), *v);
            }
            initialized = true;
        }
        if constructors.contains(&pc) {
            entered.insert(pc);
        }
        if pc == probe_entry && !startup {
            assert!(initialized && constructors.iter().all(|a| entered.contains(a)));
            startup = true;
        }
        if startup {
            assert_eq!(machine.cpu.active_exception, 0);
            assert!(machine.cpu.primask);
            let v = addresses.map(|a| machine.bus.read_u32(a).unwrap());
            if outer_return.is_none() && [alloc, realloc].contains(&pc) {
                outer += 1;
                assert_eq!(outer, v[1] as usize);
                assert_eq!(pc, if outer == 3 { realloc } else { alloc });
                assert_eq!(v[4], if outer == 1 { 32 } else { 131072 });
                if outer == 3 {
                    assert_eq!((machine.cpu.r0, machine.cpu.r1), (v[2], 131072));
                } else {
                    assert_eq!(machine.cpu.r0, v[4]);
                }
                outer_return = Some(machine.cpu.lr & !1);
                println!("PXT_NATIVE_OOM_ENTRY source={source} elf_sha256={hash} step={step} stage={outer} request={} old={} actual={pc:#x}", v[4], v[2]);
            }
            if Some(pc) == outer_return {
                println!("PXT_NATIVE_OOM_RETURN source={source} elf_sha256={hash} step={step} stage={outer} pointer={}", machine.cpu.r0);
                assert!(outer < 3, "fatal replacement must not return");
                assert!(machine.cpu.r0 != 0 && machine.cpu.r0 & 7 == 0);
                outer_return = None;
            }
            if pc == raw_entry {
                raw += 1;
                assert_eq!(raw, outer);
                let request = if raw == 1 { 47 } else { 131087 };
                println!("PXT_NATIVE_OOM_RAW source={source} elf_sha256={hash} step={step} stage={raw} request={} actual={pc:#x}", machine.cpu.r0);
                assert_eq!(machine.cpu.r0, request);
            }
            if pc == inner_entry {
                assert!(inner_return.is_none());
                assert_eq!(inner.len() + 1, raw);
                assert_eq!(machine.cpu.r0, if raw == 1 { 47 } else { 131087 });
                inner_return = Some(machine.cpu.lr & !1);
            }
            if Some(pc) == inner_return {
                inner.push(machine.cpu.r0);
                println!("PXT_NATIVE_OOM_INNER_RETURN source={source} elf_sha256={hash} step={step} stage={raw} pointer={}", machine.cpu.r0);
                assert_eq!(
                    machine.cpu.r0 == 0,
                    raw == 3,
                    "only the third real raw allocation exhausts"
                );
                inner_return = None;
            }
            let verify_payloads = pc == boundary || v[0] & 0xf0000000 == 0xe0000000;
            if verify_payloads {
                for (slot, pointer, size, fill) in [(0, v[2], 32, 0x69), (1, v[3], 131072, 0x5a)] {
                    if slot == 1 && boundaries == 0 {
                        continue;
                    }
                    assert!(
                        pointer >= v[6] + 12 && pointer & 7 == 0 && pointer + size <= 0x2002e000
                    );
                    let payload: Vec<_> = (0..size)
                        .map(|i| {
                            let a = u64::from(pointer + i);
                            ((machine.bus.read_u32(a & !3).unwrap() >> ((a & 3) * 8)) & 255) as u8
                        })
                        .collect();
                    let expected = vec![fill; size as usize];
                    let expected_sha = format!("{:x}", Sha256::digest(&expected));
                    let actual_sha = format!("{:x}", Sha256::digest(&payload));
                    println!("PXT_NATIVE_OOM_PAYLOAD source={source} elf_sha256={hash} step={step} stage={} slot={slot} pointer={pointer} bytes={size} expected_sha256={expected_sha} actual_sha256={actual_sha}", v[1]);
                    assert_eq!(
                        payload, expected,
                        "fatal OOM must preserve every live payload byte"
                    );
                }
                if pc == boundary {
                    boundaries += 1;
                    assert_eq!(boundaries, outer);
                    assert_eq!(v[5], boundaries as u32 + 1);
                }
            }
            if pc == panic_entry {
                println!("PXT_NATIVE_OOM_PANIC source={source} elf_sha256={hash} step={step} stage={} reason={} raw_calls={raw} raw_nulls={} actual={pc:#x}", v[1], machine.cpu.r0, inner.iter().filter(|p| **p == 0).count());
                assert_eq!(
                    machine.cpu.r0, 20,
                    "actual fatal OOM must retain DEVICE_OOM code 20"
                );
                assert_eq!(inner, vec![inner[0], inner[1], 0]);
                panic_observed = Some(20);
            }
            if v[0] & 0xf0000000 == 0xe0000000 {
                fatal(v, outer, raw, &inner, boundaries);
                assert_eq!(panic_observed, Some(20));
                assert!(outer_return.is_some() && inner_return.is_none());
                for a in [0xe000ed28, 0xe000ed2c] {
                    assert_eq!(machine.bus.read_u32(a).unwrap(), 0);
                }
                println!("PXT_NATIVE_OOM_CALLER engine={ENGINE} source={source} elf_sha256={hash} steps={step} bound={bound} status=0xe0000014 outer=3 raw=3 raw_nulls=1 boundaries=2 checks=3 old_bytes=32 guard_bytes=131072 constructors={} native_fatal_oom_only=true public_gc_oom_qualified=false public_oversize_qualified=false allocated_dma_qualified=false native_arcade=false performance_measured=false", entered.len());
                return;
            }
        }
        machine.step().unwrap();
    }
    println!(
        "PXT_NATIVE_OOM_BOUND source={source} elf_sha256={hash} bound={bound} qualified=false"
    );
    panic!("actual native OOM did not reach expected fatal terminal within {bound} steps");
}
