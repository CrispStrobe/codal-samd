// SPDX-License-Identifier: MIT
//! Separate processes, actual settings/RAFFS and observed atomic NVM boundaries.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::{
    bus::SystemBus, system::cortex_m::configure_cortex_m, Bus, Machine, SimulationObserver,
};
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
const ENGINE: &str = "79071f69cd6035a820cbdad93da7343d7d501e4d";
const B: usize = 0x78000;
const E: usize = 0x7c000;
fn sha(b: &[u8]) -> String {
    format!("{:x}", Sha256::digest(b))
}
fn need(ok: bool, why: &str) {
    assert!(ok, "{why}");
}
fn u16at(b: &[u8], a: usize) -> u16 {
    u16::from_le_bytes(b[a..a + 2].try_into().unwrap())
}
fn u32at(b: &[u8], a: usize) -> u32 {
    u32::from_le_bytes(b[a..a + 4].try_into().unwrap())
}
fn pattern(n: usize, newer: bool) -> Vec<u8> {
    (0..n)
        .map(|i| {
            if newer {
                ((i * 37 + 71) % 251) as u8
            } else {
                ((i * 17 + 29) % 251) as u8
            }
        })
        .collect()
}
#[derive(Debug, Default)]
struct Watch {
    low: AtomicU32,
    command: AtomicU32,
    commands: AtomicU32,
    swaps: AtomicU32,
    swap: u32,
    format: AtomicU32,
    format_pc: u32,
    mount: AtomicU32,
    mount_pc: u32,
}
impl SimulationObserver for Watch {
    fn on_step_start(&self, pc: u32, _: u32) {
        if pc == self.swap {
            self.swaps.fetch_add(1, Ordering::Relaxed);
        }
        if pc == self.format_pc {
            self.format.fetch_add(1, Ordering::Relaxed);
        }
        if pc == self.mount_pc {
            self.mount.fetch_add(1, Ordering::Relaxed);
        }
    }
    fn on_memory_write(&self, a: u64, _: u8, v: u8) {
        if a == 0x41004004 {
            self.low.store(u32::from(v), Ordering::Relaxed);
        }
        if a == 0x41004005 {
            self.command.store(
                self.low.load(Ordering::Relaxed) | (u32::from(v) << 8),
                Ordering::Relaxed,
            );
            self.commands.fetch_add(1, Ordering::Relaxed);
        }
    }
}
fn put_json(path: &Path, value: &serde_json::Value) {
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    f.write_all(&serde_json::to_vec_pretty(value).unwrap())
        .unwrap();
    f.sync_all().unwrap();
}
fn export(
    root: &Path,
    name: &str,
    bytes: &[u8],
    source: &str,
    profile: &str,
    elf: &str,
    boundary: serde_json::Value,
) {
    let dir = root.join(format!("backing-{name}"));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("flash.bin");
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    f.write_all(bytes).unwrap();
    f.sync_all().unwrap();
    drop(f);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    put_json(
        &dir.join("identity.json"),
        &serde_json::json!({"schema":1,"engine":ENGINE,"source":source,"profile":profile,"elf_sha256":elf,"pid":std::process::id(),"bytes":bytes.len(),"full_sha256":sha(bytes),"storage_sha256":sha(&bytes[B..]),"boundary":boundary,"cpu_snapshot":false,"physical_power_loss":false}),
    );
}
fn read_backing(
    root: &Path,
    name: &str,
    source: &str,
    profile: &str,
) -> (Vec<u8>, serde_json::Value) {
    let dir = root.join(format!("backing-{name}"));
    let id: serde_json::Value =
        serde_json::from_slice(&std::fs::read(dir.join("identity.json")).unwrap()).unwrap();
    let raw = std::fs::read(dir.join("flash.bin")).unwrap();
    assert_eq!(id["engine"], ENGINE);
    assert_eq!(id["source"], source);
    assert_eq!(id["profile"], profile);
    assert_ne!(id["pid"], std::process::id());
    assert_eq!(raw.len(), 524288);
    assert_eq!(id["full_sha256"], sha(&raw));
    assert_eq!(id["storage_sha256"], sha(&raw[B..]));
    (raw, id)
}
fn baseline(raw: &[u8]) -> usize {
    assert_eq!(
        [
            u32at(raw, B),
            u32at(raw, B + 4),
            u32at(raw, B + 8),
            u32at(raw, B + 12)
        ],
        [0x7776e0da, 0x8000, 1, 0xffffffff]
    );
    assert!(raw[E..].iter().all(|v| *v == 255));
    let old = E - 8;
    let keep = E - 16;
    assert_eq!(u16at(raw, old + 4), 97);
    assert_eq!(u16at(raw, keep + 4), 19);
    for (a, name, size) in [
        (old, b"atomic.value\0".as_slice(), 97),
        (keep, b"atomic.keep\0".as_slice(), 19),
    ] {
        let n = B + usize::from(u16at(raw, a + 2));
        assert_eq!(&raw[n..n + name.len()], name);
        let p = B + usize::from(u16at(raw, a + 6));
        assert_eq!(&raw[p..p + size], pattern(size, false));
    }
    assert!(raw[E - 32..E - 16].iter().all(|v| *v == 255));
    let next = usize::from(u16at(raw, keep + 6)) + 19;
    (next + 7) & !7
}
fn new_meta(raw: &[u8], payload: usize) -> [u8; 8] {
    let mut v = [0u8; 8];
    v[..4].copy_from_slice(&raw[E - 8..E - 4]);
    v[4..6].copy_from_slice(&(0x8000u16 | 257).to_le_bytes());
    v[6..].copy_from_slice(&(payload as u16).to_le_bytes());
    v
}
fn invariants(raw: &[u8], base: &[u8], payload: usize, meta: &[u8; 8], published: bool) {
    assert_eq!(
        &raw[B..B + 16],
        &base[B..B + 16],
        "bank header unchanged/no GC"
    );
    assert_eq!(&raw[E..], &base[E..], "inactive bank unchanged");
    assert_eq!(
        &raw[E - 16..E],
        &base[E - 16..E],
        "original metadata retained"
    );
    for i in B..E {
        if (B + payload..B + payload + 264).contains(&i) || (E - 24..E - 16).contains(&i) {
            continue;
        }
        assert_eq!(raw[i], base[i], "unrelated storage byte {i:x}");
    }
    assert_eq!(
        &raw[E - 24..E - 16],
        if published {
            meta.as_slice()
        } else {
            &base[E - 24..E - 16]
        },
        "publication boundary"
    );
}
#[test]
#[ignore = "hosted exact source/ELF/NVM boundary and fresh process only"]
fn actual_interruption() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    let root = PathBuf::from(std::env::var("BW_INTERRUPTION_DIR").unwrap());
    let case = std::env::var("BW_INTERRUPTION_CASE").unwrap();
    let reader = case.starts_with("read-");
    let name = if reader {
        case.trim_start_matches("read-")
    } else {
        case.trim_start_matches("write-")
    };
    let role = if case == "seed" {
        0
    } else if reader {
        2
    } else {
        1
    };
    let profile = if cfg!(feature = "event-scheduler") {
        "event-scheduler"
    } else {
        "default"
    };
    let folder = root.join(role.to_string());
    let path = folder.join("storage-control.elf");
    let bytes = std::fs::read(&path).unwrap();
    let hash = sha(&bytes);
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["elf_sha256"], hash);
    assert_eq!(report["interruption_role"], role);
    assert_eq!(
        report["composition"],
        "actual-raffs-interruption-atomic-boundary"
    );
    let elf = goblin::elf::Elf::parse(&bytes).unwrap();
    let symbol = |pattern: &str| {
        let mut rows: Vec<_> = elf
            .syms
            .iter()
            .filter(|s| {
                s.st_shndx != 0
                    && elf
                        .strtab
                        .get_at(s.st_name)
                        .is_some_and(|n| n == pattern || n.starts_with(pattern))
            })
            .map(|s| s.st_value & !1)
            .collect();
        rows.sort();
        rows.dedup();
        assert_eq!(rows.len(), 1, "unique actual symbol {pattern}");
        rows[0]
    };
    let stage = symbol("storage_stage");
    let result = symbol("storage_result");
    let status = symbol("runtime_status");
    let gc_stage = symbol("gc_probe_stage");
    let gc_checks = symbol("gc_probe_checks");
    let init = symbol("gc_static_init_entry") as u32;
    let probe = symbol("runtime_probe") as u32;
    let set = symbol("_ZN8settings4_setE") as u32;
    let watch = Arc::new(Watch {
        swap: symbol("swap_context") as u32,
        format_pc: symbol("_ZN3pxt5raffs2FS6formatE") as u32,
        mount_pc: symbol("_ZN3pxt5raffs2FS8tryMountE") as u32,
        ..Default::default()
    });
    let base_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut chip =
        ChipDescriptor::from_file(base_root.join("configs/chips/atsamd51.yaml")).unwrap();
    let peripherals: Vec<labwired_config::PeripheralConfig> =
        serde_yaml::from_str(include_str!("bw_interruption_peripherals.yaml")).unwrap();
    assert_eq!(peripherals.len(), 2);
    chip.peripherals.extend(peripherals);
    let manifest: SystemManifest = serde_yaml::from_str(
        "name: interruption\nchip: ../chips/atsamd51.yaml\nexternal_devices: []\n",
    )
    .unwrap();
    let mut bus = SystemBus::from_config(&chip, &manifest).unwrap();
    let (cpu, _) = configure_cortex_m(&mut bus);
    let mut m = Machine::new(cpu, bus);
    m.add_observer(watch.clone());
    m.load_firmware(&labwired_loader::load_elf(&path).unwrap())
        .unwrap();
    assert!(m.bus.models_flash_ops());
    assert!(symbol("_program_load_end") <= 0x77ff0);
    m.bus.flash.data[B..].fill(255);
    m.bus.flash.data[0x77ff0..B].fill(0x5a);
    let code = m.bus.flash.data[..B].to_vec();
    let input = if case == "seed" {
        None
    } else {
        let input_name = if reader { name } else { "seed" };
        let (v, id) = read_backing(&root, input_name, &source, profile);
        m.bus.flash.data[B..].copy_from_slice(&v[B..]);
        Some((v, id))
    };
    let old = if role == 1 {
        input.as_ref().unwrap().0.clone()
    } else {
        vec![]
    };
    let payload = if role == 1 { baseline(&old) } else { 0 };
    let meta = if role == 1 {
        new_meta(&old, payload)
    } else {
        [0; 8]
    };
    let mut initialized = false;
    let mut started = false;
    let mut set_call = None;
    let mut set_returned = false;
    let mut metadata_armed = false;
    let mut commands = Vec::new();
    let mut last_pending = false;
    let mut accepted = 0;
    let mut init_count = 0;
    let mut probe_count = 0;
    for step in 1..=160_000_000u64 {
        let pc = m.cpu.pc & !1;
        if pc == init {
            initialized = true;
            init_count += 1;
            assert!(m.cpu.primask && m.cpu.active_exception == 0);
        }
        if pc == probe {
            assert!(initialized);
            started = true;
            probe_count += 1;
        }
        if role == 1 && started && pc == set {
            assert!(set_call.is_none());
            set_call = Some((m.cpu.lr & !1, m.cpu.sp));
        }
        if let Some((ret, sp)) = set_call {
            if pc == ret && m.cpu.sp == sp {
                assert_eq!(m.cpu.r0, 0);
                set_returned = true;
                set_call = None;
            }
        }
        m.step()
            .expect("actual interruption guest architectural fault");
        assert!([0, 15].contains(&m.cpu.active_exception));
        let runtime = m.bus.read_u32(status).unwrap();
        assert!(
            [0, 0x100, 0x600d].contains(&runtime),
            "actual guest panic {runtime:x}"
        );
        let s = m.bus.read_u32(stage).unwrap();
        if !started {
            continue;
        }
        if s != 0 {
            assert_eq!(m.bus.read_u32(gc_stage).unwrap(), 4);
            assert_eq!(m.bus.read_u32(gc_checks).unwrap(), 60);
            assert!(watch.swaps.load(Ordering::Relaxed) >= 2);
            assert_eq!(init_count, 1);
            assert_eq!(probe_count, 1);
        }
        let pending = m.bus.has_pending_flash_op();
        if role == 1 && s == 2 {
            if name != "complete" {
                need(!set_returned, "still inside actual settings write");
            }
            if pending && !last_pending {
                let command = watch.command.load(Ordering::Relaxed);
                let ordinal = watch.commands.load(Ordering::Relaxed);
                assert!(ordinal > accepted, "actual MMIO command witness");
                accepted = ordinal;
                let address = m.bus.read_u32(0x41004014).unwrap() as usize & !15;
                let load = m.bus.read_u16(0x41004012).unwrap() & 4 != 0;
                assert_ne!(command, 0xa501, "no erase/GC in declared overwrite");
                commands.push(serde_json::json!({"step":step,"pc":pc,"command":command,"address":address,"pending":pending,"load":load,"storage_sha256":sha(&m.bus.flash.data[B..])}));
                if command == 0xa504 {
                    assert!(load);
                    assert!(set_call.is_some());
                    let cut_payload = name == "payload" && address == B + payload;
                    let cut_metadata = name == "metadata" && address == E - 32;
                    if address == E - 32 {
                        metadata_armed = true;
                        assert_eq!(
                            &m.bus.flash.data[B + payload..B + payload + 264],
                            [pattern(257, true), vec![0; 7]].concat()
                        );
                    }
                    if cut_payload || cut_metadata {
                        assert_eq!(&m.bus.flash.data[..B], code.as_slice());
                        invariants(&m.bus.flash.data, &old, payload, &meta, false);
                        let boundary = serde_json::json!({"case":case,"step":step,"pending":true,"load":true,"command":command,"address":address,"settings_returned":false,"payload_offset":payload,"new_meta":meta,"expected":1,"commands":commands});
                        export(
                            &root,
                            name,
                            &m.bus.flash.data,
                            &source,
                            profile,
                            &hash,
                            boundary,
                        );
                        println!("SAMD_INTERRUPTION_CUT engine={ENGINE} source={source} profile={profile} case={case} step={step} pending=true command=a504 address={address:#x} expected=old pid={}",std::process::id());
                        return;
                    }
                }
            }
            if name == "publish" && metadata_armed && !pending {
                assert!(set_call.is_some());
                assert_eq!(&m.bus.flash.data[..B], code.as_slice());
                invariants(&m.bus.flash.data, &old, payload, &meta, true);
                assert_eq!(
                    &m.bus.flash.data[B + payload..B + payload + 264],
                    [pattern(257, true), vec![0; 7]].concat()
                );
                let boundary = serde_json::json!({"case":case,"step":step,"pending":false,"command":0xa504,"address":E-32,"settings_returned":false,"payload_offset":payload,"new_meta":meta,"expected":2,"commands":commands});
                export(
                    &root,
                    name,
                    &m.bus.flash.data,
                    &source,
                    profile,
                    &hash,
                    boundary,
                );
                println!("SAMD_INTERRUPTION_CUT engine={ENGINE} source={source} profile={profile} case={case} step={step} pending=false committed=true settings_returned=false expected=new pid={}",std::process::id());
                return;
            }
        }
        last_pending = pending;
        if case == "seed" && s == 1 {
            assert!(!pending);
            assert_eq!(&m.bus.flash.data[..B], code.as_slice());
            baseline(&m.bus.flash.data);
            assert_eq!(watch.format.load(Ordering::Relaxed), 1);
            export(
                &root,
                "seed",
                &m.bus.flash.data,
                &source,
                profile,
                &hash,
                serde_json::json!({"case":case,"step":step,"pending":false,"expected":1}),
            );
            println!("SAMD_INTERRUPTION_SEED engine={ENGINE} source={source} profile={profile} step={step} pid={} completed=true",std::process::id());
            return;
        }
        if role == 1 && s == 3 {
            assert_eq!(
                name, "complete",
                "requested interruption must occur before return"
            );
            assert!(set_returned && !pending);
            invariants(&m.bus.flash.data, &old, payload, &meta, true);
            export(
                &root,
                name,
                &m.bus.flash.data,
                &source,
                profile,
                &hash,
                serde_json::json!({"case":case,"step":step,"pending":false,"settings_returned":true,"expected":2,"payload_offset":payload,"new_meta":meta,"commands":commands}),
            );
            println!("SAMD_INTERRUPTION_COMPLETE engine={ENGINE} source={source} profile={profile} step={step} pid={} returned=true",std::process::id());
            return;
        }
        if reader && runtime == 0x600d {
            assert_eq!(s, 4);
            let observed = m.bus.read_u32(result).unwrap();
            let (raw, id) = input.as_ref().unwrap();
            assert_eq!(id["boundary"]["expected"], observed);
            assert_eq!(
                &m.bus.flash.data[B..],
                &raw[B..],
                "recovery is read-only/no auto repair"
            );
            assert_eq!(&m.bus.flash.data[..B], code.as_slice());
            assert_eq!(watch.format.load(Ordering::Relaxed), 0);
            assert!(watch.mount.load(Ordering::Relaxed) > 0);
            assert_eq!(
                watch.commands.load(Ordering::Relaxed),
                0,
                "no NVM commands during recovery"
            );
            assert!(!pending);
            put_json(
                &root.join(format!("{case}.json")),
                &serde_json::json!({"engine":ENGINE,"source":source,"profile":profile,"case":case,"pid":std::process::id(),"producer_pid":id["pid"],"elf_sha256":hash,"step":step,"outcome":observed,"formats":0,"commands":0,"storage_unchanged":true,"storage_sha256":sha(&raw[B..]),"gc_checks":60,"startup":true,"physical_power_loss":false}),
            );
            println!("SAMD_INTERRUPTION_RECOVERY_PASS engine={ENGINE} source={source} profile={profile} case={case} step={step} outcome={observed} formats=0 commands=0 unchanged=true pid={}",std::process::id());
            return;
        }
    }
    panic!("actual interruption case did not reach its specific boundary within160000000 steps pc={:x}",m.cpu.pc);
}
