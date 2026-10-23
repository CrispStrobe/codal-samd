// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//! Actual guest SYSRESETREQ with second startup and retained flash backing.
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

const ENGINE: &str = "79071f69cd6035a820cbdad93da7343d7d501e4d";
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
fn actual_guest_reset_retains_storage() {
    run(false);
}

#[test]
#[ignore = "hosted actual guest only"]
fn actual_reset_lost_backing_refuses_mount() {
    run(true);
}

fn run(lost: bool) {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_FIXTURE_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(
        report["status"],
        "ACTUAL_RAFFS_SETTINGS_GC_SERVICES_LINK_ONLY"
    );
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
    let path = folder.join("storage-control.elf");
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
    let init_entry = symbol("gc_static_init_entry") & !1;
    let probe_entry = symbol("runtime_probe") & !1;
    let function = |name| {
        let matches: Vec<_> = elf
            .syms
            .iter()
            .filter(|s| s.st_shndx != 0 && elf.strtab.get_at(s.st_name) == Some(name))
            .collect();
        assert_eq!(matches.len(), 1, "unique diagnostic function {name}");
        let start = u32::try_from(matches[0].st_value & !1).unwrap();
        let size = u32::try_from(matches[0].st_size).unwrap();
        assert!(size > 0);
        start..start.checked_add(size).unwrap()
    };
    let memset_code = function("memset");
    let memcpy_code = function("memcpy");
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
    let mut chip = ChipDescriptor::from_file(root.join("configs/chips/atsamd51.yaml")).unwrap();
    let storage: Vec<labwired_config::PeripheralConfig> =
        serde_yaml::from_str(include_str!("bw_pxt_reset_peripherals.yaml")).unwrap();
    assert_eq!(storage.len(), 2);
    chip.peripherals.extend(storage);
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
    assert!(machine.bus.models_flash_ops());
    assert_eq!(symbol("_program_load_end") <= 0x77ff0, true);
    machine.bus.flash.data[0x78000..0x80000].fill(0xff);
    machine.bus.flash.data[0x77ff0..0x78000].copy_from_slice(&[0x5a; 16]);
    let patterns = [
        "_ZN3pxt5raffs2FSC",
        "_ZN3pxt5raffs2FS6formatE",
        "_ZN3pxt5raffs2FS8tryMountE",
        "_ZN8settings4_setE",
        "_ZN8settings4_getE",
        "_ZN8settings7_removeE",
        "_ZN8settings7_existsE",
        "_ZN8settings5_listE",
        "_ZN3pxt9hash_fnv1E",
        "_ZN3pxt2gcEi",
    ];
    let entries = patterns
        .iter()
        .enumerate()
        .map(|(kind, pattern)| {
            let mut matches: Vec<_> = elf
                .syms
                .iter()
                .filter(|s| {
                    s.st_shndx != 0
                        && elf
                            .strtab
                            .get_at(s.st_name)
                            .is_some_and(|n| n.contains(*pattern))
                })
                .map(|s| (s.st_value & !1) as u32)
                .collect();
            matches.sort();
            matches.dedup();
            assert_eq!(matches.len(), 1, "unique actual storage method {pattern}");
            (matches[0], kind)
        })
        .collect();
    let reset_entry = (symbol("reset_entry") & !1) as u32;
    let reset_store = (symbol("bw_reset_store") & !1) as u32;
    assert_eq!(
        machine.bus.read_u16(u64::from(reset_store)).unwrap(),
        0x6001
    );
    let mut storage = ResetState {
        stage: symbol("storage_stage"),
        checks: symbol("storage_checks"),
        canary: symbol("storage_canary"),
        data: symbol("reset_data"),
        bss: symbol("reset_bss"),
        entries,
        pending: vec![],
        candidate: None,
        returns: [0; 10],
        last_r0: [0; 10],
        boot: 1,
        phase: 0,
        resets: 0,
        lower: machine.bus.flash.data[..0x78000].to_vec(),
        retained: vec![],
        lost,
    };
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
    let bound = 160_000_000;
    let mut memset_steps = 0u64;
    let mut memcpy_steps = 0u64;
    let mut realloc_copies = 0u32;
    for step in 1..=bound {
        let pc = machine.cpu.pc & !1;
        if memset_code.contains(&pc) {
            memset_steps += 1;
        }
        if memcpy_code.contains(&pc) {
            memcpy_steps += 1;
        }
        if startup && previous == 1 && pc == memcpy_code.start {
            let original = machine.bus.read_u32(addresses[5]).unwrap();
            if original != 0 && machine.cpu.r1 == original {
                let header = machine.bus.read_u32(u64::from(original - 4)).unwrap();
                assert_eq!(header >> 29, 3, "actual public GC array/permanent header");
                let words = (header & 0x0fff_ffff) >> 2;
                let capacity = words.checked_sub(1).unwrap().checked_mul(4).unwrap();
                let count = machine.cpu.r2;
                realloc_copies += 1;
                println!("PXT_STORAGE_BASE_GC_REALLOC_ENTRY source={source} elf_sha256={hash} step={step} pc={pc:#x} destination={} original={} copied_bytes={count} header={header:#x} payload_capacity={capacity} excess={} guest_inputs_unchanged=true", machine.cpu.r0, original, count.saturating_sub(capacity));
                assert_eq!(capacity, 24, "actual 23-byte array payload capacity");
                assert_eq!(
                    count, capacity,
                    "public realloc must not copy its header word"
                );
            }
        }
        if u64::from(pc) == init_entry && !initialized_memory {
            for address in bss.clone().step_by(4) {
                assert_eq!(machine.bus.read_u32(address).unwrap(), 0);
            }
            for (address, expected) in data.clone().step_by(4).zip(&expected_data) {
                assert_eq!(machine.bus.read_u32(address).unwrap(), *expected);
            }
            assert!(machine.cpu.primask && machine.cpu.active_exception == 0);
            initialized_memory = true;
            println!("PXT_RESET_MEMORY source={source} elf_sha256={hash} step={step} boot={} data=true bss=true masked=true",storage.boot);
        }
        if constructor_entries.contains(&pc) {
            constructor_calls.insert(pc);
        }
        if u64::from(pc) == probe_entry && !startup {
            // Actual GC constructors have run; do not wrongly require all BSS
            // to remain zero after legitimate static initialization.
            assert!(initialized_memory);
            assert!(constructor_entries
                .iter()
                .all(|pc| constructor_calls.contains(pc)));
            startup = true;
            println!("PXT_RESET_STARTUP source={source} elf_sha256={hash} step={step} boot={} constructors={} real=true",storage.boot,constructor_calls.len());
        }
        if startup {
            if storage.before(&machine, &source, &hash, step) {
                return;
            }
        }
        let requesting = pc == reset_store;
        if requesting {
            assert!(startup && storage.boot == 1 && storage.resets == 0 && storage.phase == 1);
            assert!(storage.pending.is_empty());
            assert_eq!(machine.cpu.r0, 0xe000ed0c);
            assert_eq!(machine.cpu.r1, 0x05fa0004);
            assert_eq!(machine.cpu.active_exception, 0);
            assert!(machine.cpu.primask);
            assert_eq!(storage.returns[0], 1);
            assert_eq!(storage.returns[1], 1);
            assert_eq!(storage.returns[3], 3);
            assert_eq!(storage.returns[5], 1);
            assert_eq!(machine.bus.read_u32(storage.data).unwrap(), 0xdeadbeef);
            assert_eq!(machine.bus.read_u32(storage.bss).unwrap(), 0xa5a55a5a);
            storage.retained = machine.bus.flash.data[0x78000..0x80000].to_vec();
            println!("PXT_RESET_REQUEST source={source} elf_sha256={hash} step={step} pc={pc:#x} opcode=0x6001 r0=0xe000ed0c r1=0x05fa0004 storage_sha256={:x} completed=true",Sha256::digest(&storage.retained));
        }
        machine
            .step()
            .expect("actual GC caller architectural fault");
        if requesting {
            assert_eq!(machine.cpu.pc & !1, reset_entry);
            assert_eq!(machine.cpu.sp, 0x20030000);
            assert!(!machine.cpu.primask && !machine.cpu.faultmask);
            assert_eq!(machine.cpu.basepri, 0);
            assert_eq!(machine.cpu.active_exception, 0);
            assert_eq!(machine.cpu.control, 0);
            assert_eq!(machine.cpu.pending_exceptions, [0; 4]);
            for address in [
                0xe000ed08, 0xe000ed10, 0xe000ed18, 0xe000ed1c, 0xe000ed20, 0xe000e010, 0xe000e100,
                0xe000e200, 0xe000e300,
            ] {
                assert_eq!(
                    machine.bus.read_u32(address).unwrap()
                        & if address == 0xe000e010 { 7 } else { u32::MAX },
                    0
                );
            }
            assert_eq!(&machine.bus.flash.data[..0x78000], storage.lower.as_slice());
            assert_eq!(
                &machine.bus.flash.data[0x78000..0x80000],
                storage.retained.as_slice()
            );
            assert_eq!(machine.bus.read_u32(storage.data).unwrap(), 0xdeadbeef);
            assert_eq!(machine.bus.read_u32(storage.bss).unwrap(), 0xa5a55a5a);
            storage.boot = 2;
            storage.resets = 1;
            storage.phase = 0;
            println!("PXT_RESET_BOUNDARY source={source} elf_sha256={hash} step={step} pc={:#x} sp={:#x} masks=true core_system=true ram_retained=true storage_sha256={:x} unchanged=true",machine.cpu.pc,machine.cpu.sp,Sha256::digest(&machine.bus.flash.data[0x78000..0x80000]));
            if lost {
                machine.bus.flash.data[0x78000..0x80000].fill(0xff);
                println!("PXT_RESET_FAULT source={source} elf_sha256={hash} step={step} kind=lost-backing after_actual_reset=true bytes=32768");
            }
            startup = false;
            initialized_memory = false;
            stages = [false; 4];
            previous = 0;
            constructor_calls.clear();
            realloc_copies = 0;
            continue;
        }
        assert!([0, 15].contains(&machine.cpu.active_exception));
        if !startup {
            continue;
        }
        let v = addresses.map(|a| machine.bus.read_u32(a).unwrap());
        storage.after(&machine, &source, &hash, step, v[0]);
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
            println!("PXT_STORAGE_BASE_GC_STAGE source={source} elf_sha256={hash} step={step} stage={} checks={} pc={pc:#x} memset_steps={memset_steps} memcpy_steps={memcpy_steps}", v[1], v[2]);
        }
        if v[0] == 0x600d {
            assert!(
                realloc_copies > 0,
                "actual public realloc copy must be observed"
            );
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
            println!("PXT_STORAGE_BASE_GC_CALLER engine={ENGINE} source={source} elf_sha256={hash} steps={step} bound={bound} witnesses={v:?} swaps={swaps} restores={} constructors={} native_arcade=false performance_measured=false", calls.restores.load(Ordering::Relaxed),constructor_calls.len());
            assert!(!lost && storage.boot == 2 && storage.resets == 1 && storage.phase == 2);
            assert!(storage.pending.is_empty());
            assert_eq!(storage.returns[0], 2);
            assert_eq!(storage.returns[1], 1);
            assert_eq!(storage.returns[3], 3);
            assert_eq!(storage.returns[4], 4);
            assert_eq!(storage.returns[5], 1);
            assert_eq!(storage.returns[6], 3);
            assert_eq!(storage.returns[7], 4);
            assert_eq!(
                &machine.bus.flash.data[0x78000..0x80000],
                storage.retained.as_slice()
            );
            println!("PXT_RESET_FINITE_PASS source={source} engine={ENGINE} elf_sha256={hash} step={step} bound={bound} boots=2 resets=1 first_checks=270 second_checks=287 returns={:?} storage_sha256={:x} unchanged=true peripheral_reset=false power_loss=false full_board=false",storage.returns,Sha256::digest(&storage.retained));
            return;
        }
    }
    let last = addresses.map(|a| machine.bus.read_u32(a).unwrap());
    println!("PXT_STORAGE_BASE_GC_BOUND_DIAGNOSTIC source={source} elf_sha256={hash} bound={bound} witnesses={last:?} pc={:#x} lr={:#x} sp={:#x} exception={} mask={} memset_steps={memset_steps} memcpy_steps={memcpy_steps} realloc_copies={realloc_copies} swaps={} restores={} startup={startup} constructors={} native_arcade=false qualified=false", machine.cpu.pc, machine.cpu.lr, machine.cpu.sp, machine.cpu.active_exception, machine.cpu.primask, calls.swaps.load(Ordering::Relaxed), calls.restores.load(Ordering::Relaxed),constructor_calls.len());
    panic!("actual RAFFS/settings composition did not reach terminal within {bound} steps");
}

type StorageMachine = Machine<labwired_core::cpu::cortex_m::CortexM>;
struct ResetState {
    stage: u64,
    checks: u64,
    canary: u64,
    data: u64,
    bss: u64,
    entries: Vec<(u32, usize)>,
    pending: Vec<(u32, usize)>,
    candidate: Option<(u32, u32, usize)>,
    returns: [u32; 10],
    last_r0: [u32; 10],
    boot: u32,
    phase: u32,
    resets: u32,
    lower: Vec<u8>,
    retained: Vec<u8>,
    lost: bool,
}
impl ResetState {
    fn before(&mut self, m: &StorageMachine, source: &str, hash: &str, step: u64) -> bool {
        let pc = m.cpu.pc & !1;
        self.candidate = None;
        if m.cpu.active_exception == 0 {
            while self.pending.last().is_some_and(|p| p.0 == pc) {
                let (_, kind) = self.pending.pop().unwrap();
                self.returns[kind] += 1;
                self.last_r0[kind] = m.cpu.r0;
                println!("PXT_RESET_RETURN source={source} elf_sha256={hash} step={step} boot={} kind={kind} ordinal={} return_pc={pc:#x} r0={}",self.boot,self.returns[kind],m.cpu.r0);
                if self.boot == 2 && kind == 2 {
                    if self.lost {
                        assert_eq!(m.cpu.r0, 0);
                        assert_eq!(self.returns[0], 2);
                        assert_eq!(self.returns[1], 1);
                        assert_eq!(self.returns[3], 3);
                        assert_eq!(self.resets, 1);
                        assert!(self.pending.is_empty());
                        assert!(m.bus.flash.data[0x78000..0x80000]
                            .iter()
                            .all(|b| *b == 0xff));
                        assert_eq!(&m.bus.flash.data[..0x78000], self.lower.as_slice());
                        println!("PXT_RESET_LOST_BACKING_REFUSED source={source} elf_sha256={hash} step={step} boots=2 resets=1 constructors=2 formats=1 mount_return=0 before_format=true erased=true");
                        return true;
                    }
                    assert_eq!(m.cpu.r0, 1, "actual second boot mount must succeed");
                }
            }
            for (entry, kind) in &self.entries {
                if *entry == pc {
                    assert!(self.candidate.is_none());
                    self.candidate = Some((pc, m.cpu.lr & !1, *kind));
                }
            }
        }
        false
    }
    fn after(&mut self, m: &StorageMachine, source: &str, hash: &str, step: u64, status: u32) {
        if let Some((pc, ret, kind)) = self.candidate.take() {
            if m.cpu.active_exception == 0 {
                assert_ne!(m.cpu.pc & !1, pc);
                self.pending.push((ret, kind));
                println!("PXT_RESET_ENTRY source={source} elf_sha256={hash} step={step} boot={} kind={kind} entry_pc={pc:#x} return_pc={ret:#x} retired=true",self.boot);
            }
        }
        assert_ne!(status & 0xe0000000, 0xe0000000, "unrelated guest panic");
        let phase = m.bus.read_u32(self.stage).unwrap();
        if phase != self.phase || status == 0x600d {
            assert_eq!(&m.bus.flash.data[..0x78000], self.lower.as_slice());
            for (i, v) in [0x13579bdf, 0x2468ace0, 0x5a5a5a5a, 0xc3c3c3c3]
                .iter()
                .enumerate()
            {
                assert_eq!(m.bus.read_u32(self.canary + i as u64 * 4).unwrap(), *v);
            }
        }
        if phase != self.phase {
            assert_eq!(phase, self.boot);
            assert_eq!(self.phase, 0);
            let checks = m.bus.read_u32(self.checks).unwrap();
            assert_eq!(checks, if self.boot == 1 { 270 } else { 287 });
            if self.boot == 2 {
                assert_eq!(
                    &m.bus.flash.data[0x78000..0x80000],
                    self.retained.as_slice()
                );
            }
            self.phase = phase;
            println!("PXT_RESET_STAGE source={source} elf_sha256={hash} step={step} boot={} phase={phase} checks={checks} canaries=true",self.boot);
        }
    }
}
