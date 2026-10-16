// SPDX-License-Identifier: MIT
//! Expected premature-release ownership witness; never reads retired payload.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::{
    bus::SystemBus, peripherals::spi::SpiDevice, system::cortex_m::configure_cortex_m, Bus,
    Machine, SimulationObserver,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
struct Row {
    step: usize,
    kind: &'static str,
    value: u32,
}
#[derive(Debug, Default)]
struct Trace {
    accepts: AtomicUsize,
    wires: AtomicUsize,
    step: Mutex<usize>,
    rows: Mutex<Vec<Row>>,
}
impl Trace {
    fn record(&self, kind: &'static str, value: u32) {
        if kind == "accept" {
            self.accepts.fetch_add(1, Ordering::Relaxed);
        }
        if kind == "wire" {
            self.wires.fetch_add(1, Ordering::Relaxed);
        }
        self.rows.lock().unwrap().push(Row {
            step: *self.step.lock().unwrap(),
            kind,
            value,
        });
    }
}
impl SimulationObserver for Trace {
    fn on_memory_write(&self, address: u64, _old: u8, value: u8) {
        if address == 0x43000028 {
            self.record("accept", u32::from(value));
        }
    }
}
struct Slave(Arc<Trace>);
impl SpiDevice for Slave {
    fn cs_pin(&self) -> &str {
        "PB7"
    }
    fn cs_select(&mut self) {
        self.0.record("select", 0);
    }
    fn cs_release(&mut self) {
        self.0.record("release", 0);
    }
    fn transfer(&mut self, byte: u8) -> u8 {
        self.0.record("wire", u32::from(byte));
        0xff
    }
}

const ENGINE: &str = "73f8475f8e406b1ff4475bc9886e22479881d67a";
const BYTES: usize = 4096;
fn pattern(index: usize) -> u32 {
    ((index * 37 + 11) & 255) as u32
}
fn unique_symbol(table: &std::collections::BTreeMap<String, Vec<u64>>, name: &str) -> u64 {
    let found = table.get(name).expect("required defined ELF symbol");
    assert_eq!(found.len(), 1, "unique defined symbol {name}");
    found[0]
}
#[derive(Clone, Copy)]
struct RetiredPending {
    native_frees: usize,
    raw_frees: usize,
    caller_returned: bool,
    channel: u32,
    clock: u32,
    cs: u32,
    native: u32,
    source_end: u32,
    remaining: u16,
    before: (usize, usize, u16),
    accepts: usize,
    wires: usize,
}
impl RetiredPending {
    fn check(&self) {
        assert_eq!(
            (self.native_frees, self.raw_frees),
            (1, 1),
            "actual matching native AND raw free returns"
        );
        assert!(self.caller_returned, "actual caller free wrapper return");
        assert_ne!(
            self.channel & 2,
            0,
            "actual retired-owner descriptor remains enabled"
        );
        assert_eq!(self.clock & (1 << 6), 0, "actual clock remains gated");
        assert_eq!(self.cs & (1 << 7), 0, "actual CS remains selected");
        assert!(
            self.native >= 0x20000008
                && self.native + BYTES as u32 <= 0x2002e000
                && self.native % 8 == 0
        );
        assert_eq!(
            self.source_end,
            self.native + BYTES as u32,
            "actual descriptor still retains retired range"
        );
        assert!(self.remaining > 0 && usize::from(self.remaining) < BYTES);
        assert_eq!(self.accepts, BYTES - usize::from(self.remaining));
        assert!(self.wires <= self.accepts);
        assert_eq!(
            (self.accepts, self.wires, self.remaining),
            self.before,
            "no progress after gated pending snapshot"
        );
    }
}
fn ordering(rows: &[Row]) {
    let allowed = [
        "select",
        "accept",
        "wire",
        "buffer-free",
        "raw-free",
        "raw-free-return",
        "buffer-free-return",
        "caller-free-return",
    ];
    assert!(rows.iter().all(|r| allowed.contains(&r.kind)));
    assert!(rows.windows(2).all(|p| p[0].step <= p[1].step));
    let positions = |kind| {
        rows.iter()
            .enumerate()
            .filter(|(_, r)| r.kind == kind)
            .map(|(i, _)| i)
            .collect::<Vec<_>>()
    };
    let one = |kind| {
        let p = positions(kind);
        assert_eq!(p.len(), 1, "exact {kind}");
        p[0]
    };
    let select = one("select");
    let free = one("buffer-free");
    let accepts = positions("accept");
    let wires = positions("wire");
    assert!(!accepts.is_empty() && accepts.len() < BYTES && wires.len() <= accepts.len());
    for (i, p) in accepts.iter().enumerate() {
        assert!(select < *p && *p < free);
        assert_eq!(rows[*p].value, pattern(i));
    }
    for (i, p) in wires.iter().enumerate() {
        assert!(accepts[i] < *p && *p < free);
        assert_eq!(rows[*p].value, pattern(i));
    }
    assert!(
        free < one("raw-free")
            && one("raw-free") < one("raw-free-return")
            && one("raw-free-return") < one("buffer-free-return")
            && one("buffer-free-return") < one("caller-free-return")
    );
}
#[test]
fn manufactured_retired_pending_requires_actual_free_returns_and_pending_owner() {
    let good = RetiredPending {
        native_frees: 1,
        raw_frees: 1,
        caller_returned: true,
        channel: 2,
        clock: 0,
        cs: 0,
        native: 0x20001000,
        source_end: 0x20002000,
        remaining: 4095,
        before: (1, 1, 4095),
        accepts: 1,
        wires: 1,
    };
    good.check();
    for bad in [
        RetiredPending {
            native_frees: 0,
            ..good
        },
        RetiredPending {
            raw_frees: 0,
            ..good
        },
        RetiredPending {
            caller_returned: false,
            ..good
        },
        RetiredPending { channel: 0, ..good },
        RetiredPending { clock: 64, ..good },
        RetiredPending { cs: 128, ..good },
        RetiredPending {
            source_end: 0x20002008,
            ..good
        },
        RetiredPending {
            remaining: 0,
            ..good
        },
        RetiredPending { accepts: 2, ..good },
        RetiredPending { wires: 2, ..good },
        RetiredPending {
            before: (2, 1, 4094),
            ..good
        },
    ] {
        assert!(std::panic::catch_unwind(|| bad.check()).is_err());
    }
    let rows = [
        ("select", 0),
        ("accept", 11),
        ("wire", 11),
        ("buffer-free", 0x20001000),
        ("raw-free", 0x20000ff4),
        ("raw-free-return", 0x1000),
        ("buffer-free-return", 0x1002),
        ("caller-free-return", 0x1004),
    ]
    .into_iter()
    .enumerate()
    .map(|(step, (kind, value))| Row { step, kind, value })
    .collect::<Vec<_>>();
    ordering(&rows);
    for i in [0, 1, 3, 4, 5, 6, 7] {
        let mut bad = rows.clone();
        bad.remove(i);
        assert!(std::panic::catch_unwind(|| ordering(&bad)).is_err());
    }
    for (i, j) in [(1, 2), (3, 4), (5, 6), (6, 7)] {
        let mut bad = rows.clone();
        bad.swap(i, j);
        assert!(std::panic::catch_unwind(|| ordering(&bad)).is_err());
    }
    let mut bad = rows.clone();
    bad.push(Row {
        step: 9,
        kind: "callback",
        value: 0,
    });
    assert!(std::panic::catch_unwind(|| ordering(&bad)).is_err());
}
fn check_payload(bus: &SystemBus, public_pointer: u32, allocations: &[u32], native_frees: usize) {
    assert_ne!(public_pointer, 0);
    assert_eq!(
        bus.read_u32(u64::from(public_pointer - 4)).unwrap(),
        0x60000024
    );
    for i in 0..32 {
        assert_eq!(
            bus.read_u8(u64::from(public_pointer + i)).unwrap(),
            (i + 1) as u8
        );
    }
    if native_frees == 0 {
        let p = allocations[0];
        assert_eq!(bus.read_u32(u64::from(p - 4)).unwrap(), BYTES as u32);
        for i in 0..BYTES {
            let actual = u32::from(bus.read_u8(u64::from(p) + i as u64).unwrap());
            let expected = pattern(i);
            if actual != expected {
                println!("ALLOCATED_DMA_PREMATURE_BAD_PAYLOAD pointer={p} index={i} actual={actual} expected={expected}");
            }
            assert_eq!(actual, expected, "native live payload byte");
        }
    }
}
#[derive(Clone, Copy)]
struct Call {
    ret: u32,
    arg: u32,
}

#[test]
#[ignore = "hosted source-bound premature-release witness only; no full lifetime admission"]
fn actual_allocated_dma_premature_release() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_FIXTURE_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["status"], "GC_ALLOCATED_DMA_PREMATURE_LINK_ONLY");
    assert_eq!(report["case"], "allocated-dma-premature-release");
    assert_eq!(
        report["probe_source"],
        "pxt-gc-runtime/probe-allocated-dma-premature-release.cpp"
    );
    assert_eq!(report["startup_source"], "pxt-gc-runtime/dma-startup.S");
    assert_eq!(report["core"], "2ed19d64ec3359143409a4b0a9758b4aeef3ee7d");
    assert_eq!(
        report["framework"],
        "31abf23d118f35010fb75122e60a1eb2b6dffe9f"
    );
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
    let image = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(report["elf_sha256"], image);
    println!(
        "ALLOCATED_DMA_PREMATURE_IMAGE source={source} elf_sha256={image} case=allocated-dma-premature-release"
    );
    let elf = goblin::elf::Elf::parse(&bytes).unwrap();
    // The immutable table is indexed once. The previous linear scan at every
    // instruction is not needed to retain the same unique-definition fence.
    let mut definitions = std::collections::BTreeMap::<String, Vec<u64>>::new();
    for item in elf.syms.iter().filter(|item| item.st_shndx != 0) {
        if let Some(name) = elf.strtab.get_at(item.st_name) {
            definitions
                .entry(name.to_owned())
                .or_default()
                .push(item.st_value);
        }
    }
    let symbol = |name| unique_symbol(&definitions, name);
    let entry = |name| (symbol(name) & !1) as u32;
    let names = [
        "runtime_status",
        "gc_dma_stage",
        "gc_dma_native",
        "gc_dma_public",
        "gc_dma_replacement",
        "gc_dma_callbacks",
    ];
    let addresses = names.map(symbol);
    let bss = symbol("_bss_start")..symbol("_bss_end");
    let data = symbol("_data_start")..symbol("_data_end");
    assert!(addresses.iter().all(|a| bss.contains(a) && a % 4 == 0));
    let constructors = symbol("_init_array_start")..symbol("_init_array_end");
    assert!(
        constructors.start < constructors.end && (constructors.end - constructors.start) % 4 == 0
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut chip =
        ChipDescriptor::from_file(root.join("configs/chips/atsamd51-pybadge.yaml")).unwrap();
    assert_eq!(chip.cpu_hz, 120000000);
    chip.peripherals.push(serde_yaml::from_str("id: tx-dma\ntype: sam_dmac_sercom4_tx_byte\nbase_address: 0x4100a000\nsize: 1KB\nirq: 31\nclock: { controller: mclk, reg: AHBMASK, bit: 9 }\n").unwrap());
    let manifest = SystemManifest::from_file(root.join("configs/systems/pybadge.yaml")).unwrap();
    let mut bus = SystemBus::from_config(&chip, &manifest).unwrap();
    let trace = Arc::new(Trace::default());
    bus.attach_spi_device("sercom4", Box::new(Slave(trace.clone())))
        .unwrap();
    let (cpu, _) = configure_cortex_m(&mut bus);
    let mut machine = Machine::new(cpu, bus);
    machine.add_observer(trace.clone());
    machine
        .load_firmware(&labwired_loader::load_elf(&path).unwrap())
        .unwrap();
    assert_eq!(
        (symbol("runtime_vectors"), symbol("runtime_vectors_end")),
        (0, 1024)
    );
    assert_eq!(machine.bus.read_u32(0).unwrap(), 0x20030000);
    for vector in 1..256 {
        let word = machine.bus.read_u32(vector * 4).unwrap();
        let target = match vector {
            1 => "reset_entry",
            15 => "runtime_systick",
            47 => "runtime_dma_irq",
            79 => "runtime_spi_irq",
            _ => "runtime_fault",
        };
        assert_eq!(word & 1, 1);
        assert_eq!(word & !1, entry(target));
    }
    let expected_data = (0..data.end - data.start)
        .step_by(4)
        .map(|o| machine.bus.read_u32(symbol("_data_load") + o).unwrap())
        .collect::<Vec<_>>();
    for a in data.clone().step_by(4) {
        machine.bus.write_u32(a, 0x5a5a5a5a).unwrap();
    }
    for a in bss.clone().step_by(4) {
        machine.bus.write_u32(a, 0xa5a5a5a5).unwrap();
    }
    let ctor_entries = constructors
        .step_by(4)
        .map(|a| machine.bus.read_u32(a).unwrap() & !1)
        .collect::<Vec<_>>();
    let mut ctor_seen = std::collections::BTreeSet::new();
    let (mut initialized, mut startup) = (false, false);
    let (mut alloc_window, mut alloc_frame, mut free_frame, mut raw_free_frame) =
        (None::<Call>, None::<Call>, None::<Call>, None::<Call>);
    let (mut public_alloc, public_free, app_free) = (None::<Call>, None::<Call>, None::<Call>);
    let mut gc_frame = None::<Call>;
    let mut wrapper_frame = None::<Call>;
    let mut caller_returned = false;
    let (mut allocations, mut native_frees, mut raw_frees, public_frees, app_frees) =
        (Vec::<u32>::new(), 0, 0, 0, 0);
    let mut collections = 0;
    let mut previous_stage = 0;
    let (mut public_pointer, mut descriptor_seen) = (0, false);
    let mut frozen_counts = None;
    let mut previous_remaining = BYTES as u16;
    let bound = 12_000_000;
    let [static_init_pc, probe_pc, native_alloc_wrapper_pc, native_alloc_pc, public_alloc_pc, collect_pc, wrapper_pc, native_free_pc, raw_free_pc, public_free_pc, app_free_pc] =
        [
            "gc_static_init_entry",
            "runtime_probe",
            "gc_dma_allocate_native",
            "bw_native_alloc",
            "malloc",
            "_ZN3pxt2gcEi",
            "gc_dma_release_native",
            "bw_native_free",
            "device_free",
            "free",
            "app_free",
        ]
        .map(|name| entry(name));
    for step in 1..=bound {
        *trace.step.lock().unwrap() = step;
        let pc = machine.cpu.pc & !1;
        if pc == static_init_pc && !initialized {
            for a in bss.clone().step_by(4) {
                assert_eq!(machine.bus.read_u32(a).unwrap(), 0);
            }
            for (a, v) in data.clone().step_by(4).zip(&expected_data) {
                assert_eq!(machine.bus.read_u32(a).unwrap(), *v);
            }
            assert!(machine.cpu.primask && machine.cpu.active_exception == 0);
            initialized = true;
        }
        if ctor_entries.contains(&pc) {
            ctor_seen.insert(pc);
        }
        if pc == probe_pc && !startup {
            assert!(initialized && ctor_entries.iter().all(|p| ctor_seen.contains(p)));
            startup = true;
        }
        if startup {
            let v = addresses.map(|a| machine.bus.read_u32(a).unwrap());
            let stage = v[1];
            if pc == native_alloc_wrapper_pc {
                assert!(alloc_window.is_none());
                alloc_window = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: 0,
                });
            }
            if pc == native_alloc_pc && alloc_window.is_some() {
                assert!(alloc_frame.is_none());
                assert_eq!(machine.cpu.r0, BYTES as u32);
                alloc_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.cpu.r0,
                });
            }
            if alloc_frame.is_some_and(|c| c.ret == pc) {
                let c = alloc_frame.take().unwrap();
                let p = machine.cpu.r0;
                assert!(p >= 0x20000008 && p + BYTES as u32 <= 0x2002e000 && p % 8 == 0);
                assert_eq!(machine.bus.read_u32(u64::from(p - 4)).unwrap(), c.arg);
                allocations.push(p);
                println!(
                    "ALLOCATED_DMA_PREMATURE_ALLOC step={step} pointer={p} size={} return_pc={pc:#x}",
                    c.arg
                );
            }
            if alloc_window.is_some_and(|c| c.ret == pc) {
                assert!(alloc_frame.is_none());
                assert_eq!(Some(&machine.cpu.r0), allocations.last());
                alloc_window = None;
            }
            if pc == public_alloc_pc && machine.cpu.r0 == 32 {
                assert!(public_alloc.is_none());
                public_alloc = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: 32,
                });
            }
            if public_alloc.is_some_and(|c| c.ret == pc) {
                public_pointer = machine.cpu.r0;
                assert!(
                    public_pointer >= 0x20000004
                        && public_pointer % 4 == 0
                        && public_pointer + 32 <= 0x2002e000
                );
                assert_eq!(
                    machine.bus.read_u32(u64::from(public_pointer - 4)).unwrap(),
                    0x60000024
                );
                public_alloc = None;
            }
            if pc == collect_pc && stage == 2 {
                assert!(gc_frame.is_none());
                gc_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: 0,
                });
                check_payload(&machine.bus, public_pointer, &allocations, native_frees);
            }
            if gc_frame.is_some_and(|c| c.ret == pc) {
                gc_frame = None;
                collections += 1;
                check_payload(&machine.bus, public_pointer, &allocations, native_frees);
                println!("ALLOCATED_DMA_PREMATURE_COLLECTION step={step} ordinal={collections} native={} public={public_pointer}",allocations[0]);
            }
            if pc == wrapper_pc && stage == 3 {
                assert!(wrapper_frame.is_none() && !caller_returned);
                assert_eq!(machine.cpu.r0, allocations[0]);
                wrapper_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.cpu.r0,
                });
            }
            if wrapper_frame.is_some_and(|c| c.ret == pc) {
                wrapper_frame = None;
                caller_returned = true;
                trace.record("caller-free-return", pc);
            }
            if pc == native_free_pc && stage == 3 {
                assert!(free_frame.is_none() && native_frees == 0);
                assert_eq!(machine.cpu.active_exception, 0);
                assert!(!machine.cpu.primask);
                assert_eq!(
                    machine
                        .bus
                        .read_u32(symbol("_ZN5codal12currentFiberE"))
                        .unwrap(),
                    machine.bus.read_u32(symbol("gc_dma_main_fiber")).unwrap()
                );
                assert_eq!(machine.cpu.r0, allocations[native_frees]);
                let p = machine.cpu.r0;
                assert_eq!(
                    machine.bus.read_u32(u64::from(p - 4)).unwrap(),
                    BYTES as u32
                );
                for i in 0..BYTES {
                    let expected = if native_frees == 0 {
                        pattern(i) as u8
                    } else {
                        0xa5
                    };
                    assert_eq!(
                        machine.bus.read_u8(u64::from(p) + i as u64).unwrap(),
                        expected
                    );
                }
                free_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.bus.read_u32(u64::from(p - 8)).unwrap(),
                });
                if native_frees == 0 {
                    trace.record("buffer-free", p);
                }
            }
            if pc == raw_free_pc && free_frame.is_some() {
                assert!(raw_free_frame.is_none());
                assert_eq!(machine.cpu.r0, free_frame.unwrap().arg);
                trace.record("raw-free", machine.cpu.r0);
                raw_free_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.cpu.r0,
                });
            }
            if raw_free_frame.is_some_and(|c| c.ret == pc) {
                raw_free_frame = None;
                raw_frees += 1;
                trace.record("raw-free-return", pc);
            }
            if free_frame.is_some_and(|c| c.ret == pc) {
                assert!(raw_free_frame.is_none());
                // Defer raw return count to the actual caller terminal so the
                // omitted-raw-free mutant reaches a concrete outer API return.
                free_frame = None;
                if native_frees == 0 {
                    trace.record("buffer-free-return", pc);
                }
                native_frees += 1;
            }
            assert!(public_free.is_none() && app_free.is_none());
            assert_ne!(pc, public_free_pc, "public payload remains live at witness");
            assert_ne!(pc, app_free_pc, "no public/app free in expected witness");
            if stage != previous_stage {
                assert_eq!(stage, previous_stage + 1);
                previous_stage = stage;
                if stage == 1 {
                    assert_eq!(allocations.as_slice(), &[v[2]]);
                    assert_eq!(v[3], public_pointer);
                    check_payload(&machine.bus, public_pointer, &allocations, native_frees);
                }
                println!("ALLOCATED_DMA_PREMATURE_STAGE step={step} stage={stage} native={} public={public_pointer} frees={native_frees} collections={collections}",v[2]);
            }
            if stage >= 1 && v[2] != 0 {
                let channel = machine.bus.read_u32(0x4100a040).unwrap();
                if channel & 2 != 0 || descriptor_seen {
                    let table = machine.bus.read_u32(0x4100a034).unwrap();
                    let wb = machine.bus.read_u32(0x4100a038).unwrap();
                    assert!(
                        bss.contains(&u64::from(table))
                            && bss.contains(&u64::from(wb))
                            && table % 16 == 0
                            && wb % 16 == 0
                    );
                    assert_eq!(machine.bus.read_u16(u64::from(table)).unwrap(), 0x0409);
                    assert_eq!(machine.bus.read_u32(u64::from(table + 12)).unwrap(), 0);
                    assert_eq!(
                        machine.bus.read_u16(u64::from(table + 2)).unwrap(),
                        BYTES as u16
                    );
                    assert_eq!(
                        machine.bus.read_u32(u64::from(table + 4)).unwrap(),
                        v[2] + BYTES as u32
                    );
                    assert_eq!(
                        machine.bus.read_u32(u64::from(table + 8)).unwrap(),
                        0x43000028
                    );
                    descriptor_seen = true;
                    let count = machine.bus.read_u16(u64::from(wb + 2)).unwrap();
                    if count != 0xffff {
                        assert!(count <= previous_remaining);
                        previous_remaining = count;
                        assert_eq!(
                            machine.bus.read_u32(u64::from(wb + 4)).unwrap(),
                            v[2] + BYTES as u32
                        );
                        assert_eq!(machine.bus.read_u32(u64::from(wb + 8)).unwrap(), 0x43000028);
                        let accepts = trace.accepts.load(Ordering::Relaxed);
                        let wires = trace.wires.load(Ordering::Relaxed);
                        assert_eq!(accepts, BYTES - usize::from(count));
                        if stage == 2 {
                            assert!(count > 0 && count < (BYTES as u16));
                            let snapshot = (accepts, wires, count);
                            if let Some(expected) = frozen_counts {
                                assert_eq!(snapshot, expected);
                            } else {
                                frozen_counts = Some(snapshot);
                            }
                        }
                    }
                }
            }
            if ![0, 0x100, 0x600d].contains(&v[0]) || step == bound {
                println!("ALLOCATED_DMA_PREMATURE_FAILURE source={source} image={image} step={step} pc={pc:#x} lr={:#x} witnesses={v:?} native_frees={native_frees} raw_frees={raw_frees} collections={collections} caller_returned={caller_returned}",machine.cpu.lr);
            }
            assert!(
                [0, 0x100, 0x600d].contains(&v[0]),
                "unexpected actual guest panic {:#x}",
                v[0]
            );
            if v[0] == 0x600d {
                assert_eq!((stage, v[5]), (4, 0));
                assert_eq!(allocations.len(), 1);
                assert_eq!((public_frees, app_frees), (0, 0));
                assert!(descriptor_seen && collections >= 2 && frozen_counts.is_some());
                assert!(!machine.cpu.primask && machine.cpu.active_exception == 0);
                assert_eq!(machine.bus.read_u32(0xe000ed28).unwrap(), 0);
                assert_eq!(machine.bus.read_u32(0xe000ed2c).unwrap(), 0);
                if (native_frees, raw_frees) != (1, 1) {
                    println!("ALLOCATED_DMA_PREMATURE_WRONG_FREES step={step} native={native_frees} raw={raw_frees} expected_native=1 expected_raw=1");
                }
                let channel = machine.bus.read_u32(0x4100a040).unwrap();
                if channel & 2 == 0 {
                    println!("ALLOCATED_DMA_PREMATURE_DISABLED step={step} channel={channel} expected_enabled=true");
                }
                let table = machine.bus.read_u32(0x4100a034).unwrap();
                let state = RetiredPending {
                    native_frees,
                    raw_frees,
                    caller_returned,
                    channel,
                    clock: machine.bus.read_u32(0x40001c00 + 0x80 + 34 * 4).unwrap(),
                    cs: machine.bus.read_u32(0x41008090).unwrap(),
                    native: v[2],
                    source_end: machine.bus.read_u32(u64::from(table + 4)).unwrap(),
                    remaining: previous_remaining,
                    before: frozen_counts.unwrap(),
                    accepts: trace.accepts.load(Ordering::Relaxed),
                    wires: trace.wires.load(Ordering::Relaxed),
                };
                state.check();
                println!("ALLOCATED_DMA_PREMATURE_DESCRIPTOR table={table} writeback={} control={} count={} next={}",machine.bus.read_u32(0x4100a038).unwrap(),machine.bus.read_u16(u64::from(table)).unwrap(),machine.bus.read_u16(u64::from(table+2)).unwrap(),machine.bus.read_u32(u64::from(table+12)).unwrap());
                let rows = trace.rows.lock().unwrap();
                ordering(&rows);
                for row in rows.iter() {
                    println!(
                        "ALLOCATED_DMA_PREMATURE_ROW step={} kind={} value={}",
                        row.step, row.kind, row.value
                    );
                }
                println!("ALLOCATED_DMA_PREMATURE_WITNESS step={step} native={} source_end={} remaining={} accepts={} wires={} channel={} clock={} cs={} caller_return=true native_frees=1 raw_frees=1",state.native,state.source_end,state.remaining,state.accepts,state.wires,state.channel,state.clock,state.cs);
                println!("ALLOCATED_DMA_PREMATURE_RESULT engine={ENGINE} source={source} elf_sha256={image} steps={step} bound={bound} bytes={BYTES} collections={collections} native_allocations=1 native_frees=1 raw_frees=1 public_frees=0 app_frees=0 retired_owner_pending=true retired_payload_access=false full_lifetime=false native_arcade=false");
                return;
            }
        }
        machine
            .step()
            .expect("actual allocated DMA caller architectural fault");
        assert!(
            [0, 15].contains(&machine.cpu.active_exception),
            "no transfer IRQ while gated and genuinely pending"
        );
    }
    panic!("actual allocated DMA caller exceeded unchanged bounded budget");
}
