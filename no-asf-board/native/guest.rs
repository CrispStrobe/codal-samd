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
    atomic::{AtomicBool, AtomicU32, Ordering},
    Arc, Mutex,
};
type M = Machine<labwired_core::cpu::cortex_m::CortexM>;

/// Records the display SPI wire: each byte the panel receives while its CS
/// (PB7) is low, with the D/C level (PB5) the host sampled before it.
struct DisplayTap {
    cs: &'static str,
    dc: Arc<AtomicBool>,
    bytes: Arc<Mutex<Vec<(bool, u8)>>>,
}

impl labwired_core::peripherals::spi::SpiDevice for DisplayTap {
    fn transfer(&mut self, mosi: u8) -> u8 {
        self.bytes.lock().unwrap().push((self.dc.load(Ordering::Relaxed), mosi));
        0
    }
    fn cs_pin(&self) -> &str {
        self.cs
    }
}

/// Command frames: a D/C-low byte followed by the D/C-high bytes after it.
fn frames(bytes: &[(bool, u8)]) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = Vec::new();
    for &(data, byte) in bytes {
        match (data, out.last_mut()) {
            (true, Some(frame)) => frame.push(byte),
            _ => out.push(vec![byte]),
        }
    }
    out
}

// Function entries observed for the boot and USB path.
const POINTS: [&str; 10] = [
    "Reset_Handler",
    "_ZN3pxt5startEv",
    "_ZN3pxt11initRuntimeEv",
    "_ZN3pxt8usb_initEv",
    "_ZN8CodalUSB5startEv",
    "USB_0_Handler",
    "TC0_Handler",
    "target_panic",
    "SERCOM5_0_Handler",
    "SERCOM5_2_Handler",
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
    /// PORT A index and last level of a host jumper from PA14 to PA16.
    jumper: Option<(usize, bool)>,
    /// Pin/EIC control: one gate the host holds off once the EIC is enabled.
    gate_off: Option<&'a str>,
    /// Display D/C level shared with the SPI tap.
    dc: Option<Arc<AtomicBool>>,
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
        // Controls hold one gate off once the guest enabled the controller
        // under test, so its own clock-synchronisation waits have completed.
        if let Some(gate) = self.gate_off {
            // (controller CTRLA, gate register, gate bit)
            let (ctrla, address, bit) = match gate {
                "pins-no-mux" => (0x40002800, 0x41008050, 1), // PORT PA16 PINCFG.PMUXEN
                "pins-eic-unclocked" => (0x40002800, 0x40001c90, 1 << 6), // GCLK PCHCTRL[4].CHEN
                "pins-irq-masked" => (0x40002800, 0, 0),
                "i2c-unclocked" => (0x41012000, 0x40001cdc, 1 << 6), // GCLK PCHCTRL[23].CHEN
                "i2c-unrouted" => (0x41012000, 0x4100804c, 1), // PORT PA12 (SDA) PINCFG.PMUXEN
                "display-mosi-unrouted" => (0x43000000, 0x410080cf, 1), // PORT PB15 PINCFG.PMUXEN
                "pwm-unrouted" => (0x42001400, 0x41008057, 1), // PORT PA23 PINCFG.PMUXEN
                "pwm-stopped" => (0x42001400, 0x42001400, 2), // TC4 CTRLA.ENABLE
                "uart-dre-masked" => (0x43000400, 0, 0),
                "audio-tc3-stopped" => (0x43002400, 0x4101c000, 2), // TC3 CTRLA.ENABLE
                "audio-dma-irq-masked" => (0x43002400, 0, 0),
                _ => unreachable!("gate {gate}"),
            };
            // Every controller here has ENABLE at bit 1 of its first byte.
            if self.m.bus.read_u8(ctrla).unwrap() & 2 != 0 {
                if gate == "pins-irq-masked" {
                    self.m.bus.write_u32(0xe000e180, 1 << 12).unwrap(); // NVIC ICER0: EIC_0
                } else if gate == "audio-dma-irq-masked" {
                    self.m.bus.write_u32(0xe000e184, 1).unwrap(); // NVIC ICER1: DMAC_1 (32)
                } else if gate == "uart-dre-masked" {
                    self.m.bus.write_u32(0xe000e188, 1 << 2).unwrap(); // NVIC ICER2: SERCOM5_0 (66)
                } else if address & 0xffff_ff00 == 0x4100_8000 {
                    // PORT PINCFG is one byte per pad; touch only this pad's.
                    let value = self.m.bus.read_u8(address).unwrap();
                    if value & bit as u8 != 0 {
                        self.m.bus.write_u8(address, value & !(bit as u8)).unwrap();
                    }
                } else {
                    let value = self.m.bus.read_u32(address).unwrap();
                    if value & bit != 0 {
                        self.m.bus.write_u32(address, value & !bit).unwrap();
                    }
                }
            }
        }
        if let Some(dc) = &self.dc {
            // PORT B OUT bit 5, the panel's D/C pad (driven output).
            dc.store(self.m.bus.read_u32(0x41008090).unwrap() & (1 << 5) != 0, Ordering::Relaxed);
        }
        if let Some((port, last)) = self.jumper {
            // The wire carries PA14's driven level; an undriven PA14 leaves it low.
            let out = self.m.bus.read_u32(0x41008010).unwrap() & self.m.bus.read_u32(0x41008000).unwrap();
            let level = out & (1 << 14) != 0;
            if level != last {
                self.m
                    .bus
                    .set_pad_external(port, 16, labwired_core::pins::External::Level(level));
                self.jumper = Some((port, level));
            }
        }
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
    /// HF2 serial output (SERIAL_OUT packets) until a complete line starting with `prefix`.
    fn serial_line(&mut self, in_ep: u8, prefix: &str) -> String {
        let mut text = String::new();
        loop {
            let HostReply::In { endpoint, data } = self.transact(HostToken::In {
                address: 1,
                endpoint: in_ep,
            }) else {
                panic!("HF2 serial packet")
            };
            assert_eq!(endpoint, in_ep);
            if data.is_empty() {
                continue;
            }
            assert_eq!(data[0] & 0xc0, 0x80, "only serial output is pending");
            text.push_str(&String::from_utf8_lossy(&data[1..1 + usize::from(data[0] & 63)]));
            if let Some(start) = text.find(prefix) {
                if let Some(end) = text[start..].find('\n') {
                    return text[start..start + end].to_string();
                }
            }
        }
    }
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
    let case = std::env::var("BW_NATIVE_CASE").unwrap();
    assert!([
        "positive",
        "no-info-pointer",
        "erased-reset-forward",
        "pins",
        "pins-no-jumper",
        "pins-no-mux",
        "pins-eic-unclocked",
        "pins-irq-masked",
        "i2c",
        "i2c-no-device",
        "i2c-unclocked",
        "i2c-unrouted",
        "display",
        "display-wrong-cs",
        "display-mosi-unrouted",
        "pwm",
        "pwm-unrouted",
        "pwm-stopped",
        "uart",
        "uart-no-rx",
        "uart-dre-masked",
        "audio",
        "audio-tc3-stopped",
        "audio-dma-irq-masked"
    ]
    .contains(&case.as_str()));
    // Peripheral cases run their own program; the others the empty program.
    let program = match case.split('-').next().unwrap() {
        "pins" => "image-pins",
        "i2c" => "image-i2c",
        "pwm" => "image-pwm",
        "uart" => "image-uart",
        "audio" => "image-audio",
        _ => "image",
    };
    let report: serde_json::Value =
        serde_json::from_slice(&std::fs::read(folder.join(format!("{program}.json"))).unwrap()).unwrap();
    let admitted = std::fs::read(folder.join(format!("{program}.bin"))).unwrap();
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
    // The UART program uses the serial extension and runs on its own base.
    let base = if program == "image-uart" { "no-asf-board-serial.elf" } else { "no-asf-board.elf" };
    let elf_bytes = std::fs::read(folder.join(base)).unwrap();
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

    if case.starts_with("i2c") && case != "i2c-no-device" {
        // A BMP280 on the board's I2C bus (SERCOM2: SDA PA12, SCL PA13).
        m.bus
            .attach_i2c_slave(
                "sercom2",
                Box::new(labwired_core::peripherals::components::bmp280::Bmp280::new(0x77)),
            )
            .unwrap();
    }
    let uart_tx = Arc::new(Mutex::new(Vec::new()));
    if case.starts_with("uart") {
        // The board's serial port: SERCOM5, TX PB16 / RX PB17.
        assert!(m.bus.attach_uart_tx_sink_named("sercom5", uart_tx.clone(), false));
    }
    let tap = Arc::new(Mutex::new(Vec::new()));
    let dc = Arc::new(AtomicBool::new(false));
    if case.starts_with("display") {
        // The panel on SERCOM4; the wrong-CS control wires it to PB6 instead.
        let cs = if case == "display-wrong-cs" { "PB6" } else { "PB7" };
        m.bus
            .attach_spi_device("sercom4", Box::new(DisplayTap { cs, dc: dc.clone(), bytes: tap.clone() }))
            .unwrap();
    }
    let mut host = Host {
        m: &mut m,
        entries,
        panic: u64::from(symbol("target_panic")),
        step: 0,
        bound: 1_500_000_000,
        replies: 0,
        tag: profile,
        jumper: None,
        gate_off: None,
        dc: case.starts_with("display").then(|| dc.clone()),
    };
    if case == "display-mosi-unrouted"
        || case == "uart-dre-masked"
        || case == "audio-tc3-stopped"
        || case == "audio-dma-irq-masked"
    {
        host.gate_off = Some(case.as_str());
    }
    if case.starts_with("pins") && case != "pins-no-jumper" {
        let port = host.m.bus.peripherals.iter().position(|p| p.name == "porta").unwrap();
        host.jumper = Some((port, false));
        host.gate_off = (case != "pins").then_some(case.as_str());
    }
    if case == "i2c-unclocked" || case == "i2c-unrouted" {
        host.gate_off = Some(case.as_str());
    }
    // Natural boot: reset, runtime initialisation, then the USB controller attaches.
    if case == "display-mosi-unrouted" {
        // Without its MOSI pad the SPI never moves the first byte, the screen's
        // DMA transfer never completes and initialisation blocks before USB
        // start: run past the positive attach point and observe both.
        for _ in 0..70_000_000u64 {
            host.step_once("MOSI-unrouted window");
        }
        let attached = {
            let u = usb(host.m);
            u.read(0).unwrap() & 2 != 0 && u.read_u16(8).unwrap() & 1 == 0
        };
        let bytes = tap.lock().unwrap().len();
        assert_eq!(bytes, 0, "no byte reaches the panel without MOSI");
        assert!(!attached, "USB start never runs behind the blocked screen init");
        assert_eq!(host.count("_ZN8CodalUSB5startEv"), 0);
        println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} bytes=0 usb_attached=false", host.step);
        return;
    }
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
    if case.starts_with("display") {
        // Screen initialisation precedes USB start; by attach the panel has had
        // its whole command sequence.
        let bytes = tap.lock().unwrap().clone();
        let got = frames(&bytes);
        if case == "display" {
            // codal-core ST7735::init (initCmds), configure(MADCTL=0x80,
            // FRMCTR1=0x012c2d) from CF2, then pxt screen.cpp's RGB444 COLMOD
            // and INVOFF.
            let init: [&[u8]; 12] = [
                &[0x01],
                &[0x11],
                &[0x20],
                &[0x3a, 0x03],
                &[0xe0, 0x02, 0x1c, 0x07, 0x12, 0x37, 0x32, 0x29, 0x2d, 0x29, 0x25, 0x2b, 0x39, 0x00, 0x01, 0x03, 0x10],
                &[0xe1, 0x03, 0x1d, 0x07, 0x06, 0x2e, 0x2c, 0x29, 0x2d, 0x2e, 0x2e, 0x37, 0x3f, 0x00, 0x00, 0x02, 0x10],
                &[0x13],
                &[0x29],
                &[0x36, 0x80],
                &[0xb1, 0x01, 0x2c, 0x2d],
                &[0x3a, 0x03],
                &[0x20, 0x00],
            ];
            assert!(got.len() > 12, "display frames {}", got.len());
            for (i, expected) in init.iter().enumerate() {
                assert_eq!(got[i], *expected, "display command {i}");
            }
            // The first full frame: 128x160 panel memory at 12 bits per pixel in
            // two windows, columns 0..=119 then 120..=127.
            let shape: Vec<(Vec<u8>, usize)> =
                got.iter().map(|f| (f[..f.len().min(5)].to_vec(), f.len() - 1)).collect();
            let frame = [
                (vec![0x2a, 0, 0, 0, 0x77], 4),
                (vec![0x2b, 0, 0, 0, 0x9f], 4),
                (vec![0x2c], 120 * 160 * 3 / 2),
                (vec![0x2a, 0, 0x78, 0, 0x7f], 4),
                (vec![0x2b, 0, 0, 0, 0x9f], 4),
                (vec![0x2c], 8 * 160 * 3 / 2),
            ];
            let at = shape
                .windows(frame.len())
                .position(|w| w.iter().zip(&frame).all(|((head, n), (h, m))| head[..h.len().min(head.len())] == h[..] && n == m))
                .expect("a complete 128x160 RGB444 frame");
            println!("NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} bytes={} commands=12 first_frame_at={at}", host.step, bytes.len());
        } else {
            // A panel on another CS line, or MOSI without its pad, sees nothing.
            assert!(bytes.is_empty(), "{case}: {} bytes reached the panel", bytes.len());
            println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} bytes=0", host.step);
        }
        assert_eq!(host.count("target_panic"), 0);
        return;
    }

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
    if case.starts_with("pins") {
        // The program reports what it read and how many rise events it handled.
        let line = host.serial_line(1, "BW-PINS");
        // The jumper alone decides the reads; only the full EIC path counts rises.
        let expected = match case.as_str() {
            "pins" => "BW-PINS low=0 high=1 rises=2",
            "pins-no-jumper" => "BW-PINS low=0 high=0 rises=0",
            _ => "BW-PINS low=0 high=1 rises=0",
        };
        let eic = host.m.bus.peripherals.iter().find(|p| p.name == "eic").unwrap().dev.snapshot();
        assert_eq!(line, expected, "EIC {eic}");
        // Each control fails at its own gate: no edge reaches the channel
        // without the mux, the channel never samples without its clock, and a
        // masked line leaves the detected edge pending in INTFLAG.
        match case.as_str() {
            "pins" => assert_eq!(eic["pinstate"], 1),
            "pins-no-mux" => assert_eq!(eic["pinstate"], 0),
            "pins-eic-unclocked" => assert_eq!(eic["clocked"], false),
            "pins-irq-masked" => assert_eq!(eic["intflag"].as_u64().unwrap() & 1, 1),
            _ => {}
        }
        assert_eq!(host.count("target_panic"), 0);
        let kind = if case == "pins" { "NATIVE_FINITE_PASS" } else { "NATIVE_NEGATIVE" };
        println!("{kind} engine={engine} image_sha256={hash} case={case} profile={profile} step={} line={line:?} eic={eic}", host.step);
        return;
    }
    if case.starts_with("audio") {
        host.serial_line(1, "BW-AUDIO");
        let dac = host.m.bus.peripherals.iter().find(|p| p.name == "dac").unwrap();
        let dac = dac.dev.as_any().unwrap().downcast_ref::<labwired_core::peripherals::sam::dac::Samd51Dac>().unwrap();
        let samples = dac.conversions().to_vec();
        let mut gaps = std::collections::BTreeMap::new();
        for w in samples.windows(2) {
            *gaps.entry(w[1].0 - w[0].0).or_insert(0u64) += 1;
        }
        let (lo, hi) = samples.iter().fold((u16::MAX, 0), |(lo, hi), s| (lo.min(s.1), hi.max(s.1)));
        let mid = (u32::from(lo) + u32::from(hi)) / 2;
        let rise_at: Vec<u64> = samples
            .windows(2)
            .filter(|w| u32::from(w[0].1) < mid && u32::from(w[1].1) >= mid)
            .map(|w| w[1].0)
            .collect();
        let rises = rise_at.len();
        let hz = (rises > 2).then(|| (rises - 1) as f64 * 120e6 / (rise_at[rises - 1] - rise_at[0]) as f64);
        assert_eq!(host.count("target_panic"), 0);
        // TC3 MFRQ at 8 MHz with CC0 = 8e6/44100 = 181 wraps every 182 counts:
        // one DMA half-word per 182 * 15 CPU cycles (43956 Hz, as on silicon;
        // SAMDDAC assumes 44198 Hz).
        let regular = gaps.get(&(182 * 15)).copied().unwrap_or(0);
        match case.as_str() {
            "audio" => {
                assert!(samples.len() > 4000, "{} conversions", samples.len());
                assert_eq!(regular as usize, samples.len() - 2, "every interval after the first is 2730 cycles: {gaps:?}");
                // The mixer's 440 Hz, played at the TC3 rate: 440 * 43956/44198.
                let hz = hz.expect("a periodic tone");
                assert!((hz - 437.6).abs() < 0.5, "tone {hz} Hz");
                assert!(lo < 2048 && hi > 2048, "centred on the 12-bit mid-scale: {lo}..{hi}");
                println!("NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} conversions={} interval_cycles=2730 tone_hz={hz:.2}", host.step, samples.len());
            }
            // Without TC3 nothing paces the DMA.
            "audio-tc3-stopped" => {
                assert_eq!(samples.len(), 0);
                println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} conversions=0", host.step);
            }
            // Without the DMA completion interrupt SAMDDAC never chains the next
            // buffer: exactly one 256-sample block plays.
            _ => {
                assert_eq!(samples.len(), 256, "one block");
                println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} conversions=256", host.step);
            }
        }
        return;
    }
    if case.starts_with("uart") {
        let tx = || String::from_utf8_lossy(&uart_tx.lock().unwrap()).to_string();
        if case == "uart-dre-masked" {
            // Every byte leaves on a DRE interrupt; with that line masked the
            // first writeLine never completes and the program never reports.
            for _ in 0..20_000_000u64 {
                host.step_once("DRE-masked window");
            }
            assert_eq!(tx(), "", "no byte without the DRE interrupt");
            assert_eq!(host.count("SERCOM5_0_Handler"), 0);
            println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} tx_bytes=0", host.step);
            return;
        }
        if case == "uart" {
            // The host answers on the RX pad with one line.
            let sercom5 = host.m.bus.peripherals.iter_mut().find(|p| p.name == "sercom5").unwrap();
            let uart = sercom5
                .dev
                .as_any_mut()
                .unwrap()
                .downcast_mut::<labwired_core::peripherals::sam::sercom_usart::SamSercomUsart>()
                .unwrap();
            uart.rx_buffer().lock().unwrap().extend(b"ping\n");
        }
        // The program reports every 100 ms; take the first report that shows a
        // received line, or the twentieth (two seconds) when none arrives.
        let mut line = host.serial_line(1, "BW-UART");
        for _ in 0..19 {
            if line != "BW-UART rx=-" {
                break;
            }
            line = host.serial_line(1, "BW-UART");
        }
        // writeLine: ten characters and CRLF, one DRE interrupt per byte.
        assert_eq!(tx(), "BW-UART tx\r\n");
        assert_eq!(host.count("SERCOM5_0_Handler"), 12, "one DRE entry per byte");
        assert_eq!(host.count("target_panic"), 0);
        if case == "uart" {
            // One RXC interrupt (SERCOM5_2) per received byte.
            assert_eq!(line, "BW-UART rx=ping");
            assert_eq!(host.count("SERCOM5_2_Handler"), 5, "one RXC entry per byte");
            println!("NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} tx={:?} line={line:?}", host.step, tx());
        } else {
            assert_eq!(line, "BW-UART rx=-");
            assert_eq!(host.count("SERCOM5_2_Handler"), 0);
            println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} line={line:?}", host.step);
        }
        return;
    }
    if case.starts_with("pwm") {
        host.serial_line(1, "BW-PWM");
        // The controls take hold once TC4 is enabled, which precedes the line.
        if case != "pwm" {
            host.gate_off = Some(case.as_str());
            // The gate takes hold on this step; the window starts after it.
            host.step_once("PWM gate");
        }
        // Record the LED pad (PA23) over three nominal periods.
        let port = host.m.bus.peripherals.iter().position(|p| p.name == "porta").unwrap();
        let level = |m: &M| m.bus.peripherals[port].dev.read_gpio_pad(23).unwrap();
        let start = host.m.bus.current_cycle();
        let mut last = level(host.m);
        let mut edges: Vec<(u64, bool)> = Vec::new();
        while host.m.bus.current_cycle() < start + 3 * 2_400_000 + 1_000 {
            host.step_once("PWM window");
            let now = level(host.m);
            if now != last {
                edges.push((host.m.bus.current_cycle(), now));
                last = now;
            }
        }
        let rises: Vec<u64> = edges.iter().filter(|e| e.1).map(|e| e.0).collect();
        let falls: Vec<u64> = edges.iter().filter(|e| !e.1).map(|e| e.0).collect();
        let tc4 = host.m.bus.peripherals.iter().find(|p| p.name == "tc4").unwrap().dev.snapshot();
        assert_eq!(host.count("target_panic"), 0);
        if case == "pwm" {
            // analogWrite(512) at codal's 20 ms period: pulse 512*20000/1023 =
            // 10009 us, DUTY 10009*0xffff/20000 = 32796. pwmout picks DIV64 at
            // 120 MHz (TOP 37499) and CC1 = 37499*32796/0xffff = 18765.
            assert_eq!(tc4["cc"], serde_json::json!([37499, 18765]), "TC4 {tc4}");
            assert!(rises.len() >= 2 && falls.len() >= 2, "edges {edges:?}");
            for pair in rises.windows(2) {
                assert_eq!(pair[1] - pair[0], 37_500 * 64, "period: 50 Hz at 120 MHz");
            }
            for rise in &rises {
                if let Some(fall) = falls.iter().find(|f| *f > rise) {
                    assert_eq!(fall - rise, 18_765 * 64, "high time: CC1 counts");
                }
            }
            println!("NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} period_cycles={} high_cycles={}", host.step, 37_500 * 64, 18_765 * 64);
        } else {
            // Without the pad routed to TC4, or with TC4 stopped, the pad is flat.
            assert!(edges.is_empty(), "{case}: edges {edges:?}");
            println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} edges=0", host.step);
        }
        return;
    }
    if case.starts_with("i2c") {
        let line = host.serial_line(1, "BW-I2C");
        let i2c = host.m.bus.peripherals.iter().find(|p| p.name == "sercom2").unwrap().dev.snapshot();
        let addresses: Vec<u64> = i2c["addresses"].as_array().unwrap().iter().map(|a| a.as_u64().unwrap()).collect();
        if case == "i2c" {
            // Pointer write and repeated-start read at 0x77; the NACKed write
            // to 0x10 is retried once (codal MAX_I2C_RETRIES = 2).
            assert_eq!(line, "BW-I2C id=88 absent=-1010", "SERCOM2 {i2c}");
            assert_eq!(addresses, [0xee, 0xef, 0x20, 0x20]);
        } else {
            // No device, no clock or no SDA routing: the read fails and the
            // write to the absent address still reports DEVICE_I2C_ERROR.
            assert_eq!(line, "BW-I2C id=undefined absent=-1010", "SERCOM2 {i2c}");
            // Without a device the bus runs and every address is NACKed; a
            // master without clock or pad drives nothing at all.
            assert_eq!(addresses.is_empty(), case != "i2c-no-device", "SERCOM2 {i2c}");
        }
        assert_eq!(host.count("target_panic"), 0);
        let kind = if case == "i2c" { "NATIVE_FINITE_PASS" } else { "NATIVE_NEGATIVE" };
        println!("{kind} engine={engine} image_sha256={hash} case={case} profile={profile} step={} line={line:?} addresses={addresses:?}", host.step);
        return;
    }
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
