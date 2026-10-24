// SPDX-License-Identifier: MIT
//! Hosted actual core/SAMD driver leaf, not PXT/HF2/native-board qualification.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::peripherals::sam::usb_device::{HostReply, HostToken, Samd51UsbDeviceBanks};
use labwired_core::SimulationObserver;
use labwired_core::{
    bus::SystemBus, system::cortex_m::configure_cortex_m, Bus, Machine, Peripheral,
};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
const ENGINE: &str = "01e50371b9a7d341142399b84932b402fcac306b";
type M = Machine<labwired_core::cpu::cortex_m::CortexM>;
#[derive(Debug)]
struct Entries {
    points: [u32; 5],
    counts: [AtomicU32; 5],
}
impl SimulationObserver for Entries {
    fn on_step_start(&self, pc: u32, _opcode: u32) {
        for (i, point) in self.points.iter().enumerate() {
            if pc == *point {
                self.counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}
fn digest(raw: &[u8]) -> String {
    format!("{:x}", Sha256::digest(raw))
}
fn usb(m: &M) -> &Samd51UsbDeviceBanks {
    m.bus
        .peripherals
        .iter()
        .find(|p| p.name == "usb")
        .unwrap()
        .dev
        .as_any()
        .unwrap()
        .downcast_ref()
        .unwrap()
}
fn usb_mut(m: &mut M) -> &mut Samd51UsbDeviceBanks {
    m.bus
        .peripherals
        .iter_mut()
        .find(|p| p.name == "usb")
        .unwrap()
        .dev
        .as_any_mut()
        .unwrap()
        .downcast_mut()
        .unwrap()
}
fn tokens(interrupt: bool) -> Vec<HostToken> {
    let setup = |address, packet| HostToken::Setup { address, packet };
    let input = |address, endpoint| HostToken::In { address, endpoint };
    let status = |address| HostToken::Out {
        address,
        endpoint: 0,
        data: vec![],
    };
    let mut values = vec![
        HostToken::Reset,
        setup(0, [0x80, 6, 0, 1, 0, 0, 18, 0]),
        input(0, 0),
        status(0),
        setup(0, [0, 5, 1, 0, 0, 0, 0, 0]),
        input(0, 0),
        setup(1, [0x80, 6, 0, 2, 0, 0, 32, 0]),
        input(1, 0),
        status(1),
        setup(1, [0x80, 6, 0, 15, 0, 0, 57, 0]),
        input(1, 0),
        status(1),
        setup(1, [0xc0, 0x41, 0, 0, 7, 0, 170, 0]),
        input(1, 0),
        input(1, 0),
        input(1, 0),
        status(1),
        setup(1, [0xc0, 0x40, 1, 0, 2, 0, 255, 0]),
        input(1, 0),
        setup(1, [0x80, 0, 0, 0, 0, 0, 2, 0]),
        input(1, 0),
        status(1),
        setup(1, [0, 9, 1, 0, 0, 0, 0, 0]),
        input(1, 0),
        HostToken::Out {
            address: 1,
            endpoint: 2,
            data: (0..64).map(|i| ((i * 37 + 11) & 255) as u8).collect(),
        },
        input(1, 1),
    ];
    if !interrupt {
        values.push(input(1, 1));
    }
    values
}
fn check_reply(index: usize, reply: &HostReply, interrupt: bool, ms: &mut Vec<u8>) {
    match index {
        0 => assert_eq!(reply, &HostReply::Reset),
        1 | 4 | 6 | 9 | 12 | 17 | 19 | 22 => assert_eq!(reply, &HostReply::SetupAccepted),
        3 | 8 | 11 | 16 | 21 => assert_eq!(
            reply,
            &HostReply::OutAccepted {
                endpoint: 0,
                bytes: 0
            }
        ),
        18 => assert_eq!(
            reply,
            &HostReply::Stall {
                endpoint: 0,
                input: true
            }
        ),
        24 => assert_eq!(
            reply,
            &HostReply::OutAccepted {
                endpoint: 2,
                bytes: 64
            }
        ),
        _ => {
            let HostReply::In { endpoint, data } = reply else {
                panic!("unexpected host reply at {index}: {reply:?}")
            };
            assert_eq!(*endpoint, if index >= 25 { 1 } else { 0 });
            match index {
                2 => assert_eq!(
                    data,
                    &[18, 1, 0x10, 2, 0, 0, 0, 64, 0xeb, 3, 0x66, 0x20, 2, 0x42, 1, 2, 3, 1]
                ),
                5 | 23 | 26 => assert!(data.is_empty()),
                7 => {
                    let attr = if interrupt { 3 } else { 2 };
                    assert_eq!(
                        data,
                        &[
                            9, 2, 32, 0, 1, 1, 0, 0x80, 50, 9, 4, 0, 0, 2, 0xff, 0x42, 1, 0, 7, 5,
                            0x81, attr, 64, 0, 1, 7, 5, 2, attr, 64, 0, 1
                        ]
                    );
                }
                10 => {
                    assert_eq!(data.len(), 57);
                    assert_eq!(&data[..5], &[5, 15, 57, 0, 2]);
                    assert_eq!(&data[53..], &[170, 0, 0x41, 0]);
                }
                13 | 14 | 15 => {
                    assert_eq!(data.len(), if index == 15 { 42 } else { 64 });
                    ms.extend(data);
                    if index == 15 {
                        assert_eq!(ms.len(), 170);
                        assert_eq!(&ms[..10], &[10, 0, 0, 0, 0, 0, 3, 6, 170, 0]);
                        assert_eq!(ms[14], 0);
                        assert_eq!(&ms[22..28], b"WINUSB");
                    }
                }
                20 => assert_eq!(data, &[0, 0]),
                25 => assert_eq!(
                    data,
                    &(0..64)
                        .map(|i| ((i * 37 + 11) & 255) as u8)
                        .collect::<Vec<_>>()
                ),
                _ => panic!("unknown IN token {index}"),
            }
        }
    }
}
#[test]
#[ignore = "hosted exact source/image/model actual guest only"]
fn actual_usb_driver_leaf_cases() {
    assert_eq!(std::env::var("BW_USB_ENGINE_SOURCE").unwrap(), ENGINE);
    let source = std::env::var("BW_USB_CONSUMER_SOURCE").unwrap();
    assert!(source.len() == 40 && source.bytes().all(|b| b.is_ascii_hexdigit()));
    let folder = PathBuf::from(std::env::var("BW_USB_FIXTURE_DIR").unwrap());
    let case = std::env::var("BW_USB_CASE").unwrap();
    assert!([
        "bulk",
        "interrupt",
        "hidden-webusb",
        "missing-irq",
        "missing-in-arm",
        "bad-descriptor"
    ]
    .contains(&case.as_str()));
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("build-report.json")).unwrap()).unwrap();
    assert_eq!(report["source"], source);
    assert_eq!(report["status"], "USB1_ACTUAL_CORE_DRIVER_LEAF_LINK_ONLY");
    assert_eq!(report["core"], "2ed19d64ec3359143409a4b0a9758b4aeef3ee7d");
    assert_eq!(report["schema"], 1);
    assert_eq!(
        report["configuration"]["header_sha256"],
        "1d812a7253fb632cc30537cb34141a1394778c845918172ff5cd36e4953727d8"
    );
    assert_eq!(report["vectors"], 0x4000);
    assert_eq!(report["abi"], "cortex-m4/thumb/softfp/fpv4-sp-d16");
    assert_eq!(report["cpp_units"], 16);
    assert_eq!(report["assembly_units"], 2);
    assert_eq!(report["cases"][case.as_str()]["guest_qualified"], false);
    for name in [
        "guest_qualified",
        "usb_qualified",
        "webusb_qualified",
        "native_arcade",
        "full_board",
        "licence_clearance",
        "performance_measured",
    ] {
        assert_eq!(report[name], false);
    }
    let path = folder.join("cases").join(&case).join("usb-leaf.elf");
    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.len() > 0 && bytes.len() <= 8 * 1024 * 1024);
    let hash = digest(&bytes);
    assert_eq!(report["cases"][&case]["elf_sha256"], hash);
    let elf = goblin::elf::Elf::parse(&bytes).unwrap();
    let symbol = |name: &str| {
        let rows: Vec<_> = elf
            .syms
            .iter()
            .filter(|s| s.st_shndx != 0 && elf.strtab.get_at(s.st_name) == Some(name))
            .collect();
        assert_eq!(rows.len(), 1, "unique symbol {name}");
        rows[0].st_value
    };
    let function = |fragment: &str| {
        let mut found = std::collections::BTreeMap::<u32, u32>::new();
        for symbol in elf.syms.iter().filter(|s| {
            s.st_shndx != 0
                && elf
                    .strtab
                    .get_at(s.st_name)
                    .is_some_and(|n| n.contains(fragment))
        }) {
            let entry = found
                .entry(u32::try_from(symbol.st_value & !1).unwrap())
                .or_default();
            *entry = (*entry).max(u32::try_from(symbol.st_size).unwrap());
        }
        assert_eq!(found.len(), 1, "unique actual method address {fragment}");
        let (&start, &size) = found.iter().next().unwrap();
        assert!(size > 0);
        start..start.checked_add(size).unwrap()
    };
    let reset = symbol("reset_entry") & !1;
    let init = symbol("usb_static_init_entry") & !1;
    let wait = symbol("usb_leaf_wait") & !1;
    let write = function("writeEP");
    let core_ctor = function("CodalUSBC");
    let in_ctor = function("UsbEndpointInC");
    let out_ctor = function("UsbEndpointOutC");
    let irq0 = symbol("USB_0_Handler") & !1;
    let irq2 = symbol("USB_2_Handler") & !1;
    let stage = symbol("usb_leaf_stage");
    let status = symbol("runtime_status");
    let data = symbol("_data_start")..symbol("_data_end");
    let bss = symbol("_bss_start")..symbol("_bss_end");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut chip = ChipDescriptor::from_file(root.join("configs/chips/atsamd51.yaml")).unwrap();
    chip.peripherals.retain(|p| p.id != "usb");
    let added: Vec<labwired_config::PeripheralConfig> =
        serde_yaml::from_str(include_str!("bw_usb_leaf_peripherals.yaml")).unwrap();
    assert_eq!(added.len(), 1);
    chip.peripherals.extend(added);
    let manifest: SystemManifest = serde_yaml::from_str(
        "name: actual-usb-leaf\nchip: ../chips/atsamd51.yaml\nexternal_devices: []\n",
    )
    .unwrap();
    let mut bus = SystemBus::from_config(&chip, &manifest).unwrap();
    // Declared authored calibration input, documented defaults, not a capture.
    let mut fuse = labwired_core::memory::LinearMemory::new(4, 0x00800084);
    fuse.data.copy_from_slice(&0x1b29u32.to_le_bytes());
    bus.extra_mem.push(fuse);
    let (cpu, _) = configure_cortex_m(&mut bus);
    let mut m = Machine::new(cpu, bus);
    let entries = Arc::new(Entries {
        points: [
            core_ctor.start,
            in_ctor.start,
            out_ctor.start,
            irq0 as u32,
            irq2 as u32,
        ],
        counts: std::array::from_fn(|_| AtomicU32::new(0)),
    });
    m.add_observer(entries.clone());
    m.load_firmware(&labwired_loader::load_elf(&path).unwrap())
        .unwrap();
    assert_eq!(u64::from(m.cpu.pc & !1), reset);
    assert_eq!(m.cpu.sp, 0x20030000);
    assert_eq!(symbol("runtime_vectors"), 0x4000);
    assert_eq!(symbol("runtime_vectors_end"), 0x4400);
    for index in 0..256u64 {
        let expected = match index {
            0 => 0x20030000,
            1 => reset | 1,
            15 => symbol("runtime_systick") | 1,
            96 => irq0 | 1,
            97 => symbol("USB_1_Handler") | 1,
            98 => irq2 | 1,
            99 => symbol("USB_3_Handler") | 1,
            _ => symbol("runtime_fault") | 1,
        };
        assert_eq!(
            u64::from(m.bus.read_u32(0x4000 + 4 * index).unwrap()),
            expected
        );
    }
    let expected_data: Vec<_> = (0..data.end - data.start)
        .step_by(4)
        .map(|n| m.bus.read_u32(symbol("_data_load") + n).unwrap())
        .collect();
    for address in data.clone().step_by(4) {
        m.bus.write_u32(address, 0x5a5a5a5a).unwrap();
    }
    for address in bss.clone().step_by(4) {
        m.bus.write_u32(address, 0xa5a5a5a5).unwrap();
    }
    let requests = tokens(case == "interrupt");
    let expected_replies = if case == "interrupt" { 26 } else { 27 };
    assert_eq!(requests.len(), expected_replies);
    // Host reset arrives first. Enumeration is released after the real reset
    // handler returns with initialized endpoint buffers, not during construction.
    usb_mut(&mut m).queue_host(requests[0].clone()).unwrap();
    let mut enumeration_released = false;
    let profile = if cfg!(feature = "event-scheduler") {
        "event-scheduler"
    } else {
        "default"
    };
    println!("USB_LEAF_INPUT source={source} engine={ENGINE} elf_sha256={hash} case={case} profile={profile} tokens={expected_replies} fuse_word=0x00001b29 physical_capture=false");
    for (i, token) in requests.iter().enumerate() {
        println!("USB_LEAF_HOST_PLAN source={source} elf_sha256={hash} case={case} ordinal={i} token={token:?}");
    }
    println!("USB_LEAF_HOST_QUEUE source={source} elf_sha256={hash} case={case} step=0 ordinal=0 token={:?}", requests[0]);
    let mut memory = false;
    let mut calibrated = false;
    let mut reply_count = 0;
    let mut ms = vec![];

    let mut stable = 0u32;
    let bound = 2_000_000u64;
    for step in 1..=bound {
        if !enumeration_released && memory && calibrated && reply_count == 1 {
            let observed = entries.counts.each_ref().map(|v| v.load(Ordering::Relaxed));
            if observed == [1, 2, 2, 1, 0]
                && m.cpu.active_exception == 0
                && !m.cpu.primask
                && usb(&m).read(0x109).unwrap() & 0x10 != 0
                && usb(&m).read(0x100).unwrap() & 7 == 1
                && usb(&m).read(0x106).unwrap() & 0x40 == 0
            {
                assert_eq!(usb(&m).transactions(), 1);
                println!("USB_LEAF_HOST_RELEASE source={source} elf_sha256={hash} case={case} step={step} phase=after-real-reset-handler constructors=[1, 2, 2] irqs=[1, 0] exception=0 primask=false control_out_ready=true");
                for (ordinal, token) in requests.iter().enumerate().skip(1) {
                    usb_mut(&mut m).queue_host(token.clone()).unwrap();
                    println!("USB_LEAF_HOST_QUEUE source={source} elf_sha256={hash} case={case} step={step} ordinal={ordinal} token={token:?}");
                }
                enumeration_released = true;
            }
        }
        let pc = m.cpu.pc & !1;
        if u64::from(pc) == init {
            assert!(!memory);
            assert!(m.cpu.primask && m.cpu.active_exception == 0);
            assert_eq!(m.bus.read_u32(0xe000ed08).unwrap(), 0x4000);
            for (address, value) in data.clone().step_by(4).zip(&expected_data) {
                assert_eq!(m.bus.read_u32(address).unwrap(), *value);
            }
            for address in bss.clone().step_by(4) {
                assert_eq!(m.bus.read_u32(address).unwrap(), 0);
            }
            memory = true;
            println!("USB_LEAF_MEMORY source={source} elf_sha256={hash} case={case} step={step} actual_data_bss=true vtor=0x4000 masked=true");
        }
        if u64::from(pc) == irq0 {
            assert_eq!(m.cpu.active_exception, 96);
        }
        if u64::from(pc) == irq2 {
            assert_eq!(m.cpu.active_exception, 98);
        }
        let result = m.step();
        let observed = entries.counts.each_ref().map(|v| v.load(Ordering::Relaxed));
        let constructors = [observed[0], observed[1], observed[2]];
        let irqs = [observed[3], observed[4]];
        if let Some(reason) = usb(&m).refusal() {
            assert_eq!(case, "bad-descriptor", "unexpected model refusal: {reason}");
            assert!(reason.contains("descriptor table must be four-byte aligned primary RAM"));
            assert!(memory && calibrated && reply_count == 1 && irqs[0] == 1);
            assert_eq!(m.bus.read_u32(status).unwrap(), 0x100);
            println!("USB_LEAF_NEGATIVE source={source} engine={ENGINE} elf_sha256={hash} case={case} step={step} kind=exact-descriptor-alignment-refusal replies={reply_count} unrelated_panic=false");
            return;
        }
        result.expect("unrelated architectural/setup failure");
        let current = m.bus.read_u32(stage).unwrap();
        let outcome = m.bus.read_u32(status).unwrap();
        assert_eq!(outcome & 0xe0000000, 0, "unrelated panic");
        if current >= 2 && !calibrated {
            assert!(memory);
            assert_eq!(usb(&m).read_u16(0x28).unwrap(), 0x6259);
            calibrated = true;
            println!("USB_LEAF_READY source={source} elf_sha256={hash} case={case} step={step} calibration=0x6259 stage=2");
        }
        while let Some(reply) = usb_mut(&mut m).take_host_reply() {
            println!("USB_LEAF_HOST_REPLY source={source} elf_sha256={hash} case={case} step={step} ordinal={reply_count} reply={reply:?}");
            if case == "hidden-webusb" && reply_count == 10 {
                assert_eq!(
                    reply,
                    HostReply::Stall {
                        endpoint: 0,
                        input: true
                    }
                );
                assert!(memory && calibrated);
                assert_eq!(outcome, 0x100);
                println!("USB_LEAF_NEGATIVE source={source} engine={ENGINE} elf_sha256={hash} case={case} step={step} kind=actual-bos-interface-discovery-stall replies=11 unrelated_panic=false");
                return;
            }
            assert!(reply_count < expected_replies);
            check_reply(reply_count, &reply, case == "interrupt", &mut ms);
            reply_count += 1;
        }
        let enable = m.bus.read_u32(0xe000e108).unwrap() & (1 << 16) != 0;
        let pending = m.bus.read_u32(0xe000e208).unwrap() & (1 << 16) != 0;
        let negative = if case == "missing-irq" {
            memory
                && calibrated
                && reply_count == 1
                && !enable
                && pending
                && irqs == [0, 0]
                && constructors == [1, 0, 0]
                && current == 2
                && !m.cpu.primask
                && m.cpu.active_exception == 0
                && [wait, wait + 2].contains(&u64::from(m.cpu.pc & !1))
        } else if case == "missing-in-arm" {
            memory
                && calibrated
                && reply_count == 2
                && enable
                && irqs == [2, 0]
                && current == 2
                && outcome == 0x100
                && m.cpu.active_exception == 96
                && write.contains(&(m.cpu.pc & !1))
                && usb(&m).read(0x106).unwrap() & 0x80 == 0
                && usb(&m).read(0x107).unwrap() & 2 == 0
        } else {
            false
        };
        if negative {
            stable += 1;
        } else {
            stable = 0;
        }
        if stable == 128 {
            println!("USB_LEAF_NEGATIVE source={source} engine={ENGINE} elf_sha256={hash} case={case} step={step} kind=actual-driver-blocked-specific-gate stable=128 replies={reply_count} unrelated_panic=false");
            return;
        }
        if outcome == 0x600d {
            assert!(["bulk", "interrupt"].contains(&case.as_str()));
            assert!(memory && calibrated && enumeration_released);
            assert_eq!(current, 4);
            assert_eq!(reply_count, expected_replies);
            assert_eq!(usb(&m).transactions(), expected_replies as u64);
            assert_eq!(m.bus.read_u32(symbol("usb_leaf_echo_checks")).unwrap(), 66);
            assert_eq!(m.bus.read_u32(symbol("usb_leaf_initialised")).unwrap(), 1);
            assert_eq!(constructors, [1, 2, 2]);
            assert_eq!(irqs, [9, 1]);
            assert_eq!(m.cpu.active_exception, 0);
            assert_eq!(m.bus.read_u32(0xe000e108).unwrap() & (15 << 16), 15 << 16);
            assert_eq!(
                m.bus.read_u32(0xe000e208).unwrap() & ((1 << 16) | (1 << 18)),
                0
            );
            assert!(!m.cpu.primask);
            assert_eq!(m.bus.extra_mem[0].data, 0x1b29u32.to_le_bytes());
            assert_eq!(m.bus.read_u32(0xe000ed28).unwrap(), 0);
            assert_eq!(m.bus.read_u32(0xe000ed2c).unwrap(), 0);
            println!("USB_LEAF_FINITE_PASS source={source} engine={ENGINE} elf_sha256={hash} case={case} profile={profile} step={step} bound={bound} replies={expected_replies} constructors={constructors:?} irqs={irqs:?} echo_checks=66 webusb_core=true hf2=false native_arcade=false full_board=false physical_timing=false");
            return;
        }
    }
    panic!("actual source-built USB leaf did not reach its specific terminal within {bound} steps");
}
