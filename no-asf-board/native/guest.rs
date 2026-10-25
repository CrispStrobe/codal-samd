// SPDX-License-Identifier: MIT
//! Hosted native MakeCode program on the no-ASF board: natural boot from the
//! authored boot region, runtime initialisation, USB enumeration and HF2.
//! Not a physical, timing, display or complete-peripheral qualification.
use labwired_config::{ChipDescriptor, SystemManifest};
use labwired_core::memory::{LinearMemory, ProgramImage, Segment};
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
type M = Machine<labwired_core::cpu::cortex_m::CortexM>;

// Function entries observed for the boot and USB path.
const POINTS: [&str; 8] = [
    "Reset_Handler",
    "_ZN3pxt5startEv",
    "_ZN3pxt11initRuntimeEv",
    "_ZN3pxt8usb_initEv",
    "_ZN8CodalUSB5startEv",
    "USB_0_Handler",
    "TC0_Handler",
    "target_panic",
];
#[derive(Debug)]
struct Entries {
    points: Vec<u32>,
    counts: Vec<AtomicU32>,
}
impl SimulationObserver for Entries {
    fn on_step_start(&self, pc: u32, _opcode: u32) {
        for (i, p) in self.points.iter().enumerate() {
            if pc == *p {
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

struct Host<'a> {
    m: &'a mut M,
    entries: Arc<Entries>,
    panic: u64,
    step: u64,
    bound: u64,
    replies: u32,
    tag: &'a str,
}
impl Host<'_> {
    fn count(&self, name: &str) -> u32 {
        let i = POINTS.iter().position(|p| *p == name).unwrap();
        self.entries.counts[i].load(Ordering::Relaxed)
    }
    /// One real guest instruction; a panic, fault, model refusal or the bound fails.
    fn step_once(&mut self, what: &str) {
        assert!(
            self.step < self.bound,
            "{what}: not reached within {} steps",
            self.bound
        );
        if u64::from(self.m.cpu.pc & !1) == self.panic {
            panic!("{what}: target_panic({}) step={}", self.m.cpu.r0, self.step);
        }
        if let Some(reason) = usb(self.m).refusal() {
            panic!("{what}: USB model refusal {reason}");
        }
        self.m
            .step()
            .unwrap_or_else(|e| panic!("{what}: {e:?} step={}", self.step));
        assert_ne!(
            self.m.cpu.active_exception, 3,
            "{what}: HardFault step={}",
            self.step
        );
        self.step += 1;
    }
    fn run_until(&mut self, what: &str, mut done: impl FnMut(&M) -> bool) {
        while !done(self.m) {
            self.step_once(what);
        }
    }
    /// One host transaction: queue the token, run real guest instructions until
    /// the device answers it. The reply is whatever the guest's controller produced.
    fn transact(&mut self, token: HostToken) -> HostReply {
        let tag = self.tag;
        println!(
            "NATIVE_HOST_QUEUE case={tag} step={} token={token:?}",
            self.step
        );
        usb_mut(self.m).queue_host(token).unwrap();
        let reply = loop {
            if let Some(reply) = usb_mut(self.m).take_host_reply() {
                break reply;
            }
            self.step_once("host reply");
        };
        println!(
            "NATIVE_HOST_REPLY case={tag} step={} ordinal={} reply={reply:?}",
            self.step, self.replies
        );
        self.replies += 1;
        reply
    }
    fn setup(&mut self, address: u8, packet: [u8; 8]) {
        assert_eq!(
            self.transact(HostToken::Setup { address, packet }),
            HostReply::SetupAccepted
        );
    }
    /// Control IN: data packets until a short one, then a zero-length status OUT.
    fn control_in(&mut self, address: u8, packet: [u8; 8]) -> Vec<u8> {
        self.setup(address, packet);
        let wanted = usize::from(u16::from_le_bytes([packet[6], packet[7]]));
        let mut data = vec![];
        loop {
            let HostReply::In {
                endpoint: 0,
                data: part,
            } = self.transact(HostToken::In {
                address,
                endpoint: 0,
            })
            else {
                panic!("control IN data stage")
            };
            let short = part.len() < 64;
            data.extend(part);
            if short || data.len() >= wanted {
                break;
            }
        }
        let status = self.transact(HostToken::Out {
            address,
            endpoint: 0,
            data: vec![],
        });
        assert_eq!(
            status,
            HostReply::OutAccepted {
                endpoint: 0,
                bytes: 0
            }
        );
        data
    }
    fn control_out(&mut self, address: u8, packet: [u8; 8]) {
        self.setup(address, packet);
        let status = self.transact(HostToken::In {
            address,
            endpoint: 0,
        });
        assert_eq!(
            status,
            HostReply::In {
                endpoint: 0,
                data: vec![]
            }
        );
    }
    /// One HF2 command over the bulk endpoints; returns the reassembled response.
    fn hf2(&mut self, out_ep: u8, in_ep: u8, command: u32, tag: u16) -> Vec<u8> {
        let mut packet = vec![0u8; 64];
        packet[0] = 0x40 | 8; // HF2_FLAG_CMDPKT_LAST, 8-byte command header
        packet[1..5].copy_from_slice(&command.to_le_bytes());
        packet[5..7].copy_from_slice(&tag.to_le_bytes());
        let sent = self.transact(HostToken::Out {
            address: 1,
            endpoint: out_ep,
            data: packet,
        });
        assert_eq!(
            sent,
            HostReply::OutAccepted {
                endpoint: out_ep,
                bytes: 64
            }
        );
        let mut message = vec![];
        loop {
            let HostReply::In { endpoint, data } = self.transact(HostToken::In {
                address: 1,
                endpoint: in_ep,
            }) else {
                panic!("HF2 response packet")
            };
            assert_eq!(endpoint, in_ep);
            if data.is_empty() {
                continue; // terminating ZLP of a full-size bulk packet
            }
            assert_eq!(data.len(), 64, "HF2 always writes 64-byte packets");
            let flag = data[0] & 0xc0;
            let size = usize::from(data[0] & 63);
            message.extend_from_slice(&data[1..1 + size]);
            if flag == 0x40 {
                break;
            }
            assert_eq!(flag, 0x00, "command body packet");
        }
        assert!(message.len() >= 4);
        assert_eq!(
            u16::from_le_bytes([message[0], message[1]]),
            tag,
            "HF2 response tag"
        );
        assert_eq!((message[2], message[3]), (0, 0), "HF2 status OK");
        message[4..].to_vec()
    }
}

#[test]
#[ignore = "hosted exact source/image/model native guest only"]
fn native_program_boot_usb_hf2() {
    let folder = PathBuf::from(std::env::var("BW_NATIVE_FIXTURE_DIR").unwrap());
    let engine = std::env::var("BW_NATIVE_ENGINE_SOURCE").unwrap();
    let profile = if cfg!(feature = "event-scheduler") {
        "event-scheduler"
    } else {
        "default"
    };
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join("image.json")).unwrap()).unwrap();
    let case = std::env::var("BW_NATIVE_CASE").unwrap();
    assert!(["positive", "no-info-pointer", "erased-reset-forward"].contains(&case.as_str()));
    let admitted = std::fs::read(folder.join("image.bin")).unwrap();
    assert_eq!(report["image_sha256"], digest(&admitted));
    // Manufactured controls mutate only the declared input bytes before load.
    let mut image = admitted.clone();
    match case.as_str() {
        "no-info-pointer" => image[0x3ffc..0x4000].copy_from_slice(&[0; 4]), // UF2_BInfo.info_uf2
        "erased-reset-forward" => image[0..8].copy_from_slice(&[0xff; 8]),
        _ => {}
    }
    let hash = digest(&image);
    assert_eq!(report["bootloader_code"], false);
    let elf_bytes = std::fs::read(folder.join("no-asf-board.elf")).unwrap();
    let elf = goblin::elf::Elf::parse(&elf_bytes).unwrap();
    let symbol = |name: &str| -> u32 {
        let rows: Vec<_> = elf
            .syms
            .iter()
            .filter(|s| s.st_shndx != 0 && elf.strtab.get_at(s.st_name) == Some(name))
            .collect();
        assert_eq!(rows.len(), 1, "unique base symbol {name}");
        (rows[0].st_value & !1) as u32
    };
    let points: Vec<u32> = POINTS.iter().map(|p| symbol(p)).collect();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut chip =
        ChipDescriptor::from_file(root.join("configs/chips/atsamd51-pybadge.yaml")).unwrap();
    let added: Vec<labwired_config::PeripheralConfig> =
        serde_yaml::from_str(include_str!("bw_native_peripherals.yaml")).unwrap();
    let ids: Vec<String> = added.iter().map(|p| p.id.clone()).collect();
    chip.peripherals.retain(|p| !ids.contains(&p.id));
    chip.peripherals.extend(added);
    let manifest: SystemManifest = serde_yaml::from_str(
        "name: native-board\nchip: ../chips/atsamd51-pybadge.yaml\nexternal_devices: []\n",
    )
    .unwrap();
    let mut bus = SystemBus::from_config(&chip, &manifest).unwrap();
    // Declared authored NVM inputs, not captures: SW0 calibration (the USB leaf's
    // documented pad defaults) and a 128-bit serial number.
    let mut sw0 = LinearMemory::new(16, 0x00800080);
    sw0.data.copy_from_slice(&[0xff; 16]);
    sw0.data[4..8].copy_from_slice(&0x1b29u32.to_le_bytes());
    bus.extra_mem.push(sw0);
    let mut serial = LinearMemory::new(0x200, 0x00806000);
    serial.data[0x10..0x1c].copy_from_slice(&[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0]);
    serial.data[0x1fc..0x200].copy_from_slice(&0x4257_0001u32.to_le_bytes());
    bus.extra_mem.push(serial);
    let (cpu, _) = configure_cortex_m(&mut bus);
    let mut m = Machine::new(cpu, bus);
    let entries = Arc::new(Entries {
        points: points.clone(),
        counts: points.iter().map(|_| AtomicU32::new(0)).collect(),
    });
    m.add_observer(entries.clone());
    let mut program = ProgramImage::new(0, labwired_core::Arch::Arm);
    program.segments.push(Segment {
        start_addr: 0,
        data: image.clone(),
    });
    m.load_firmware(&program).unwrap();
    // Architectural reset from the authored boot region at address 0.
    m.cpu.sp = u32::from_le_bytes(image[0..4].try_into().unwrap());
    m.cpu.pc = u32::from_le_bytes(image[4..8].try_into().unwrap()) & !1;
    println!("NATIVE_INPUT engine={engine} image_sha256={hash} case={case} profile={profile} physical_capture=false bootloader_code=false");
    if case == "erased-reset-forward" {
        // Erased reset words: the first fetch fails and the program is never entered.
        let first = m.step();
        assert!(first.is_err(), "erased reset forward must not execute");
        assert_eq!(
            entries.counts[0].load(Ordering::Relaxed),
            0,
            "Reset_Handler not entered"
        );
        println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} kind=no-reset-entry error={first:?}");
        return;
    }
    assert_eq!(
        m.cpu.pc, points[0],
        "reset forward reaches the program's Reset_Handler"
    );

    let mut host = Host {
        m: &mut m,
        entries,
        panic: u64::from(symbol("target_panic")),
        step: 0,
        bound: 1_500_000_000,
        replies: 0,
        tag: profile,
    };
    // Natural boot: reset, runtime initialisation, then the USB controller attaches.
    host.run_until("USB attach", |m| {
        let u = usb(m);
        u.read(0).unwrap() & 2 != 0 && u.read_u16(8).unwrap() & 1 == 0
    });
    for name in [
        "Reset_Handler",
        "_ZN3pxt5startEv",
        "_ZN3pxt11initRuntimeEv",
        "_ZN3pxt8usb_initEv",
        "_ZN8CodalUSB5startEv",
    ] {
        assert_eq!(host.count(name), 1, "{name} entered exactly once");
    }
    assert!(
        host.count("TC0_Handler") > 0,
        "system timer interrupts delivered"
    );
    println!(
        "NATIVE_BOOT case={profile} step={} runtime=true usb_attached=true tc0_irqs={}",
        host.step,
        host.count("TC0_Handler")
    );

    assert_eq!(host.transact(HostToken::Reset), HostReply::Reset);
    // The real reset handler re-initialises endpoints and enables control OUT.
    host.run_until("reset handler", |m| {
        m.cpu.active_exception == 0 && usb(m).read(0x100).unwrap() & 7 == 1
    });

    let device = host.control_in(0, [0x80, 6, 0, 1, 0, 0, 18, 0]);
    assert_eq!(
        device,
        [18, 1, 0x10, 2, 0, 0, 0, 64, 0xeb, 3, 0x66, 0x20, 2, 0x42, 1, 2, 3, 1],
        "PXT device descriptor"
    );
    host.control_out(0, [0, 5, 1, 0, 0, 0, 0, 0]);
    let head = host.control_in(1, [0x80, 6, 0, 2, 0, 0, 9, 0]);
    let total = u16::from_le_bytes([head[2], head[3]]);
    let config = host.control_in(1, [0x80, 6, 0, 2, 0, 0, total as u8, (total >> 8) as u8]);
    assert_eq!(config.len(), usize::from(total));
    // Interfaces as the guest describes them: HF2 (vendor, subclass 42, two bulk
    // endpoints) followed by the endpoint-less dummy interface.
    let mut interfaces = vec![];
    let mut endpoints = vec![];
    let mut at = 0;
    while at < config.len() {
        let (len, kind) = (usize::from(config[at]), config[at + 1]);
        match kind {
            4 => interfaces.push((
                config[at + 4],
                config[at + 5],
                config[at + 6],
                config[at + 7],
            )),
            5 => endpoints.push((config[at + 2], config[at + 3])),
            _ => {}
        }
        at += len;
    }
    assert_eq!(
        interfaces,
        [(2, 0xff, 42, 1), (0, 0xff, 0xff, 0xff)],
        "HF2 and dummy interfaces"
    );
    assert_eq!(endpoints, [(0x81, 2), (0x02, 2)], "HF2 bulk IN 1 and OUT 2");
    host.control_out(1, [0, 9, 1, 0, 0, 0, 0, 0]);
    println!(
        "NATIVE_ENUMERATED case={profile} step={} config_bytes={total} interfaces=2",
        host.step
    );

    let bininfo = host.hf2(2, 1, 0x0001, 0x5a01);
    let word = |i: usize| u32::from_le_bytes(bininfo[4 * i..4 * i + 4].try_into().unwrap());
    assert_eq!(bininfo.len(), 20);
    assert_eq!(word(0), 2, "HF2_MODE_USERSPACE");
    assert_eq!(word(3), 260, "max_message_size = HF2_BUF_SIZE");
    assert_eq!(word(4), 0x55114460, "SAMD51 UF2 family");
    let info = host.hf2(2, 1, 0x0002, 0x5a02);
    if case == "no-info-pointer" {
        // Without the pointer the guest's own uf2_info() reports its fallback.
        assert_eq!(info, b"N/A", "HF2 INFO without the UF2 information pointer");
        println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} kind=info-pointer-absent info={:?}", host.step, String::from_utf8_lossy(&info));
        return;
    }
    // The guest's own uf2_info(): the authored string behind the 0x3ffc pointer.
    assert_eq!(
        info, b"Authored no-ASF PyBadge metadata; no bootloader.\n",
        "HF2 INFO from the authored boot region"
    );
    assert_eq!(host.count("target_panic"), 0);
    println!(
        "NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} replies={} bininfo_mode=2 family=0x55114460 info_bytes={} native_program=true hf2=true physical_timing=false display=false",
        host.step, host.replies, info.len()
    );
}
