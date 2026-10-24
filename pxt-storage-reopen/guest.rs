// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//! Separate actual engine processes with exact completed-write backing transport.
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
#[ignore = "hosted source-bound actual guest producer only"]
fn actual_export_completed_backing() {
    run(0);
}
#[test]
#[ignore = "hosted separate fresh process only"]
fn actual_reopen_completed_backing() {
    run(1);
}
#[test]
#[ignore = "hosted separate fresh negative process only"]
fn actual_reopen_lost_backing_refuses_mount() {
    run(2);
}

fn run(mode: u32) {
    let lost = mode == 2;
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
        serde_yaml::from_str(include_str!("bw_pxt_reopen_peripherals.yaml")).unwrap();
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
    let backing_dir = folder.join("completed-backing");
    let descriptor_hash = digest(include_bytes!("bw_pxt_reopen_peripherals.yaml"));
    let profile = if cfg!(feature = "event-scheduler") {
        "event-scheduler"
    } else {
        "default"
    };
    let identity = Identity {
        schema: 1,
        engine: ENGINE.into(),
        source: source.clone(),
        elf: hash.clone(),
        descriptor: descriptor_hash,
        profile: profile.into(),
        base: 0,
        bytes: 524288,
        storage_start: 0x78000,
        storage_bytes: 32768,
        producer_pid: std::process::id(),
        full_sha256: String::new(),
        storage_sha256: String::new(),
    };
    let retained = if mode == 0 {
        vec![]
    } else {
        let bytes = read_backing(&backing_dir, &identity);
        assert_eq!(&bytes[..0x78000], &machine.bus.flash.data[..0x78000]);
        machine.bus.flash.data.copy_from_slice(&bytes);
        if lost {
            machine.bus.flash.data[0x78000..0x80000].fill(0xff);
        }
        println!("PXT_REOPEN_FRESH source={source} elf_sha256={hash} pid={} mode={mode} cpu_bus_ram_controllers=new before_instruction=true snapshot=false",std::process::id());
        bytes[0x78000..].to_vec()
    };
    let mut storage = ReopenState {
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
        boot: if mode == 0 { 1 } else { 2 },
        phase: 0,
        lower: machine.bus.flash.data[..0x78000].to_vec(),
        retained,
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
            println!("PXT_REOPEN_MEMORY source={source} elf_sha256={hash} step={step} boot={} data=true bss=true masked=true",storage.boot);
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
            println!("PXT_REOPEN_STARTUP source={source} elf_sha256={hash} step={step} boot={} constructors={} real=true",storage.boot,constructor_calls.len());
        }
        if startup {
            if storage.before(&machine, &source, &hash, step) {
                return;
            }
        }
        machine
            .step()
            .expect("actual backing guest architectural fault");
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
        if mode == 0 && storage.phase == 1 {
            assert_eq!(v[0], 0x100);
            assert_eq!(v[1], 4);
            assert_eq!(v[2], 60);
            assert_eq!(&v[3..5], &[1, 1]);
            assert!(
                stages.iter().all(|v| *v) && startup && initialized_memory && realloc_copies > 0
            );
            assert!(machine.cpu.primask && machine.cpu.active_exception == 0);
            assert_ne!(v[5], v[6]);
            assert!((0x20000000..0x2002e000).contains(&v[5]) && v[5] % 4 == 0);
            assert!((0x20000000..0x2002e000).contains(&v[6]) && v[6] % 8 == 0);
            assert!(calls.swaps.load(Ordering::Relaxed) >= 2);
            assert!(storage.pending.is_empty() && !machine.bus.has_pending_flash_op());
            assert_eq!(&storage.returns[..8], &[1, 1, 2, 3, 1, 1, 1, 2]);
            assert_eq!(machine.bus.read_u32(storage.data).unwrap(), 0xdeadbeef);
            assert_eq!(machine.bus.read_u32(storage.bss).unwrap(), 0xa5a55a5a);
            assert_eq!(machine.bus.read_u32(0xe000ed28).unwrap(), 0);
            assert_eq!(machine.bus.read_u32(0xe000ed2c).unwrap(), 0);
            println!("PXT_REOPEN_BASE source={source} elf_sha256={hash} mode={mode} witnesses={v:?} swaps={} restores={} constructors={} real=true",calls.swaps.load(Ordering::Relaxed),calls.restores.load(Ordering::Relaxed),constructor_calls.len());
            write_backing(&backing_dir, identity, &machine.bus.flash.data);
            println!("PXT_REOPEN_EXPORT_PASS source={source} engine={ENGINE} elf_sha256={hash} step={step} checks=270 pid={} pending=false completed=true bytes=524288",std::process::id());
            return;
        }
        if v[0] == 0x600d {
            assert!(mode == 1 && !lost && storage.boot == 2 && storage.phase == 2);
            assert!(
                realloc_copies > 0
                    && storage.pending.is_empty()
                    && !machine.bus.has_pending_flash_op()
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
            assert_eq!(&storage.returns[..8], &[1, 0, 1, 0, 3, 0, 2, 2]);
            assert_eq!(
                &machine.bus.flash.data[0x78000..],
                storage.retained.as_slice()
            );
            println!("PXT_REOPEN_BASE source={source} elf_sha256={hash} mode={mode} witnesses={v:?} swaps={swaps} restores={} constructors={} real=true",calls.restores.load(Ordering::Relaxed),constructor_calls.len());
            println!("PXT_REOPEN_FINITE_PASS source={source} engine={ENGINE} elf_sha256={hash} step={step} bound={bound} checks=287 pid={} unchanged=true processes=separate power_loss=false full_board=false",std::process::id());
            return;
        }
    }
    let last = addresses.map(|a| machine.bus.read_u32(a).unwrap());
    println!("PXT_STORAGE_BASE_GC_BOUND_DIAGNOSTIC source={source} elf_sha256={hash} bound={bound} witnesses={last:?} pc={:#x} lr={:#x} sp={:#x} exception={} mask={} memset_steps={memset_steps} memcpy_steps={memcpy_steps} realloc_copies={realloc_copies} swaps={} restores={} startup={startup} constructors={} native_arcade=false qualified=false", machine.cpu.pc, machine.cpu.lr, machine.cpu.sp, machine.cpu.active_exception, machine.cpu.primask, calls.swaps.load(Ordering::Relaxed), calls.restores.load(Ordering::Relaxed),constructor_calls.len());
    panic!("actual RAFFS/settings composition did not reach terminal within {bound} steps");
}

type StorageMachine = Machine<labwired_core::cpu::cortex_m::CortexM>;
struct ReopenState {
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
    lower: Vec<u8>,
    retained: Vec<u8>,
    lost: bool,
}
impl ReopenState {
    fn before(&mut self, m: &StorageMachine, source: &str, hash: &str, step: u64) -> bool {
        let pc = m.cpu.pc & !1;
        self.candidate = None;
        if m.cpu.active_exception == 0 {
            while self.pending.last().is_some_and(|p| p.0 == pc) {
                let (_, kind) = self.pending.pop().unwrap();
                self.returns[kind] += 1;
                self.last_r0[kind] = m.cpu.r0;
                println!("PXT_REOPEN_RETURN source={source} elf_sha256={hash} step={step} boot={} kind={kind} ordinal={} return_pc={pc:#x} r0={}",self.boot,self.returns[kind],m.cpu.r0);
                if self.boot == 2 && kind == 2 {
                    if self.lost {
                        assert_eq!(m.cpu.r0, 0);
                        assert_eq!(self.returns[0], 1);
                        assert_eq!(self.returns[1], 0);
                        assert_eq!(self.returns[3], 0);
                        assert!(!m.bus.has_pending_flash_op());
                        assert!(self.pending.is_empty());
                        assert!(m.bus.flash.data[0x78000..0x80000]
                            .iter()
                            .all(|b| *b == 0xff));
                        assert_eq!(&m.bus.flash.data[..0x78000], self.lower.as_slice());
                        println!("PXT_REOPEN_LOST_BACKING_REFUSED source={source} elf_sha256={hash} step={step} constructors=1 formats=0 mount_return=0 before_format=true erased=true");
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
                println!("PXT_REOPEN_ENTRY source={source} elf_sha256={hash} step={step} boot={} kind={kind} entry_pc={pc:#x} return_pc={ret:#x} retired=true",self.boot);
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
            println!("PXT_REOPEN_STAGE source={source} elf_sha256={hash} step={step} boot={} phase={phase} checks={checks} canaries=true",self.boot);
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    schema: u32,
    engine: String,
    source: String,
    elf: String,
    descriptor: String,
    profile: String,
    base: u32,
    bytes: u32,
    storage_start: u32,
    storage_bytes: u32,
    producer_pid: u32,
    full_sha256: String,
    storage_sha256: String,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn bounded_file(path: &std::path::Path, exact: Option<usize>, cap: usize) -> Vec<u8> {
    use std::io::Read;
    let meta = std::fs::symlink_metadata(path).unwrap();
    assert!(meta.is_file() && !meta.file_type().is_symlink() && meta.len() <= cap as u64);
    if let Some(n) = exact {
        assert_eq!(meta.len(), n as u64);
    }
    let mut raw = Vec::new();
    std::fs::File::open(path)
        .unwrap()
        .take(cap as u64 + 1)
        .read_to_end(&mut raw)
        .unwrap();
    assert!(raw.len() <= cap);
    if let Some(n) = exact {
        assert_eq!(raw.len(), n);
    }
    raw
}
fn read_backing(folder: &std::path::Path, expected: &Identity) -> Vec<u8> {
    let meta = std::fs::symlink_metadata(folder).unwrap();
    assert!(meta.is_dir() && !meta.file_type().is_symlink());
    let raw = bounded_file(&folder.join("manifest.json"), None, 4096);
    let manifest: Identity = serde_json::from_slice(&raw).unwrap();
    assert_eq!(
        serde_json::to_vec(&manifest).unwrap(),
        raw,
        "canonical envelope, no duplicate/unknown fields"
    );
    let mut identity = manifest.clone();
    identity.producer_pid = expected.producer_pid;
    identity.full_sha256 = String::new();
    identity.storage_sha256 = String::new();
    assert_eq!(
        &identity, expected,
        "source, image, descriptor, profile and geometry join"
    );
    assert!(
        manifest.producer_pid > 0 && manifest.producer_pid != std::process::id(),
        "separate producer process"
    );
    assert_eq!(expected.base, 0);
    assert_eq!(expected.bytes, 524288);
    assert_eq!(expected.storage_start, 0x78000);
    assert_eq!(expected.storage_bytes, 32768);
    let bytes = bounded_file(&folder.join("flash.bin"), Some(524288), 524288);
    assert_eq!(digest(&bytes), manifest.full_sha256);
    assert_eq!(digest(&bytes[0x78000..]), manifest.storage_sha256);
    bytes
}
fn write_backing(folder: &std::path::Path, mut identity: Identity, bytes: &[u8]) {
    use std::io::Write;
    assert_eq!(bytes.len(), 524288);
    assert_eq!(identity.producer_pid, std::process::id());
    identity.full_sha256 = digest(bytes);
    identity.storage_sha256 = digest(&bytes[0x78000..]);
    std::fs::create_dir(folder).unwrap();
    for (name, raw) in [
        ("flash.bin", bytes.to_vec()),
        ("manifest.json", serde_json::to_vec(&identity).unwrap()),
    ] {
        let path = folder.join(name);
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        file.write_all(&raw).unwrap();
        file.sync_all().unwrap();
        drop(file);
        assert_eq!(
            bounded_file(&path, Some(raw.len()), 524288),
            raw,
            "closed file readback"
        );
    }
    println!("PXT_REOPEN_BACKING producer_pid={} full_sha256={} storage_sha256={} bytes=524288 closed_readback=true physical_durability=false",identity.producer_pid,identity.full_sha256,identity.storage_sha256);
}
#[test]
fn manufactured_backing_admission_refuses_changed_or_incomplete_inputs() {
    let folder = std::env::current_dir()
        .unwrap()
        .join(format!("owned-reopen-controls-{}", std::process::id()));
    std::fs::create_dir(&folder).unwrap();
    let bytes = vec![0xff; 524288];
    let expected = Identity {
        schema: 1,
        engine: ENGINE.into(),
        source: "a".repeat(40),
        elf: "b".repeat(64),
        descriptor: "c".repeat(64),
        profile: "default".into(),
        base: 0,
        bytes: 524288,
        storage_start: 0x78000,
        storage_bytes: 32768,
        producer_pid: std::process::id(),
        full_sha256: String::new(),
        storage_sha256: String::new(),
    };
    let mut manifest = expected.clone();
    manifest.producer_pid = if std::process::id() == 1 { 2 } else { 1 };
    manifest.full_sha256 = digest(&bytes);
    manifest.storage_sha256 = digest(&bytes[0x78000..]);
    let original = serde_json::to_vec(&manifest).unwrap();
    let path = folder.join("manifest.json");
    let backing = folder.join("flash.bin");
    std::fs::write(&path, &original).unwrap();
    std::fs::write(&backing, &bytes).unwrap();
    assert_eq!(read_backing(&folder, &expected), bytes);
    let mut bads = Vec::new();
    for field in [
        "engine",
        "source",
        "elf",
        "descriptor",
        "profile",
        "full_sha256",
        "storage_sha256",
    ] {
        let mut value = serde_json::to_value(&manifest).unwrap();
        value[field] = serde_json::json!("changed");
        let changed: Identity = serde_json::from_value(value).unwrap();
        assert_ne!(
            changed, manifest,
            "each negative must actually alter its field"
        );
        bads.push(serde_json::to_vec(&changed).unwrap());
    }
    for field in [
        "schema",
        "base",
        "bytes",
        "storage_start",
        "storage_bytes",
        "producer_pid",
    ] {
        let mut value = serde_json::to_value(&manifest).unwrap();
        value[field] = serde_json::json!(if field == "base" { 1 } else { 0 });
        let changed: Identity = serde_json::from_value(value).unwrap();
        assert_ne!(
            changed, manifest,
            "each negative must actually alter its field"
        );
        bads.push(serde_json::to_vec(&changed).unwrap());
    }
    bads.push([original.clone(), b"\n".to_vec()].concat());
    bads.push(original[..original.len() - 1].to_vec());
    let mut unknown = serde_json::to_value(&manifest).unwrap();
    unknown["extra"] = serde_json::json!(1);
    bads.push(serde_json::to_vec(&unknown).unwrap());
    bads.push(
        original
            .iter()
            .copied()
            .take(original.len() - 1)
            .chain(b",\"schema\":1}".iter().copied())
            .collect(),
    );
    for bad in bads {
        std::fs::write(&path, bad).unwrap();
        assert!(std::panic::catch_unwind(|| read_backing(&folder, &expected)).is_err());
    }
    std::fs::write(&path, &original).unwrap();
    for bad in [bytes[..524287].to_vec(), vec![0; 524288]] {
        std::fs::write(&backing, bad).unwrap();
        assert!(std::panic::catch_unwind(|| read_backing(&folder, &expected)).is_err());
    }
    std::fs::remove_file(&backing).unwrap();
    assert!(std::panic::catch_unwind(|| read_backing(&folder, &expected)).is_err());
    std::fs::remove_file(&path).unwrap();
    std::fs::remove_dir(&folder).unwrap();
}
