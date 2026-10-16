// SPDX-License-Identifier: MIT
//! Separate partial allocated-buffer cancellation/error observations; qualification pending.
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
fn ordering(rows: &[Row], kind: u32) {
    assert!([1, 2].contains(&kind));
    let terminal = if kind == 1 {
        "wrapper-return"
    } else {
        "irq-return"
    };
    let mut required = vec![
        "select",
        "release",
        "failure",
        "reentry-return",
        "failure-return",
        terminal,
        "buffer-free",
        "buffer-free-return",
    ];
    if kind == 1 {
        required.extend(["cancel", "cancel-return"]);
    } else {
        required.extend(["fault", "error"]);
    }
    assert!(rows
        .iter()
        .all(|row| row.kind == "accept" || row.kind == "wire" || required.contains(&row.kind)));
    let one = |name| {
        let positions = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.kind == name)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        assert_eq!(positions.len(), 1, "exact terminal witness {name}");
        positions[0]
    };
    for name in &required {
        one(*name);
    }
    let accepted = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.kind == "accept")
        .collect::<Vec<_>>();
    let wires = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.kind == "wire")
        .collect::<Vec<_>>();
    assert!(!accepted.is_empty() && accepted.len() < BYTES && wires.len() <= accepted.len());
    for (i, (position, row)) in accepted.iter().enumerate() {
        assert_eq!(row.value, pattern(i));
        assert!(one("select") < *position && *position < one("release"));
    }
    for (i, (position, row)) in wires.iter().enumerate() {
        assert_eq!(row.value, pattern(i));
        assert!(accepted[i].0 < *position && *position < one("release"));
    }
    assert!(
        one("release") < one("failure")
            && one("failure") < one("reentry-return")
            && one("reentry-return") < one("failure-return")
            && one("failure-return") < one(terminal)
            && one(terminal) < one("buffer-free")
            && one("buffer-free") < one("buffer-free-return")
    );
    assert_eq!(rows[one("failure")].value, if kind == 1 { 2 } else { 1 });
    assert_eq!(rows[one("reentry-return")].value as i32, -1006);
    if kind == 1 {
        assert!(
            accepted.last().unwrap().0 < one("cancel")
                && wires
                    .last()
                    .map_or(true, |(position, _)| *position < one("cancel"))
                && one("cancel") < one("release")
                && one("failure-return") < one("cancel-return")
                && one("cancel-return") < one(terminal)
        );
    } else {
        assert!(one("fault") < one("error") && one("error") < one("release"));
        assert_eq!(rows[one("fault")].value, 0x60000000);
        assert_eq!(rows[one("error")].value & 1, 1);
    }
    assert!(rows.windows(2).all(|pair| pair[0].step <= pair[1].step));
}
#[test]
fn manufactured_terminal_orders_require_actual_returns_and_partial_bytes() {
    for kind in [1, 2] {
        let mut rows = vec![Row {
            step: 1,
            kind: "select",
            value: 0,
        }];
        for i in 0..3 {
            rows.push(Row {
                step: 2,
                kind: "accept",
                value: pattern(i),
            });
            rows.push(Row {
                step: 2,
                kind: "wire",
                value: pattern(i),
            });
        }
        if kind == 2 {
            rows.push(Row {
                step: 3,
                kind: "fault",
                value: 0x60000000,
            });
        }
        rows.push(Row {
            step: 3,
            kind: if kind == 1 { "cancel" } else { "error" },
            value: if kind == 1 { 0 } else { 1 },
        });
        for name in ["release", "failure", "reentry-return", "failure-return"] {
            rows.push(Row {
                step: 4,
                kind: name,
                value: if name == "failure" {
                    if kind == 1 {
                        2
                    } else {
                        1
                    }
                } else if name == "reentry-return" {
                    (-1006i32) as u32
                } else {
                    0
                },
            });
        }
        if kind == 1 {
            rows.push(Row {
                step: 5,
                kind: "cancel-return",
                value: 0,
            });
        }
        for name in [
            if kind == 1 {
                "wrapper-return"
            } else {
                "irq-return"
            },
            "buffer-free",
            "buffer-free-return",
        ] {
            rows.push(Row {
                step: 6,
                kind: name,
                value: 0,
            });
        }
        ordering(&rows, kind);
        for i in 0..rows.len() {
            // A shorter final wire prefix remains a valid partial transfer.
            if rows[i].kind == "wire" && !rows[i + 1..].iter().any(|row| row.kind == "wire") {
                continue;
            }
            let mut missing = rows.clone();
            missing.remove(i);
            assert!(std::panic::catch_unwind(|| ordering(&missing, kind)).is_err());
        }
        for name in ["accept", "wire", "failure", "reentry-return"] {
            let mut wrong = rows.clone();
            wrong.iter_mut().find(|row| row.kind == name).unwrap().value ^= 1;
            assert!(std::panic::catch_unwind(|| ordering(&wrong, kind)).is_err());
        }
        let mut premature = rows.clone();
        let a = premature
            .iter()
            .position(|row| row.kind == "failure-return")
            .unwrap();
        let b = premature
            .iter()
            .position(|row| row.kind == "buffer-free")
            .unwrap();
        premature.swap(a, b);
        assert!(std::panic::catch_unwind(|| ordering(&premature, kind)).is_err());
        let mut duplicate = rows.clone();
        duplicate.push(rows.last().unwrap().clone());
        assert!(std::panic::catch_unwind(|| ordering(&duplicate, kind)).is_err());
    }
    let symbols = std::collections::BTreeMap::from([("ambiguous".to_owned(), vec![1, 2])]);
    for name in ["missing", "ambiguous"] {
        assert!(std::panic::catch_unwind(|| unique_symbol(&symbols, name)).is_err());
    }
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
                println!("ALLOCATED_DMA_TERMINAL_BAD_PAYLOAD pointer={p} index={i} actual={actual} expected={expected}");
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
#[ignore = "hosted source-bound allocated DMA partial terminal only; no full lifetime admission"]
fn actual_allocated_dma_terminal() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_FIXTURE_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["status"], "GC_ALLOCATED_DMA_TERMINAL_LINK_ONLY");
    let case = report["case"].as_str().unwrap();
    let kind = match case {
        "allocated-dma-cancel" => 1u32,
        "allocated-dma-error" => 2u32,
        _ => panic!("undeclared terminal case"),
    };
    let expected_result = if kind == 1 { 2 } else { 1 };

    assert_eq!(
        report["probe_source"],
        format!("pxt-gc-runtime/probe-{case}.cpp")
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
    println!("ALLOCATED_DMA_TERMINAL_IMAGE source={source} elf_sha256={image} case={case}");
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
    let current_fiber = symbol("_ZN5codal12currentFiberE");
    let main_fiber = symbol("gc_dma_main_fiber");
    let saved_writeback = symbol("gc_dma_saved_writeback");
    let mode_address = symbol("gc_dma_mode");
    let quiet_iterations = symbol("gc_dma_unmasked_iterations");
    let reentry_pc = entry("gc_dma_try_reentry");
    let cancel_pc = if kind == 1 {
        entry("_ZN19brickwright_runtime12Sercom4TxSPI6cancelEv")
    } else {
        0
    };
    let cancel_wrapper_pc = if kind == 1 { entry("gc_dma_cancel") } else { 0 };
    let begin_pc = entry("_ZN19brickwright_runtime23RuntimeSpiTransferState5beginEmmRy");
    let take_pc = entry("_ZN19brickwright_runtime23RuntimeSpiTransferState4takeEyRNS0_6ResultE");
    let mut begin_frame = None::<(Call, u32)>;
    let mut token = None::<(u32, u64)>;
    let mut take_frame = None::<Call>;
    let mut take_returned = false;
    let mut take_count = 0usize;
    let mut reentry_frame = None::<Call>;
    let mut reentry_returned = false;
    let mut cancel_frame = None::<Call>;
    let mut cancel_wrapper = None::<Call>;
    let mut cancel_returned = false;
    let mut irq_owner = None::<u32>;
    let mut unmasked_thread_steps = 0usize;
    let mut quiet_rows = None;
    let mut fault_seen = false;
    let bound = 12_000_000;
    let [static_init_pc, probe_pc, native_alloc_wrapper_pc, native_alloc_pc, public_alloc_pc, collect_pc, callback_pc, native_free_pc, raw_free_pc, public_free_pc, app_free_pc] =
        [
            "gc_static_init_entry",
            "runtime_probe",
            "gc_dma_allocate_native",
            "bw_native_alloc",
            "malloc",
            "_ZN3pxt2gcEi",
            "gc_dma_failed",
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
                    "ALLOCATED_DMA_TERMINAL_ALLOC step={step} pointer={p} size={} return_pc={pc:#x}",
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
                println!("ALLOCATED_DMA_TERMINAL_COLLECTION step={step} ordinal={collections} native={} public={public_pointer}",allocations[0]);
            }
            assert_eq!(machine.bus.read_u32(mode_address).unwrap(), kind);
            if kind == 1 && pc == cancel_wrapper_pc {
                assert!(cancel_wrapper.is_none() && !irq_returned);
                cancel_wrapper = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: 0,
                });
            }
            if kind == 1 && pc == cancel_pc {
                assert!(cancel_frame.is_none() && cancel_wrapper.is_some());
                assert_eq!(machine.cpu.active_exception, 0);
                assert!(!machine.cpu.primask);
                assert_eq!(
                    machine.bus.read_u32(current_fiber).unwrap(),
                    machine.bus.read_u32(main_fiber).unwrap()
                );
                cancel_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.cpu.r0,
                });
                trace.record("cancel", machine.cpu.r0);
            }
            if cancel_frame.is_some_and(|frame| frame.ret == pc) {
                assert!(callback_returned && take_returned && reentry_returned);
                assert_eq!(machine.cpu.r0, 0);
                cancel_frame = None;
                cancel_returned = true;
                trace.record("cancel-return", pc);
            }
            if cancel_wrapper.is_some_and(|frame| frame.ret == pc) {
                assert!(cancel_returned);
                assert_eq!(machine.cpu.r0, 0);
                assert_eq!(machine.cpu.active_exception, 0);
                assert!(!machine.cpu.primask);
                cancel_wrapper = None;
                irq_returned = true;
                trace.record("wrapper-return", pc);
            }
            if pc == begin_pc {
                assert!(begin_frame.is_none() && token.is_none());
                assert_eq!((machine.cpu.r1, machine.cpu.r2), (BYTES as u32, 0));
                assert_eq!(
                    machine.bus.read_u32(current_fiber).unwrap(),
                    machine.bus.read_u32(main_fiber).unwrap()
                );
                let pointer = machine.cpu.r3;
                assert!(pointer >= 0x20000000 && pointer + 8 <= 0x2002e000 && pointer % 8 == 0);
                begin_frame = Some((
                    Call {
                        ret: machine.cpu.lr & !1,
                        arg: pointer,
                    },
                    machine.cpu.r0,
                ));
            }
            if begin_frame.is_some_and(|(frame, _)| frame.ret == pc) {
                let (frame, state) = begin_frame.take().unwrap();
                assert_eq!(machine.cpu.r0, 1);
                let value = u64::from(machine.bus.read_u32(u64::from(frame.arg)).unwrap())
                    | (u64::from(machine.bus.read_u32(u64::from(frame.arg + 4)).unwrap()) << 32);
                assert_ne!(value, 0);
                token = Some((state, value));
                println!("ALLOCATED_DMA_TERMINAL_TOKEN_BEGIN step={step} state={state} token={value} owner={} result=1 return_pc={pc:#x}",machine.bus.read_u32(main_fiber).unwrap());
            }
            if pc == take_pc {
                assert!(take_frame.is_none() && !take_returned && take_count == 0);
                let value = u64::from(machine.cpu.r2) | (u64::from(machine.cpu.r3) << 32);
                assert_eq!(Some((machine.cpu.r0, value)), token);
                assert_eq!(machine.cpu.active_exception, if kind == 1 { 0 } else { 47 });
                let actor = machine.bus.read_u32(current_fiber).unwrap();
                assert_eq!(
                    actor,
                    if kind == 1 {
                        machine.bus.read_u32(main_fiber).unwrap()
                    } else {
                        irq_owner.unwrap()
                    }
                );
                let pointer = machine.bus.read_u32(u64::from(machine.cpu.sp)).unwrap();
                assert!(pointer >= 0x20000000 && pointer + 4 <= 0x20030000 && pointer % 4 == 0);
                take_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: pointer,
                });
                take_count += 1;
            }
            if take_frame.is_some_and(|frame| frame.ret == pc) {
                let frame = take_frame.take().unwrap();
                assert_eq!(machine.cpu.r0, 1);
                assert_eq!(
                    machine.bus.read_u32(u64::from(frame.arg)).unwrap(),
                    expected_result
                );
                take_returned = true;
                println!("ALLOCATED_DMA_TERMINAL_TOKEN_TAKE step={step} state={} token={} actor={} result=1 terminal={expected_result} return_pc={pc:#x}",token.unwrap().0,token.unwrap().1,machine.bus.read_u32(current_fiber).unwrap());
            }
            if pc == reentry_pc {
                assert!(callback_frame.is_some() && reentry_frame.is_none() && !reentry_returned);
                reentry_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: 0,
                });
            }
            if reentry_frame.is_some_and(|frame| frame.ret == pc) {
                assert_eq!(
                    machine.cpu.r0 as i32, -1006,
                    "actual busy reentry during failure delivery"
                );
                reentry_frame = None;
                reentry_returned = true;
                trace.record("reentry-return", machine.cpu.r0);
            }
            if pc == callback_pc {
                if callback_frame.is_some() || callback_returned {
                    println!("ALLOCATED_DMA_TERMINAL_WRONG_CALLBACK step={step} pc={pc:#x} returned={callback_returned} active={}", machine.cpu.active_exception);
                }
                assert!(
                    callback_frame.is_none() && !callback_returned,
                    "at most one original failure callback"
                );
                if machine.cpu.r1 != expected_result {
                    println!("ALLOCATED_DMA_TERMINAL_WRONG_FAILURE_RESULT step={step} actual={} expected={expected_result}",machine.cpu.r1);
                }
                assert_eq!(
                    machine.cpu.r1, expected_result,
                    "actual partial terminal failure result"
                );
                assert!(take_returned && take_count == 1);
                assert_eq!(machine.cpu.active_exception, if kind == 1 { 0 } else { 47 });
                assert_eq!(machine.cpu.primask, kind == 1);
                assert_eq!(
                    machine.cpu.r0,
                    machine.bus.read_u32(symbol("_ZL3spi")).unwrap()
                );
                assert_eq!(
                    machine.bus.read_u32(current_fiber).unwrap(),
                    if kind == 1 {
                        machine.bus.read_u32(main_fiber).unwrap()
                    } else {
                        irq_owner.unwrap()
                    }
                );
                assert_eq!(machine.bus.read_u32(0x4100a040).unwrap(), 0x200d00);
                assert_eq!(machine.bus.read_u32(0x43000000).unwrap(), 0x2000c);
                assert_eq!(machine.bus.read_u8(0x43000016).unwrap(), 0);
                assert_ne!(machine.bus.read_u32(0x41008090).unwrap() & (1 << 7), 0);
                callback_frame = Some(Call {
                    ret: machine.cpu.lr & !1,
                    arg: machine.cpu.r0,
                });
                trace.record("failure", machine.cpu.r1);
                check_payload(&machine.bus, public_pointer, &allocations, native_frees);
            }
            if callback_frame.is_some_and(|c| c.ret == pc) {
                callback_frame = None;
                callback_returned = true;
                assert!(reentry_returned && reentry_frame.is_none());
                assert_eq!(machine.cpu.active_exception, if kind == 1 { 0 } else { 47 });
                trace.record("failure-return", pc);
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
                        "ALLOCATED_DMA_TERMINAL_MISSING_FREE step={step} actual={native_frees} expected=2"
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
                println!("ALLOCATED_DMA_TERMINAL_STAGE step={step} stage={stage} native={} public={public_pointer} frees={native_frees} collections={collections}",v[2]);
            }
            if stage >= 1 && v[2] != 0 {
                let channel = machine.bus.read_u32(0x4100a040).unwrap();
                if channel & 2 != 0 || descriptor_seen {
                    let table = machine.bus.read_u32(0x4100a034).unwrap();
                    let current_wb = machine.bus.read_u32(0x4100a038).unwrap();
                    if kind == 2 && current_wb == 0x60000000 && !fault_seen {
                        assert_eq!(stage, 3);
                        assert_eq!(machine.bus.read_u32(0x4100a000).unwrap() & 2, 0);
                        let old_wb = machine.bus.read_u32(saved_writeback).unwrap();
                        assert!(
                            old_wb >= 0x20000000 && old_wb + 16 <= 0x2002e000 && old_wb % 16 == 0
                        );
                        let remaining = machine.bus.read_u16(u64::from(old_wb + 2)).unwrap();
                        assert!(remaining > 0 && remaining < BYTES as u16);
                        assert_eq!(
                            trace.accepts.load(Ordering::Relaxed),
                            BYTES - usize::from(remaining)
                        );
                        fault_seen = true;
                        trace.record("fault", current_wb);
                        println!("ALLOCATED_DMA_TERMINAL_FAULT step={step} address={current_wb} writeback={old_wb} remaining={remaining} accepted={} wire_bytes={}",trace.accepts.load(Ordering::Relaxed),trace.wires.load(Ordering::Relaxed));
                    }
                    let wb = if kind == 2 && current_wb == 0x60000000 {
                        machine.bus.read_u32(saved_writeback).unwrap()
                    } else {
                        current_wb
                    };
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
                        let written = BYTES - usize::from(count);
                        if kind == 2 && current_wb == 0x60000000 {
                            assert!(
                                accepts >= written && accepts <= written + 1 && accepts < BYTES
                            );
                        } else {
                            assert_eq!(accepts, written);
                        }
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
                    assert!(previous_remaining > 0 && previous_remaining < BYTES as u16);
                }
            }
            if ![0, 0x100, 0x600d].contains(&v[0]) || step == bound {
                println!("ALLOCATED_DMA_TERMINAL_FAILURE source={source} image={image} step={step} pc={pc:#x} lr={:#x} witnesses={v:?} native_frees={native_frees} raw_frees={raw_frees} collections={collections} callback_returned={callback_returned} irq_returned={irq_returned}",machine.cpu.lr);
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
                let rows = trace.rows.lock().unwrap().len();
                if let Some(expected) = quiet_rows {
                    assert_eq!(rows, expected, "no stale terminal peripheral activity");
                } else {
                    quiet_rows = Some(rows);
                }
                assert_eq!(machine.bus.read_u32(0x4100a040).unwrap() & 2, 0);
                assert!(![47, 79].contains(&machine.cpu.active_exception));
                if machine.cpu.active_exception == 0 && !machine.cpu.primask {
                    unmasked_thread_steps += 1;
                }
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
                    println!("ALLOCATED_DMA_TERMINAL_MISSING_COLLECTIONS step={step} actual={collections} expected=2");
                }
                assert!(collections >= 2, "actual collections while DMA pending");
                assert!(
                    descriptor_seen && frozen_counts.is_some() && callback_returned && irq_returned
                );
                assert_eq!(fault_seen, kind == 2);
                assert!(
                    take_returned
                        && take_count == 1
                        && reentry_returned
                        && unmasked_thread_steps >= 2048
                );
                assert_eq!(machine.bus.read_u32(quiet_iterations).unwrap(), 8192);
                assert_eq!(Some(trace.rows.lock().unwrap().len()), quiet_rows);
                assert!(machine.cpu.primask && machine.cpu.active_exception == 0);
                assert_eq!(machine.bus.read_u32(0xe000ed28).unwrap(), 0);
                assert_eq!(machine.bus.read_u32(0xe000ed2c).unwrap(), 0);
                let began = *terminal_step.get_or_insert(step);
                if step - began >= 2048 {
                    let rows = trace.rows.lock().unwrap();
                    ordering(&rows, kind);
                    for row in rows.iter() {
                        println!(
                            "ALLOCATED_DMA_TERMINAL_ROW step={} kind={} value={}",
                            row.step, row.kind, row.value
                        );
                    }
                    let accepts = trace.accepts.load(Ordering::Relaxed);
                    let wires = trace.wires.load(Ordering::Relaxed);
                    println!("ALLOCATED_DMA_TERMINAL_RESULT case={case} engine={ENGINE} source={source} elf_sha256={image} steps={step} bound={bound} bytes={BYTES} accepted={accepts} wire_bytes={wires} collections={collections} native_allocations=2 native_frees=2 raw_frees=2 public_frees=1 app_frees=1 failure_return=true terminal_return=true token_takes=1 unmasked_thread_steps={unmasked_thread_steps} post_terminal_steps=2048 full_lifetime=false native_arcade=false");
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
                assert_eq!(kind, 2, "cancellation must not invent a DMA error IRQ");
                assert!(irq_owner.is_none());
                irq_owner = Some(machine.bus.read_u32(current_fiber).unwrap());
                let flags = machine.bus.read_u8(0x4100a04e).unwrap();
                assert_eq!(flags & 1, 1);
                trace.record("error", u32::from(flags));
            }
            assert_ne!(
                exception, 79,
                "no TXC success IRQ in partial terminal cases"
            );
            if kind == 2 && callback_returned && !irq_returned && exception == 0 {
                irq_returned = true;
                trace.record("irq-return", 0);
            }
        }
        previous_exception = exception;
    }
    panic!("actual allocated DMA caller exceeded unchanged bounded budget");
}
