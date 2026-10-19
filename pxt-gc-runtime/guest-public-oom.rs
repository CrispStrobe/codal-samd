// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
//! Actual bounded public-GC exhaustion, not native OOM or oversized refusal.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::{bus::SystemBus, system::cortex_m::configure_cortex_m, Bus, Machine};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const ENGINE: &str = "79071f69cd6035a820cbdad93da7343d7d501e4d";
const NAMES: [&str; 6] = [
    "runtime_status",
    "public_oom_attempt",
    "public_oom_completed",
    "public_oom_request",
    "public_oom_checks",
    "public_oom_heap_bytes",
];

fn scratch_complete(collections: u32, calls: u32, frees: u32, active: bool, pointer: u32) {
    assert_eq!((calls, frees), (collections, collections));
    assert!(
        !active && pointer == 0,
        "every real root-scan temporary must be retired"
    );
}

#[test]
fn manufactured_root_scan_requires_exact_allocation_and_release() {
    scratch_complete(3, 3, 3, false, 0);
    for (calls, frees, active, pointer) in [
        (2, 3, false, 0),
        (3, 2, false, 0),
        (4, 3, false, 0),
        (3, 3, true, 0),
        (3, 3, false, 4),
    ] {
        assert!(
            std::panic::catch_unwind(|| scratch_complete(3, calls, frees, active, pointer))
                .is_err()
        );
    }
}

fn fatal(
    v: [u32; 6],
    capacity: u32,
    calls: [u32; 4],
    returns: [u32; 4],
    blocks: u32,
    raw: u32,
    collections: u32,
    boundaries: u32,
) {
    assert_eq!(
        v[0], 0xe0000015,
        "actual public exhaustion must retain PANIC_GC_OOM code 21"
    );
    assert!((2..32).contains(&v[2]));
    assert_eq!((v[1], v[3], v[4]), (v[2] + 1, 8192, v[2] + 3));
    assert_eq!(capacity, v[5] - 32768 - 8);
    assert_eq!(
        v[2],
        capacity / 8196,
        "all complete below-cap arrays exhaust the actual block capacity"
    );
    assert_eq!(calls, [v[1]; 4]);
    assert_eq!(returns, [v[2]; 4]);
    assert_eq!((blocks, raw, boundaries), (2, 1, v[2]));
    assert!(collections > 0);
}

#[test]
fn manufactured_public_fatal_requires_complete_capacity_and_call_witnesses() {
    let heap = 32768 + 8 + 8196 * 18 + 100;
    let good = [0xe0000015, 19, 18, 8192, 21, heap];
    fatal(good, heap - 32776, [19; 4], [18; 4], 2, 1, 5, 18);
    for i in 0..6 {
        let mut bad = good;
        bad[i] = 0;
        assert!(std::panic::catch_unwind(|| fatal(
            bad,
            heap - 32776,
            [19; 4],
            [18; 4],
            2,
            1,
            5,
            18
        ))
        .is_err());
    }
    for index in 0..4 {
        let mut calls = [19; 4];
        calls[index] -= 1;
        assert!(std::panic::catch_unwind(|| fatal(
            good,
            heap - 32776,
            calls,
            [18; 4],
            2,
            1,
            5,
            18
        ))
        .is_err());
        let mut returns = [18; 4];
        returns[index] -= 1;
        assert!(std::panic::catch_unwind(|| fatal(
            good,
            heap - 32776,
            [19; 4],
            returns,
            2,
            1,
            5,
            18
        ))
        .is_err());
    }
    for (blocks, raw, collections, boundaries) in
        [(1, 1, 5, 18), (2, 2, 5, 18), (2, 1, 0, 18), (2, 1, 5, 17)]
    {
        assert!(std::panic::catch_unwind(|| fatal(
            good,
            heap - 32776,
            [19; 4],
            [18; 4],
            blocks,
            raw,
            collections,
            boundaries
        ))
        .is_err());
    }
}

#[test]
#[ignore = "hosted source-bound actual public GC exhaustion only"]
fn actual_public_gc_exhaustion_preserves_permanent_live_arrays() {
    assert_eq!(std::env::var("BW_GC_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_GC_RUNTIME_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|c| c.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_GC_PUBLIC_OOM_DIR").unwrap());
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["status"], "GC_PUBLIC_REAL_OOM_LINK_ONLY");
    assert_eq!(report["case"], "public-gc-real-oom");
    assert_eq!(
        report["probe_source"],
        "pxt-gc-runtime/probe-public-oom.cpp"
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
    let entry = |name| (symbol(name) & !1) as u32;
    let addresses = NAMES.map(symbol);
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
    let raw_entry = entry("device_malloc");
    let raw_free = entry("device_free");
    let roots_entry = entry("_ZN3pxt15gcProcessStacksEi");
    let current_fiber = symbol("_ZN5codal12currentFiberE");
    let block_entry = entry("_ZN3pxtL17allocateBlockCoreEv");
    let collect_entry = entry("_ZN3pxt2gcEi");
    let panic_entry = entry("target_panic");
    let boundary = entry("authored_public_oom_boundary");
    let live_base = symbol("public_oom_live");
    let block_address = symbol("_ZN3pxtL10firstBlockE");
    let in_gc = symbol("_ZN3pxt4inGCE");
    let init_entry = entry("gc_static_init_entry");
    let probe_entry = entry("runtime_probe");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let chip = ChipDescriptor::from_file(root.join("configs/chips/atsamd51.yaml")).unwrap();
    assert_eq!(chip.cpu_hz, 120000000);
    let manifest: SystemManifest = serde_yaml::from_str(
        "name: pxt-public-gc-oom\nchip: ../chips/atsamd51.yaml\nexternal_devices: []\n",
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
    let (mut initialized, mut startup) = (false, false);
    let mut entered = std::collections::BTreeSet::new();
    let (mut calls, mut returns) = ([0u32; 4], [0u32; 4]);
    let (mut depth, mut return_depth) = (0usize, 0usize);
    let mut return_pcs = [0u32; 4];
    let mut pending = false;
    let (mut block_calls, mut raw_calls, mut collections, mut boundaries) =
        (0u32, 0u32, 0u32, 0u32);
    let mut raw_return = None;
    let (mut roots_return, mut scratch_return, mut scratch_free_return) = (None, None, None);
    let (mut scratch_pointer, mut scratch_calls, mut scratch_frees) = (0u32, 0u32, 0u32);
    let (mut actual_block, mut capacity, mut pending_header) = (0u32, 0u32, 0u32);
    let mut pointers = Vec::new();
    let (mut panic_observed, mut dump_observed) = (false, false);
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
                assert!(v[1] <= 32 && v[3] == 8192 && v[5] > 32768 && v[5] < 196608);
                if let Some(index) = apis.iter().position(|a| *a == pc) {
                    if index == 0 {
                        assert!(!pending);
                        pending = true;
                        depth = 0;
                        return_depth = 4;
                        assert_eq!(v[1], pointers.len() as u32 + 1);
                    }
                    assert!(
                        pending && index == depth,
                        "actual public allocation must traverse its real GC callees"
                    );
                    assert_eq!(machine.cpu.r0, if index == 3 { 8196 } else { 8192 });
                    assert_eq!(
                        (machine.bus.read_u32(in_gc & !3).unwrap() >> ((in_gc & 3) * 8)) & 255,
                        0
                    );
                    calls[index] += 1;
                    return_pcs[index] = machine.cpu.lr & !1;
                    depth += 1;
                    println!("PUBLIC_GC_OOM_ENTRY source={source} elf_sha256={hash} step={step} attempt={} index={index} request={} actual={pc:#x}", v[1], machine.cpu.r0);
                }
                if pending && depth == 4 && return_depth > 0 && pc == return_pcs[return_depth - 1] {
                    let index = return_depth - 1;
                    let pointer = machine.cpu.r0;
                    println!("PUBLIC_GC_OOM_RETURN source={source} elf_sha256={hash} step={step} attempt={} index={index} pointer={pointer}", v[1]);
                    assert!(pointer >= actual_block + 8 && pointer < actual_block + 8 + capacity);
                    if index == 3 {
                        pending_header = pointer;
                    } else {
                        assert_eq!(pointer, pending_header + 4);
                    }
                    returns[index] += 1;
                    return_depth -= 1;
                    if index == 0 {
                        let header = machine.bus.read_u32(u64::from(pointer - 4)).unwrap();
                        println!("PUBLIC_GC_OOM_OWNERSHIP source={source} elf_sha256={hash} step={step} attempt={} pointer={pointer} header={header:#x} expected=0x60002004", v[1]);
                        assert_eq!(
                            header & !1,
                            0x60002004,
                            "public GC malloc must retain permanent array ownership"
                        );
                        assert_eq!(pointer & 3, 0);
                        if let Some(previous) = pointers.last() {
                            assert_eq!(pointer, *previous + 8196);
                        }
                        pointers.push(pointer);
                        pending = false;
                    }
                }
                if pc == roots_entry {
                    scratch_complete(
                        collections - 1,
                        scratch_calls,
                        scratch_frees,
                        roots_return.is_some(),
                        scratch_pointer,
                    );
                    assert_eq!(machine.bus.read_u32(current_fiber).unwrap(), 0);
                    assert!(collections > 0 && [0, 2].contains(&machine.cpu.r0));
                    roots_return = Some(machine.cpu.lr & !1);
                    println!("PUBLIC_GC_OOM_ROOT_SCAN source={source} elf_sha256={hash} step={step} attempt={} collection={collections} current_fiber=0 actual={pc:#x}", v[1]);
                }
                if pc == raw_entry && roots_return.is_some() {
                    assert_eq!(
                        machine.cpu.r0, 1,
                        "actual scheduler-absent root scan allocates one byte"
                    );
                    assert!(
                        scratch_return.is_none()
                            && scratch_free_return.is_none()
                            && scratch_pointer == 0
                    );
                    scratch_calls += 1;
                    assert_eq!(scratch_calls, collections);
                    scratch_return = Some(machine.cpu.lr & !1);
                    println!("PUBLIC_GC_OOM_SCRATCH source={source} elf_sha256={hash} step={step} attempt={} collection={collections} request=1 actual={pc:#x}", v[1]);
                } else if pc == raw_entry {
                    scratch_complete(
                        collections,
                        scratch_calls,
                        scratch_frees,
                        false,
                        scratch_pointer,
                    );
                    raw_calls += 1;
                    assert_eq!(
                        raw_calls, 1,
                        "public GC exhaustion must not be a second raw-heap allocation failure"
                    );
                    assert_eq!(machine.cpu.r0, v[5] - 32768);
                    assert_eq!(v[1], 1);
                    raw_return = Some(machine.cpu.lr & !1);
                    println!("PUBLIC_GC_OOM_RAW source={source} elf_sha256={hash} step={step} attempt=1 request={} actual={pc:#x}", machine.cpu.r0);
                }
                if Some(pc) == scratch_return {
                    scratch_pointer = machine.cpu.r0;
                    assert!(
                        scratch_pointer >= 0x20000000
                            && scratch_pointer < 0x2002e000
                            && scratch_pointer & 3 == 0
                    );
                    assert!(
                        actual_block == 0
                            || scratch_pointer < actual_block
                            || scratch_pointer >= actual_block + 8 + capacity
                    );
                    scratch_return = None;
                    println!("PUBLIC_GC_OOM_SCRATCH_RETURN source={source} elf_sha256={hash} step={step} attempt={} collection={collections} pointer={scratch_pointer}", v[1]);
                }
                if pc == raw_free {
                    assert!(
                        roots_return.is_some()
                            && scratch_return.is_none()
                            && scratch_free_return.is_none()
                    );
                    assert_ne!(scratch_pointer, 0);
                    assert_eq!(machine.cpu.r0, scratch_pointer);
                    scratch_free_return = Some(machine.cpu.lr & !1);
                    println!("PUBLIC_GC_OOM_SCRATCH_FREE source={source} elf_sha256={hash} step={step} attempt={} collection={collections} pointer={scratch_pointer} actual={pc:#x}", v[1]);
                }
                if Some(pc) == scratch_free_return {
                    scratch_frees += 1;
                    assert_eq!(scratch_frees, collections);
                    println!("PUBLIC_GC_OOM_SCRATCH_RELEASED source={source} elf_sha256={hash} step={step} attempt={} collection={collections} pointer={scratch_pointer}", v[1]);
                    scratch_pointer = 0;
                    scratch_free_return = None;
                }
                if Some(pc) == roots_return {
                    scratch_complete(
                        collections,
                        scratch_calls,
                        scratch_frees,
                        false,
                        scratch_pointer,
                    );
                    assert!(scratch_return.is_none() && scratch_free_return.is_none());
                    roots_return = None;
                    println!("PUBLIC_GC_OOM_ROOT_RETURN source={source} elf_sha256={hash} step={step} attempt={} collection={collections}", v[1]);
                }
                if Some(pc) == raw_return {
                    actual_block = machine.cpu.r0;
                    capacity = v[5] - 32768 - 8;
                    assert!(
                        actual_block >= 0x20000000 && actual_block + 8 + capacity <= 0x2002e000
                    );
                    println!("PUBLIC_GC_OOM_RAW_RETURN source={source} elf_sha256={hash} step={step} pointer={actual_block} capacity={capacity} heap_bytes={}", v[5]);
                    raw_return = None;
                }
                if pc == block_entry {
                    block_calls += 1;
                    let block = machine.bus.read_u32(block_address).unwrap();
                    println!("PUBLIC_GC_OOM_BLOCK source={source} elf_sha256={hash} step={step} attempt={} number={block_calls} existing={block} actual={pc:#x}", v[1]);
                    assert_eq!(block_calls, if block == 0 { 1 } else { 2 });
                    if block != 0 {
                        assert_eq!(block, actual_block);
                    }
                }
                if pc == collect_entry {
                    collections += 1;
                    let flags = machine.cpu.r0;
                    assert!([0, 2].contains(&flags));
                    if flags == 2 {
                        dump_observed = true;
                    }
                    println!("PUBLIC_GC_OOM_COLLECT source={source} elf_sha256={hash} step={step} attempt={} flags={flags} number={collections} actual={pc:#x}", v[1]);
                }
                if pc == panic_entry {
                    scratch_complete(
                        collections,
                        scratch_calls,
                        scratch_frees,
                        roots_return.is_some(),
                        scratch_pointer,
                    );
                    println!("PUBLIC_GC_OOM_PANIC source={source} elf_sha256={hash} step={step} attempt={} completed={} reason={} block_calls={block_calls} raw_calls={raw_calls} collections={collections} actual={pc:#x}", v[1], v[2], machine.cpu.r0);
                    assert_eq!(
                        machine.cpu.r0, 21,
                        "actual public exhaustion must retain PANIC_GC_OOM code 21"
                    );
                    assert!(pending && depth == 4 && return_depth == 4 && dump_observed);
                    panic_observed = true;
                }
                if pc == boundary || v[0] & 0xf0000000 == 0xe0000000 {
                    assert_eq!(v[2] as usize, pointers.len());
                    let block = machine.bus.read_u32(block_address).unwrap();
                    assert_eq!(block, actual_block);
                    assert_eq!(machine.bus.read_u32(u64::from(block)).unwrap(), 0);
                    assert_eq!(
                        machine.bus.read_u32(u64::from(block + 4)).unwrap(),
                        capacity
                    );
                    for slot in 0..32 {
                        let pointer = machine.bus.read_u32(live_base + slot as u64 * 4).unwrap();
                        assert_eq!(pointer, pointers.get(slot).copied().unwrap_or(0));
                        if pointer == 0 {
                            continue;
                        }
                        let header = machine.bus.read_u32(u64::from(pointer - 4)).unwrap();
                        assert_eq!(header & !1, 0x60002004);
                        let payload: Vec<_> = (0..8192)
                            .map(|i| {
                                let a = u64::from(pointer + i);
                                ((machine.bus.read_u32(a & !3).unwrap() >> ((a & 3) * 8)) & 255)
                                    as u8
                            })
                            .collect();
                        let expected = vec![0x30 + slot as u8; 8192];
                        let expected_sha = format!("{:x}", Sha256::digest(&expected));
                        let actual_sha = format!("{:x}", Sha256::digest(&payload));
                        println!("PUBLIC_GC_OOM_PAYLOAD source={source} elf_sha256={hash} step={step} attempt={} completed={} slot={slot} pointer={pointer} bytes=8192 header={header:#x} expected_sha256={expected_sha} actual_sha256={actual_sha}", v[1], v[2]);
                        assert_eq!(
                            payload, expected,
                            "public fatal OOM must preserve every live payload byte"
                        );
                    }
                    if pc == boundary {
                        scratch_complete(
                            collections,
                            scratch_calls,
                            scratch_frees,
                            roots_return.is_some(),
                            scratch_pointer,
                        );
                        boundaries += 1;
                        assert!(!pending && boundaries == v[2] && v[1] == v[2] && v[4] == v[2] + 3);
                        assert_eq!(
                            (machine.bus.read_u32(in_gc & !3).unwrap() >> ((in_gc & 3) * 8)) & 255,
                            0
                        );
                        println!("PUBLIC_GC_OOM_BOUNDARY source={source} elf_sha256={hash} step={step} completed={} checks={} capacity={capacity}", v[2], v[4]);
                    }
                }
                if v[0] & 0xf0000000 == 0xe0000000 {
                    fatal(
                        v,
                        capacity,
                        calls,
                        returns,
                        block_calls,
                        raw_calls,
                        collections,
                        boundaries,
                    );
                    assert!(panic_observed && raw_return.is_none());
                    scratch_complete(
                        collections,
                        scratch_calls,
                        scratch_frees,
                        roots_return.is_some(),
                        scratch_pointer,
                    );
                    assert_eq!(
                        (machine.bus.read_u32(in_gc & !3).unwrap() >> ((in_gc & 3) * 8)) & 255,
                        1
                    );
                    for a in [0xe000ed28, 0xe000ed2c] {
                        assert_eq!(machine.bus.read_u32(a).unwrap(), 0);
                    }
                    println!("PUBLIC_GC_OOM_CALLER engine={ENGINE} source={source} elf_sha256={hash} steps={step} bound={bound} status=0xe0000015 attempted={} completed={} request=8192 checks={} capacity={capacity} block_calls={block_calls} raw_calls={raw_calls} collections={collections} scratch_calls={scratch_calls} scratch_frees={scratch_frees} boundaries={boundaries} constructors={} public_gc_fatal_oom_only=true oversized_qualified=false allocated_dma_qualified=false native_arcade=false performance_measured=false", v[1], v[2], v[4], entered.len());
                    return;
                }
            }
        }
        machine.step().unwrap();
    }
    println!("PUBLIC_GC_OOM_BOUND source={source} elf_sha256={hash} bound={bound} qualified=false");
    panic!("actual public GC OOM did not reach expected fatal terminal within {bound} steps");
}
