// SPDX-License-Identifier: MIT
//! Separate async completion plus bounded unmasked post-release observations.
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
    let found = table
        .get(name)
        .unwrap_or_else(|| panic!("required defined ELF symbol {name}"));
    assert_eq!(found.len(), 1, "unique defined symbol {name}");
    found[0]
}
fn ordering(rows: &[Row]) {
    assert!(rows.iter().all(|r| [
        "select",
        "accept",
        "wire",
        "dma",
        "txc",
        "release",
        "callback",
        "callback-return",
        "irq-return",
        "buffer-free",
        "buffer-free-return"
    ]
    .contains(&r.kind)));
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
    let dma = one("dma");
    let txc = one("txc");
    let release = one("release");
    let callback = one("callback");
    let returned = one("callback-return");
    let retired = one("irq-return");
    let free = one("buffer-free");
    let freed = one("buffer-free-return");
    let accept = positions("accept");
    let wire = positions("wire");
    assert_eq!((accept.len(), wire.len()), (BYTES, BYTES));
    for i in 0..BYTES {
        assert_eq!(
            (rows[accept[i]].value, rows[wire[i]].value),
            (pattern(i), pattern(i))
        );
        assert!(select < accept[i] && accept[i] < wire[i] && wire[i] < txc);
    }
    assert!(accept[BYTES - 1] < dma && dma < wire[BYTES - 1] && wire[BYTES - 1] < txc);
    assert!(
        txc < release
            && release < callback
            && callback < returned
            && returned < retired
            && retired < free
            && free < freed
    );
    assert!(rows.windows(2).all(|p| p[0].step <= p[1].step));
}
#[test]
fn manufactured_allocated_dma_order_requires_real_returns_and_bytes() {
    let symbols = std::collections::BTreeMap::from([
        ("defined".to_owned(), vec![0x1001]),
        ("ambiguous".to_owned(), vec![0x1001, 0x2001]),
    ]);
    assert_eq!(unique_symbol(&symbols, "defined"), 0x1001);
    for name in ["missing", "ambiguous"] {
        assert!(std::panic::catch_unwind(|| unique_symbol(&symbols, name)).is_err());
    }
    let mut rows = Vec::new();
    let mut add = |kind, value| {
        rows.push(Row {
            step: 0,
            kind,
            value,
        })
    };
    add("select", 0);
    for i in 0..BYTES {
        add("accept", pattern(i));
        if i == BYTES - 1 {
            add("dma", 0);
        }
        add("wire", pattern(i));
    }
    for kind in [
        "txc",
        "release",
        "callback",
        "callback-return",
        "irq-return",
        "buffer-free",
        "buffer-free-return",
    ] {
        add(kind, 0);
    }
    ordering(&rows);
    for kind in [
        "select",
        "dma",
        "txc",
        "release",
        "callback",
        "callback-return",
        "irq-return",
        "buffer-free",
        "buffer-free-return",
        "accept",
        "wire",
    ] {
        let mut bad = rows.clone();
        bad.remove(bad.iter().position(|r| r.kind == kind).unwrap());
        assert!(std::panic::catch_unwind(|| ordering(&bad)).is_err());
    }
    for (kind, ordinal) in [("accept", 0), ("wire", BYTES / 2), ("wire", BYTES - 1)] {
        let mut bad = rows.clone();
        let index = bad
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == kind)
            .nth(ordinal)
            .unwrap()
            .0;
        bad[index].value ^= 1;
        assert!(std::panic::catch_unwind(|| ordering(&bad)).is_err());
    }
    let mut unknown = rows.clone();
    unknown.push(Row {
        step: 0,
        kind: "unadmitted",
        value: 0,
    });
    assert!(std::panic::catch_unwind(|| ordering(&unknown)).is_err());
    let mut bad = rows.clone();
    let a = bad.iter().position(|r| r.kind == "irq-return").unwrap();
    let b = bad.iter().position(|r| r.kind == "buffer-free").unwrap();
    bad.swap(a, b);
    assert!(std::panic::catch_unwind(|| ordering(&bad)).is_err());
}
#[derive(Default)]
struct QuietWindow {
    rows: Option<usize>,
    thread_steps: usize,
}
impl QuietWindow {
    fn observe(
        &mut self,
        primask: bool,
        exception: u32,
        rows: usize,
        channel: u32,
    ) -> Result<(), &'static str> {
        if channel & 2 != 0 {
            return Err("enabled-dma");
        }
        if ![0, 15].contains(&exception) {
            return Err("stale-transfer-irq");
        }
        if exception == 0 && primask {
            return Err("masked-thread");
        }
        if let Some(expected) = self.rows {
            if rows != expected {
                return Err("new-peripheral-row");
            }
        } else {
            self.rows = Some(rows);
        }
        if exception == 0 {
            self.thread_steps += 1;
        }
        Ok(())
    }
    fn finish(&self, iterations: u32, rows: usize) {
        assert!(
            self.thread_steps >= 2048,
            "actual unmasked post-release thread interval"
        );
        assert_eq!(iterations, 8192, "actual completed quiet loop");
        assert_eq!(
            self.rows,
            Some(rows),
            "no stale post-release peripheral activity"
        );
    }
}
#[test]
fn manufactured_quiet_window_requires_unmasked_steps_and_no_stale_activity() {
    let mut q = QuietWindow::default();
    for _ in 0..2048 {
        q.observe(false, 0, 7, 0).unwrap();
    }
    q.observe(true, 15, 7, 0).unwrap(); // An interrupt is not a thread step.
    assert_eq!(q.thread_steps, 2048);
    q.finish(8192, 7);
    for (mask, exception, rows, channel) in [
        (true, 0, 7, 0),
        (false, 47, 7, 0),
        (false, 79, 7, 0),
        (false, 0, 8, 0),
        (false, 0, 7, 2),
    ] {
        assert!(q.observe(mask, exception, rows, channel).is_err());
    }
    for (iterations, rows) in [(8191, 7), (8192, 8)] {
        assert!(std::panic::catch_unwind(|| q.finish(iterations, rows)).is_err());
    }
    let empty = QuietWindow::default();
    assert!(std::panic::catch_unwind(|| empty.finish(8192, 7)).is_err());
    let mut short = QuietWindow::default();
    for _ in 0..2047 {
        short.observe(false, 0, 7, 0).unwrap();
    }
    assert!(std::panic::catch_unwind(|| short.finish(8192, 7)).is_err());
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
                println!("ALLOCATED_DMA_ASYNC_UNMASKED_BAD_PAYLOAD pointer={p} index={i} actual={actual} expected={expected}");
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
#[ignore = "hosted source-bound allocated DMA completion only; no full lifetime admission"]
fn actual_allocated_dma_async_unmasked() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_FIXTURE_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(
        report["status"],
        "GC_ALLOCATED_DMA_ASYNC_UNMASKED_LINK_ONLY"
    );
    assert_eq!(report["case"], "allocated-dma-async-unmasked");
    assert_eq!(
        report["probe_source"],
        "pxt-gc-runtime/probe-allocated-dma-async-unmasked.cpp"
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
        "ALLOCATED_DMA_ASYNC_UNMASKED_IMAGE source={source} elf_sha256={image} case=allocated-dma-async-unmasked"
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
    let (mut public_alloc, mut public_free, mut app_free) =
        (None::<Call>, None::<Call>, None::<Call>);
    let (mut gc_frame, mut callback_frame) = (None::<Call>, None::<Call>);
    let (mut allocations, mut native_frees, mut raw_frees, mut public_frees, mut app_frees) =
        (Vec::<u32>::new(), 0, 0, 0, 0);
    let (mut collections, mut callback_returned, mut irq_returned) = (0, false, false);
    let (mut previous_stage, mut previous_exception, mut terminal_step) = (0, 0, None);
    let (mut public_pointer, mut descriptor_seen) = (0, false);
    let mut frozen_counts = None;
    let mut previous_remaining = BYTES as u16;
    let mut quiet = QuietWindow::default();
    let iterations_address = symbol("gc_dma_unmasked_iterations");
    let bound = 12_000_000;
    let [static_init_pc, probe_pc, native_alloc_wrapper_pc, native_alloc_pc, public_alloc_pc, collect_pc, callback_pc, native_free_pc, raw_free_pc, public_free_pc, app_free_pc] =
        [
            "gc_static_init_entry",
            "runtime_probe",
            "gc_dma_allocate_native",
            "bw_native_alloc",
            "malloc",
            "_ZN3pxt2gcEi",
            "gc_dma_done",
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
                    "ALLOCATED_DMA_ASYNC_UNMASKED_ALLOC step={step} pointer={p} size={} return_pc={pc:#x}",
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
                println!("ALLOCATED_DMA_ASYNC_UNMASKED_COLLECTION step={step} ordinal={collections} native={} public={public_pointer}",allocations[0]);
            }
            if pc == callback_pc {
                if callback_frame.is_some() || callback_returned {
                    println!("ALLOCATED_DMA_ASYNC_UNMASKED_WRONG_CALLBACK step={step} pc={pc:#x} returned={callback_returned} active={}", machine.cpu.active_exception);
                }
                assert!(
                    callback_frame.is_none() && !callback_returned,
                    "at most one original callback"
                );
                assert_eq!(machine.cpu.active_exception, 79);
                callback_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.cpu.r0,
                });
                trace.record("callback", machine.cpu.r0);
                check_payload(&machine.bus, public_pointer, &allocations, native_frees);
            }
            if callback_frame.is_some_and(|c| c.ret == pc) {
                callback_frame = None;
                callback_returned = true;
                assert_eq!(machine.cpu.active_exception, 79);
                trace.record("callback-return", pc);
            }
            if pc == native_free_pc && stage == 4 {
                assert!(
                    free_frame.is_none() && callback_returned && irq_returned && native_frees < 2
                );
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
                raw_free_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.cpu.r0,
                });
            }
            if raw_free_frame.is_some_and(|c| c.ret == pc) {
                raw_free_frame = None;
                raw_frees += 1;
            }
            if free_frame.is_some_and(|c| c.ret == pc) {
                assert!(raw_free_frame.is_none());
                assert_eq!(raw_frees, native_frees + 1);
                free_frame = None;
                if native_frees == 0 {
                    trace.record("buffer-free-return", pc);
                }
                native_frees += 1;
            }
            if pc == public_free_pc {
                assert_eq!(machine.cpu.r0, public_pointer);
                if native_frees != 2 {
                    println!(
                        "ALLOCATED_DMA_ASYNC_UNMASKED_MISSING_FREE step={step} actual={native_frees} expected=2"
                    );
                }
                assert_eq!(
                    native_frees, 2,
                    "actual two native frees before public release"
                );
                public_free = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.cpu.r0,
                });
                check_payload(&machine.bus, public_pointer, &allocations, native_frees);
            }
            if pc == app_free_pc && public_free.is_some() {
                assert_eq!(machine.cpu.r0, public_pointer);
                app_free = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.cpu.r0,
                });
            }
            if app_free.is_some_and(|c| c.ret == pc) {
                app_free = None;
                app_frees += 1;
            }
            if public_free.is_some_and(|c| c.ret == pc) {
                public_free = None;
                public_frees += 1;
                assert_eq!(app_frees, public_frees);
            }
            if stage != previous_stage {
                assert_eq!(stage, previous_stage + 1);
                previous_stage = stage;
                if stage == 1 {
                    assert_eq!(allocations.as_slice(), &[v[2]]);
                    assert_eq!(v[3], public_pointer);
                    check_payload(&machine.bus, public_pointer, &allocations, native_frees);
                }
                println!("ALLOCATED_DMA_ASYNC_UNMASKED_STAGE step={step} stage={stage} native={} public={public_pointer} frees={native_frees} collections={collections}",v[2]);
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
                if native_frees > 0 {
                    assert_eq!(channel & 2, 0);
                    assert_eq!(previous_remaining, 0);
                }
            }
            if ![0, 0x100, 0x600d].contains(&v[0]) || step == bound {
                println!("ALLOCATED_DMA_ASYNC_UNMASKED_FAILURE source={source} image={image} step={step} pc={pc:#x} lr={:#x} witnesses={v:?} native_frees={native_frees} raw_frees={raw_frees} collections={collections} callback_returned={callback_returned} irq_returned={irq_returned}",machine.cpu.lr);
            }
            assert!(
                [0, 0x100, 0x600d].contains(&v[0]),
                "unexpected actual guest panic {:#x}",
                v[0]
            );
            if stage == 5 {
                assert_eq!(
                    (native_frees, raw_frees, public_frees, app_frees),
                    (2, 2, 1, 1)
                );
                let observed = quiet.observe(
                    machine.cpu.primask,
                    machine.cpu.active_exception,
                    trace.rows.lock().unwrap().len(),
                    machine.bus.read_u32(0x4100a040).unwrap(),
                );
                if let Err(reason) = observed {
                    println!("ALLOCATED_DMA_ASYNC_UNMASKED_BAD_QUIET step={step} active={} primask={} reason={reason}", machine.cpu.active_exception, machine.cpu.primask);
                }
                observed.expect("actual admitted quiet phase");
            }
            if stage >= 6 {
                if quiet.thread_steps < 2048 {
                    println!("ALLOCATED_DMA_ASYNC_UNMASKED_SHORT_QUIET step={step} actual={} expected_min=2048", quiet.thread_steps);
                }
                quiet.finish(
                    machine.bus.read_u32(iterations_address).unwrap(),
                    trace.rows.lock().unwrap().len(),
                );
            }
            if v[0] == 0x600d {
                assert_eq!((v[1], v[5]), (6, 1));
                assert_eq!(
                    (
                        allocations.len(),
                        native_frees,
                        raw_frees,
                        public_frees,
                        app_frees
                    ),
                    (2, 2, 2, 1, 1)
                );
                if collections < 2 {
                    println!("ALLOCATED_DMA_ASYNC_UNMASKED_MISSING_COLLECTIONS step={step} actual={collections} expected=2");
                }
                assert!(collections >= 2, "actual collections while DMA pending");
                assert!(
                    descriptor_seen && frozen_counts.is_some() && callback_returned && irq_returned
                );
                assert!(machine.cpu.primask && machine.cpu.active_exception == 0);
                assert_eq!(machine.bus.read_u32(0xe000ed28).unwrap(), 0);
                assert_eq!(machine.bus.read_u32(0xe000ed2c).unwrap(), 0);
                let began = *terminal_step.get_or_insert(step);
                if step - began >= 2048 {
                    let rows = trace.rows.lock().unwrap();
                    ordering(&rows);
                    for row in rows.iter() {
                        println!(
                            "ALLOCATED_DMA_ASYNC_UNMASKED_ROW step={} kind={} value={}",
                            row.step, row.kind, row.value
                        );
                    }
                    println!("ALLOCATED_DMA_ASYNC_UNMASKED_RESULT engine={ENGINE} source={source} elf_sha256={image} steps={step} bound={bound} bytes={BYTES} collections={collections} native_allocations=2 native_frees=2 raw_frees=2 public_frees=1 app_frees=1 callback_return=true irq_return=true unmasked_thread_steps={} unmasked_iterations=8192 post_terminal_steps=2048 full_lifetime=false native_arcade=false", quiet.thread_steps);
                    return;
                }
            }
        }
        machine
            .step()
            .expect("actual allocated DMA caller architectural fault");
        let exception = machine.cpu.active_exception;
        assert!([0, 15, 47, 79].contains(&exception));
        if exception != previous_exception {
            if exception == 47 {
                assert_eq!(machine.bus.read_u8(0x4100a04e).unwrap() & 3, 2);
                trace.record("dma", 0);
            }
            if exception == 79 {
                assert_ne!(machine.bus.read_u8(0x43000018).unwrap() & 2, 0);
                trace.record("txc", 0);
            }
            if callback_returned && !irq_returned && exception == 0 {
                irq_returned = true;
                trace.record("irq-return", 0);
            }
        }
        previous_exception = exception;
    }
    panic!("actual allocated DMA caller exceeded unchanged bounded budget");
}
