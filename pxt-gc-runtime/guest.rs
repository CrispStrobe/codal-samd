// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//! Owned GC consumer fixture against an immutable, otherwise unmodified engine.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::{
    bus::SystemBus, system::cortex_m::configure_cortex_m, Bus, Machine, SimulationObserver,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

const ENGINE: &str = "73f8475f8e406b1ff4475bc9886e22479881d67a";
const NAMES: [&str; 7] = [
    "runtime_status",
    "gc_probe_stage",
    "gc_probe_checks",
    "gc_worker_ready",
    "gc_worker_done",
    "gc_public_pointer",
    "gc_native_pointer",
];
#[derive(Debug)]
struct Entries {
    swap: u32,
    swaps: AtomicU32,
    restores: AtomicU32,
    restore: u32,
}
impl SimulationObserver for Entries {
    fn on_step_start(&self, pc: u32, _opcode: u32) {
        if pc == self.swap {
            self.swaps.fetch_add(1, Ordering::Relaxed);
        }
        if pc == self.restore {
            self.restores.fetch_add(1, Ordering::Relaxed);
        }
    }
}
fn terminal(
    v: [u32; 7],
    stages: [bool; 4],
    swaps: u32,
    startup: bool,
    masked: bool,
    exception: u32,
) {
    assert_eq!(v[0], 0x600d);
    assert_eq!(v[1], 4);
    assert_eq!(v[2], 60);
    assert_eq!(&v[3..5], &[1, 1]);
    assert!(v[5] != v[6]);
    assert!((0x20000000..0x2002e000).contains(&v[5]) && v[5] % 4 == 0);
    assert!((0x20000000..0x2002e000).contains(&v[6]) && v[6] % 8 == 0);
    assert!(stages.iter().all(|seen| *seen));
    assert!(swaps >= 2 && startup && masked && exception == 0);
}
#[test]
fn manufactured_gc_terminal_controls_refuse_missing_witnesses() {
    let good = [0x600d, 4, 60, 1, 1, 0x20001004, 0x20002000];
    terminal(good, [true; 4], 2, true, true, 0);
    for i in 0..good.len() {
        let mut bad = good;
        bad[i] = 0;
        assert!(std::panic::catch_unwind(|| terminal(bad, [true; 4], 2, true, true, 0)).is_err());
    }
    for i in 0..4 {
        let mut stages = [true; 4];
        stages[i] = false;
        assert!(std::panic::catch_unwind(|| terminal(good, stages, 2, true, true, 0)).is_err());
    }
    for (swaps, startup, masked, exception) in [
        (1, true, true, 0),
        (2, false, true, 0),
        (2, true, false, 0),
        (2, true, true, 15),
    ] {
        assert!(std::panic::catch_unwind(|| terminal(
            good, [true; 4], swaps, startup, masked, exception
        ))
        .is_err());
    }
}

#[test]
#[ignore = "hosted source-bound GC execution only; not Arcade admission"]
fn actual_pxt_gc_preserves_global_current_and_suspended_fiber_roots() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_FIXTURE_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["status"], "GC_SERVICES_LINK_ONLY");
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
    assert_eq!(symbol("runtime_vectors"), 0);
    assert_eq!(symbol("runtime_vectors_end"), 1024);
    let constructors = symbol("_init_array_start")..symbol("_init_array_end");
    assert!(
        constructors.start < constructors.end && (constructors.end - constructors.start) % 4 == 0
    );
    let calls = Arc::new(Entries {
        swap: (symbol("swap_context") & !1) as u32,
        restore: (symbol("restore_register_context") & !1) as u32,
        swaps: AtomicU32::new(0),
        restores: AtomicU32::new(0),
    });
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let chip = ChipDescriptor::from_file(root.join("configs/chips/atsamd51.yaml")).unwrap();
    assert_eq!(chip.cpu_hz, 120000000);
    let manifest: SystemManifest = serde_yaml::from_str(
        "name: pxt-gc-caller\nchip: ../chips/atsamd51.yaml\nexternal_devices: []\n",
    )
    .unwrap();
    let mut bus = SystemBus::from_config(&chip, &manifest).unwrap();
    let (cpu, _) = configure_cortex_m(&mut bus);
    let mut machine = Machine::new(cpu, bus);
    machine.add_observer(calls.clone());
    machine
        .load_firmware(&labwired_loader::load_elf(&path).unwrap())
        .unwrap();
    assert_eq!(machine.bus.read_u32(0).unwrap(), 0x20030000);
    for vector in 1..256 {
        let word = machine.bus.read_u32(vector * 4).unwrap();
        let target = match vector {
            1 => "reset_entry",
            15 => "runtime_systick",
            _ => "runtime_fault",
        };
        assert_eq!(word & 1, 1);
        assert_eq!(u64::from(word & !1), symbol(target) & !1);
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
    let mut startup = false;
    let mut initialized_memory = false;
    let mut stages = [false; 4];
    let mut previous = 0;
    let mut constructor_calls = std::collections::BTreeSet::new();
    let constructor_entries: Vec<_> = constructors
        .step_by(4)
        .map(|a| machine.bus.read_u32(a).unwrap() & !1)
        .collect();
    let bound = 12_000_000;
    for step in 1..=bound {
        let pc = machine.cpu.pc & !1;
        if u64::from(pc) == symbol("gc_static_init_entry") & !1 && !initialized_memory {
            for address in bss.clone().step_by(4) {
                assert_eq!(machine.bus.read_u32(address).unwrap(), 0);
            }
            for (address, expected) in data.clone().step_by(4).zip(&expected_data) {
                assert_eq!(machine.bus.read_u32(address).unwrap(), *expected);
            }
            assert!(machine.cpu.primask && machine.cpu.active_exception == 0);
            initialized_memory = true;
        }
        if constructor_entries.contains(&pc) {
            constructor_calls.insert(pc);
        }
        if u64::from(pc) == symbol("runtime_probe") & !1 && !startup {
            // Actual GC constructors have run; do not wrongly require all BSS
            // to remain zero after legitimate static initialization.
            assert!(initialized_memory);
            assert!(constructor_entries
                .iter()
                .all(|pc| constructor_calls.contains(pc)));
            startup = true;
        }
        machine
            .step()
            .expect("actual GC caller architectural fault");
        assert!([0, 15].contains(&machine.cpu.active_exception));
        if !startup {
            continue;
        }
        let v = addresses.map(|a| machine.bus.read_u32(a).unwrap());
        assert!(
            [0, 0x100, 0x600d].contains(&v[0]),
            "guest panic status={:#x} stage={} checks={}",
            v[0],
            v[1],
            v[2]
        );
        if v[1] != previous {
            assert_eq!(v[1], previous + 1);
            stages[v[1] as usize - 1] = true;
            previous = v[1];
        }
        if v[0] == 0x600d {
            let swaps = calls.swaps.load(Ordering::Relaxed);
            terminal(
                v,
                stages,
                swaps,
                startup,
                machine.cpu.primask,
                machine.cpu.active_exception,
            );
            assert_eq!(machine.bus.read_u32(0xe000ed28).unwrap(), 0);
            assert_eq!(machine.bus.read_u32(0xe000ed2c).unwrap(), 0);
            println!("PXT_GC_CALLER engine={ENGINE} source={source} elf_sha256={hash} steps={step} bound={bound} witnesses={v:?} swaps={swaps} restores={} constructors={} native_arcade=false performance_measured=false", calls.restores.load(Ordering::Relaxed),constructor_calls.len());
            return;
        }
    }
    panic!("actual GC caller did not reach terminal within {bound} steps");
}
