// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//! Selected public size diagnostics; no allocator policy change.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::{bus::SystemBus, system::cortex_m::configure_cortex_m, Bus, Machine};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
const ENGINE: &str = "73f8475f8e406b1ff4475bc9886e22479881d67a";
const NAMES: [&str; 7] = [
    "runtime_status",
    "public_size_stage",
    "public_size_request",
    "public_size_live",
    "public_size_pointer",
    "public_size_bytes",
    "public_size_result",
];

fn terminal(
    conversion: bool,
    status: u32,
    calls: [u32; 4],
    returns: [u32; 4],
    boundaries: u32,
    frees: u32,
    app_frees: u32,
    panic: Option<u32>,
    result: u32,
) {
    assert_eq!(calls, [6, 6, 5, 5]);
    assert_eq!(
        returns,
        if conversion {
            [6, 6, 5, 5]
        } else {
            [5, 5, 4, 4]
        }
    );
    assert_eq!((boundaries, frees, app_frees), (4, 3, 3));
    if conversion {
        assert_eq!((status, panic), (0x600d, None));
        assert!(
            result != 0,
            "selected conversion diagnostic requires the actual nonnull return"
        );
    } else {
        assert_eq!(
            (status, panic, result),
            (0xe0000016, Some(22), 0),
            "selected cap must retain actual panic 22"
        );
    }
}
#[test]
fn manufactured_terminal_requires_actual_calls_returns_and_retirement() {
    for conversion in [false, true] {
        let (status, returns, panic, result) = if conversion {
            (0x600d, [6, 6, 5, 5], None, 0x20001000)
        } else {
            (0xe0000016, [5, 5, 4, 4], Some(22), 0)
        };
        terminal(
            conversion,
            status,
            [6, 6, 5, 5],
            returns,
            4,
            3,
            3,
            panic,
            result,
        );
        for i in 0..4 {
            let mut calls = [6, 6, 5, 5];
            calls[i] -= 1;
            assert!(std::panic::catch_unwind(|| terminal(
                conversion, status, calls, returns, 4, 3, 3, panic, result
            ))
            .is_err());
            let mut bad_returns = returns;
            bad_returns[i] -= 1;
            assert!(std::panic::catch_unwind(|| terminal(
                conversion,
                status,
                [6, 6, 5, 5],
                bad_returns,
                4,
                3,
                3,
                panic,
                result
            ))
            .is_err());
        }
        for (bad_status, boundaries, frees, app_frees, bad_panic, bad_result) in [
            (0, 4, 3, 3, panic, result),
            (status, 3, 3, 3, panic, result),
            (status, 4, 2, 3, panic, result),
            (status, 4, 3, 2, panic, result),
            (status, 4, 3, 3, Some(20), result),
            (status, 4, 3, 3, panic, if conversion { 0 } else { 4 }),
        ] {
            assert!(std::panic::catch_unwind(|| terminal(
                conversion,
                bad_status,
                [6, 6, 5, 5],
                returns,
                boundaries,
                frees,
                app_frees,
                bad_panic,
                bad_result
            ))
            .is_err());
        }
    }
}
#[test]
#[ignore = "hosted source-bound actual public size diagnostics only"]
fn actual_public_size_boundary_diagnostic() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|c| c.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_PUBLIC_SIZE_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["status"], "GC_PUBLIC_SIZE_LINK_ONLY");
    let case = report["case"].as_str().unwrap();
    assert!(["public-size-cap", "public-size-conversion"].contains(&case));
    let conversion = case == "public-size-conversion";
    assert_eq!(
        report["probe_source"],
        format!("pxt-gc-runtime/probe-{case}.cpp")
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

    let apis = [
        "malloc",
        "app_alloc",
        "_ZN3pxt15gcAllocateArrayEi",
        "_ZN3pxt10gcAllocateEi",
    ]
    .map(entry);
    assert_eq!(
        apis.iter().collect::<std::collections::BTreeSet<_>>().len(),
        4
    );
    let free_entry = entry("free");
    let app_free_entry = entry("app_free");
    let raw_entry = entry("device_malloc");
    let collect_entry = entry("_ZN3pxt2gcEi");
    let panic_entry = entry("target_panic");
    let boundary = entry("authored_public_size_boundary");
    let probe_entry = entry("runtime_probe");
    let init_entry = entry("gc_static_init_entry");
    let in_gc = symbol("_ZN3pxt4inGCE");
    let block_address = symbol("_ZN3pxtL10firstBlockE");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let chip = ChipDescriptor::from_file(root.join("configs/chips/atsamd51.yaml")).unwrap();
    assert_eq!(chip.cpu_hz, 120000000);
    let manifest: SystemManifest = serde_yaml::from_str(
        "name: pxt-public-size\nchip: ../chips/atsamd51.yaml\nexternal_devices: []\n",
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

    let mut calls = [0u32; 4];
    let mut returns = [0u32; 4];
    let mut stack: Vec<(usize, u32)> = Vec::new();
    let mut raw_return = None;
    let mut free_return = None;
    let (mut boundaries, mut frees, mut app_frees, mut collections) = (0u32, 0u32, 0u32, 0u32);
    let mut observed_panic = None;
    let mut final_pointer = 0u32;
    let mut allocated_header = 0u32;
    let mut stage_pointer = 0u32;
    let mut collect_return = None;
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
            if v[1] > 0 {
                let stage = v[1];
                assert!((1..=6).contains(&stage));
                let request = [
                    0,
                    32,
                    17,
                    33,
                    131068,
                    if conversion { u32::MAX } else { 131069 },
                ][stage as usize - 1];
                if let Some(index) = apis.iter().position(|a| *a == pc) {
                    assert_eq!(index, stack.len(), "actual public size allocation chain");
                    assert_eq!(v[2], request);
                    let aligned = if request == u32::MAX {
                        4
                    } else {
                        ((request + 3) & !3u32) + 4
                    };
                    assert_eq!(
                        machine.cpu.r0,
                        if index == 3 { aligned } else { request },
                        "actual selected argument conversion and array alignment"
                    );
                    assert_eq!(
                        (machine.bus.read_u32(in_gc & !3).unwrap() >> ((in_gc & 3) * 8)) & 255,
                        0
                    );
                    calls[index] += 1;
                    assert_eq!(calls[index], if index < 2 { stage } else { stage - 1 });
                    stack.push((index, machine.cpu.lr & !1));
                    println!("PUBLIC_SIZE_ENTRY source={source} elf_sha256={hash} step={step} stage={stage} index={index} request={} actual={pc:#x}",machine.cpu.r0);
                }
                if let Some(&(index, return_pc)) = stack.last() {
                    if pc == return_pc {
                        let pointer = machine.cpu.r0;
                        println!("PUBLIC_SIZE_RETURN source={source} elf_sha256={hash} step={step} stage={stage} index={index} pointer={pointer}");
                        if stage == 1 {
                            assert_eq!(pointer, 0);
                        } else {
                            let block = machine.bus.read_u32(block_address).unwrap();
                            assert!((0x20000000..0x20030000).contains(&block));
                            let capacity = machine.bus.read_u32(u64::from(block + 4)).unwrap();
                            assert!(capacity > 131072 && capacity < 196608);
                            assert!(
                                pointer >= block + 8
                                    && pointer < block + 8 + capacity
                                    && pointer & 3 == 0
                            );
                            if index == 3 {
                                allocated_header = pointer;
                            } else {
                                assert_eq!(pointer, allocated_header + 4);
                            }
                            if index < 3 {
                                let header = machine.bus.read_u32(u64::from(pointer - 4)).unwrap();
                                let total = if stage == 6 && conversion {
                                    4
                                } else {
                                    ((request + 3) & !3) + 4
                                };
                                let expected =
                                    (if index < 2 { 0x60000000 } else { 0x40000000 }) | total;
                                println!("PUBLIC_SIZE_HEADER source={source} elf_sha256={hash} step={step} stage={stage} index={index} pointer={pointer} header={header:#x} expected={expected:#x}");
                                assert_eq!(header & !1,expected,"public size return must retain selected array ownership and capacity");
                            }
                        }
                        returns[index] += 1;
                        stack.pop();
                        if index == 0 {
                            stage_pointer = pointer;
                        }
                        if stage == 6 && index == 0 {
                            final_pointer = pointer;
                        }
                    }
                }
                if pc == raw_entry {
                    assert!(raw_return.is_none());
                    raw_return = Some(machine.cpu.lr & !1);
                    println!("PUBLIC_SIZE_RAW source={source} elf_sha256={hash} step={step} stage={stage} request={}",machine.cpu.r0);
                }
                if Some(pc) == raw_return {
                    println!("PUBLIC_SIZE_RAW_RETURN source={source} elf_sha256={hash} step={step} stage={stage} pointer={}",machine.cpu.r0);
                    assert!(
                        machine.cpu.r0 != 0,
                        "size diagnostic must not exhaust the raw heap"
                    );
                    raw_return = None;
                }
                if pc == collect_entry {
                    assert!(collect_return.is_none());
                    collect_return = Some(machine.cpu.lr & !1);
                    collections += 1;
                    println!("PUBLIC_SIZE_COLLECTION source={source} elf_sha256={hash} step={step} stage={stage} flags={}",machine.cpu.r0);
                }
                if pc == free_entry {
                    assert!((3..=5).contains(&stage) && free_return.is_none() && stack.is_empty());
                    assert_eq!(machine.cpu.r0, v[4]);
                    free_return = Some(machine.cpu.lr & !1);
                    println!("PUBLIC_SIZE_FREE source={source} elf_sha256={hash} step={step} stage={stage} pointer={}",machine.cpu.r0);
                }
                if pc == app_free_entry {
                    assert!(free_return.is_some());
                    assert_eq!(machine.cpu.r0, v[4]);
                    app_frees += 1;
                }
                if Some(pc) == free_return {
                    frees += 1;
                    assert_eq!((frees, app_frees), (stage - 2, stage - 2));
                    free_return = None;
                    println!("PUBLIC_SIZE_FREE_RETURN source={source} elf_sha256={hash} step={step} stage={stage} completed={frees}");
                }
                let terminal_seen = v[0] == 0x600d || v[0] & 0xf0000000 == 0xe0000000;
                let collected = Some(pc) == collect_return;
                if collected {
                    collect_return = None;
                }
                if pc == boundary || terminal_seen || (collected && stage >= 3) {
                    let mut payloads = vec![(v[3], 32u32, 0x69u8)];
                    if pc == boundary && stage > 2 {
                        payloads.push((v[4], v[5], (0x40 + stage) as u8));
                    }
                    for (pointer, size, fill) in payloads {
                        assert!(
                            pointer >= 0x20000004
                                && pointer.checked_add(size).unwrap() <= 0x20030000
                        );
                        let header = machine.bus.read_u32(u64::from(pointer - 4)).unwrap();
                        assert_eq!(header & !1, 0x60000000 | (((size + 3) & !3) + 4));
                        let payload: Vec<u8> = (0..size)
                            .map(|i| {
                                let a = u64::from(pointer + i);
                                ((machine.bus.read_u32(a & !3).unwrap() >> ((a & 3) * 8)) & 255)
                                    as u8
                            })
                            .collect();
                        let expected = vec![fill; size as usize];
                        let actual_sha = format!("{:x}", Sha256::digest(&payload));
                        let expected_sha = format!("{:x}", Sha256::digest(&expected));
                        println!("PUBLIC_SIZE_PAYLOAD source={source} elf_sha256={hash} step={step} stage={stage} pointer={pointer} bytes={size} expected_sha256={expected_sha} actual_sha256={actual_sha}");
                        assert_eq!(
                            payload, expected,
                            "public size diagnostic preserves every live payload byte"
                        );
                    }
                    if pc == boundary {
                        assert_eq!(v[4], stage_pointer);
                        if stage == 2 {
                            assert_eq!(v[3], stage_pointer);
                        }
                        boundaries += 1;
                        assert_eq!(boundaries, stage - 1);
                        assert_eq!(v[5], request);
                        println!("PUBLIC_SIZE_BOUNDARY source={source} elf_sha256={hash} step={step} stage={stage} boundaries={boundaries}");
                    }
                    if terminal_seen {
                        terminal(
                            conversion,
                            v[0],
                            calls,
                            returns,
                            boundaries,
                            frees,
                            app_frees,
                            observed_panic,
                            v[6],
                        );
                        assert_eq!(stage, 6);
                        assert!(raw_return.is_none() && free_return.is_none());
                        assert!(collect_return.is_none());
                        assert_eq!(v[6], final_pointer);
                        assert_eq!(stack.len(), if conversion { 0 } else { 4 });
                        for a in [0xe000ed28, 0xe000ed2c] {
                            assert_eq!(machine.bus.read_u32(a).unwrap(), 0);
                        }
                        println!("PUBLIC_SIZE_DIAGNOSTIC engine={ENGINE} source={source} elf_sha256={hash} case={case} steps={step} bound={bound} status={:#x} boundaries={boundaries} frees={frees} collections={collections} final_pointer={final_pointer} constructors={} policy_changed=false native_arcade=false performance_measured=false",v[0],entered.len());
                        return;
                    }
                }
                if pc == panic_entry {
                    println!("PUBLIC_SIZE_PANIC source={source} elf_sha256={hash} step={step} stage={stage} reason={}",machine.cpu.r0);
                    assert!(
                        !conversion && stage == 6 && machine.cpu.r0 == 22,
                        "selected cap must retain actual panic 22"
                    );
                    observed_panic = Some(machine.cpu.r0);
                }
            }
        }
        machine.step().unwrap();
    }
    println!("PUBLIC_SIZE_BOUND source={source} elf_sha256={hash} bound={bound} qualified=false");
    panic!("public size diagnostic did not reach its named terminal within {bound} steps");
}
