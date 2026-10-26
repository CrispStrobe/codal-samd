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
    panel: Option<Arc<Mutex<labwired_core::peripherals::components::declarative_display::GenericDisplay>>>,
}

impl labwired_core::peripherals::spi::SpiDevice for DisplayTap {
    fn transfer(&mut self, mosi: u8) -> u8 {
        let dc = self.dc.load(Ordering::Relaxed);
        self.bytes.lock().unwrap().push((dc, mosi));
        if let Some(panel) = &self.panel {
            use labwired_core::peripherals::spi::SpiDevice;
            let mut panel = panel.lock().unwrap();
            panel.set_dc_level(dc);
            panel.transfer(mosi);
        }
        0
    }
    fn cs_pin(&self) -> &str {
        self.cs
    }
}

/// ST7735 frame memory as the controller addresses it: 160 rows (RASET) by 128
/// columns (CASET) of RGB444 codes, written column-fastest within the window.
/// The codes are the 4-bit channel values on the wire; the panel's expansion
/// to 6 bits goes through an RGBSET table this firmware never loads (its
/// power-on contents are unspecified), so no colour beyond the codes is claimed.
struct Panel {
    mem: Vec<Option<u16>>,
    cols: (usize, usize),
    rows: (usize, usize),
    at: (usize, usize),
}

impl Panel {
    fn new() -> Self {
        Self { mem: vec![None; 160 * 128], cols: (0, 127), rows: (0, 159), at: (0, 0) }
    }
    fn window(args: &[u8]) -> (usize, usize) {
        (usize::from(args[0]) << 8 | usize::from(args[1]), usize::from(args[2]) << 8 | usize::from(args[3]))
    }
    fn put(&mut self, code: u16) {
        let (col, row) = self.at;
        if row < 160 && col < 128 {
            self.mem[row * 128 + col] = Some(code);
        }
        // Column address first, then the row (MADCTL MV = 0).
        self.at = if col >= self.cols.1 { (self.cols.0, row + 1) } else { (col + 1, row) };
    }
    fn apply(&mut self, frame: &[u8]) {
        match frame[0] {
            0x2a if frame.len() == 5 => self.cols = Self::window(&frame[1..]),
            0x2b if frame.len() == 5 => self.rows = Self::window(&frame[1..]),
            0x2c => {
                self.at = (self.cols.0, self.rows.0);
                for pair in frame[1..].chunks(3) {
                    if let [a, b, c] = *pair {
                        self.put(u16::from(a) << 4 | u16::from(b >> 4));
                        self.put(u16::from(b & 15) << 8 | u16::from(c));
                    }
                }
            }
            _ => {}
        }
    }
    fn at(&self, row: usize, col: usize) -> Option<u16> {
        self.mem[row * 128 + col]
    }
}

/// A shift-register SPI device: each byte it receives is answered on the next
/// transfer (the first answer is 0x3C).
struct ShiftDevice {
    last: u8,
    received: Arc<Mutex<Vec<u8>>>,
}

impl labwired_core::peripherals::spi::SpiDevice for ShiftDevice {
    fn transfer(&mut self, mosi: u8) -> u8 {
        self.received.lock().unwrap().push(mosi);
        std::mem::replace(&mut self.last, mosi)
    }
    fn cs_pin(&self) -> &str {
        ""
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
const POINTS: [&str; 13] = [
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
    "TCC0_0_Handler",
    "TCC0_1_Handler",
    "TCC0_2_Handler",
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
    /// The PyBadge button shift register (74HC165) on LATCH PB00, CLOCK PB31
    /// and DATA PB30: (pressed inputs, shift register, last latch, last clock,
    /// last data, loads while latched).
    keys: Option<(u8, u8, bool, bool, bool, bool)>,
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
            panic!("{what}: target_panic({}) step={} lr={:#x}", self.m.cpu.r0, self.step, self.m.cpu.lr);
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
                "uart-tx-unrouted" => (0x43000400, 0x410080d0, 1), // PORT PB16 PINCFG.PMUXEN
                "uart-rx-unrouted" => (0x43000400, 0x410080d1, 1), // PORT PB17 PINCFG.PMUXEN
                "audio-tc3-stopped" => (0x43002400, 0x4101c000, 2), // TC3 CTRLA.ENABLE
                "audio-dma-irq-masked" => (0x43002400, 0, 0),
                "tcc-leaf-mc-masked" | "tcc-system-irqs-masked" => (0x41016000, 0, 0),
                "spi-miso-unrouted" => (0x40003400, 0x410080d6, 1), // PORT PB22 PINCFG.PMUXEN
                "adc-unclocked" => (0x43001c00, 0x40001d20, 1 << 6), // GCLK PCHCTRL[40].CHEN
                _ => unreachable!("gate {gate}"),
            };
            // Every controller here has ENABLE at bit 1 of its first byte.
            if self.m.bus.read_u8(ctrla).unwrap() & 2 != 0 {
                if gate == "pins-irq-masked" {
                    self.m.bus.write_u32(0xe000e180, 1 << 12).unwrap(); // NVIC ICER0: EIC_0
                } else if gate == "tcc-system-irqs-masked" {
                    self.m.bus.write_u32(0xe000e188, 0x7f << 21).unwrap(); // TCC0_0..6, IRQ85..91
                } else if gate == "tcc-leaf-mc-masked" {
                    self.m.bus.write_u32(0xe000e188, 1 << 22).unwrap(); // NVIC ICER2: TCC0_1 (86)
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
        if let Some((pressed, mut shift, last_latch, last_clock, last_data, loads)) = self.keys {
            let out = self.m.bus.read_u32(0x41008090).unwrap() & self.m.bus.read_u32(0x41008080).unwrap();
            let (latch, clock) = (out & 1 != 0, out & (1 << 31) != 0);
            if !latch && loads {
                shift = pressed; // SH/LD low: parallel load of A..H
            } else if latch && clock && !last_clock {
                shift <<= 1; // rising CLK in shift mode; SER tied low
            }
            let _ = last_latch;
            let data = shift & 0x80 != 0; // QH
            if data != last_data {
                let port = self.m.bus.peripherals.iter().position(|p| p.name == "portb").unwrap();
                self.m.bus.set_pad_external(port, 30, labwired_core::pins::External::Level(data));
            }
            self.keys = Some((pressed, shift, latch, clock, data, loads));
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
    /// One HF2 command with `payload` in a single packet; serial output the
    /// program sends meanwhile is skipped. Returns (status, response data).
    fn hf2_cmd(&mut self, command: u32, tag: u16, payload: &[u8]) -> (u8, Vec<u8>) {
        assert!(payload.len() <= 55, "single-packet HF2 command");
        let mut packet = vec![0u8; 64];
        packet[0] = 0x40 | (8 + payload.len()) as u8;
        packet[1..5].copy_from_slice(&command.to_le_bytes());
        packet[5..7].copy_from_slice(&tag.to_le_bytes());
        packet[9..9 + payload.len()].copy_from_slice(payload);
        let sent = self.transact(HostToken::Out { address: 1, endpoint: 2, data: packet });
        assert_eq!(sent, HostReply::OutAccepted { endpoint: 2, bytes: 64 });
        let mut message = vec![];
        loop {
            let HostReply::In { data, .. } = self.transact(HostToken::In { address: 1, endpoint: 1 }) else {
                panic!("HF2 response packet")
            };
            if data.is_empty() || data[0] & 0x80 != 0 {
                continue; // ZLP, or interleaved serial output
            }
            let size = usize::from(data[0] & 63);
            message.extend_from_slice(&data[1..1 + size]);
            if data[0] & 0xc0 == 0x40 {
                break;
            }
        }
        assert_eq!(u16::from_le_bytes([message[0], message[1]]), tag, "HF2 response tag");
        (message[2], message[4..].to_vec())
    }
    fn read_words(&mut self, address: u32, words: u32, tag: u16) -> Vec<u32> {
        let mut payload = address.to_le_bytes().to_vec();
        payload.extend_from_slice(&words.to_le_bytes());
        let (status, data) = self.hf2_cmd(0x0008, tag, &payload);
        assert_eq!(status, 0, "READ_WORDS");
        data.chunks(4).map(|w| u32::from_le_bytes(w.try_into().unwrap())).collect()
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

/// Bus reset and enumeration against the guest's own descriptors; returns the
/// configuration descriptor length.
fn enumerate(host: &mut Host) -> u16 {
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
    total
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
        "uart-tx-unrouted",
        "uart-rx-unrouted",
        "audio",
        "audio-tc3-stopped",
        "audio-dma-irq-masked",
        "frame",
        "colour",
        "colour-short-lut",
        "colour-no-lut",
        "input",
        "input-none",
        "input-no-load",
        "tcc-leaf",
        "tcc-leaf-mc-masked",
        "tcc-system",
        "tcc-system-irqs-masked",
        "tcc-original",
        "hf2",
        "spi",
        "spi-no-device",
        "spi-miso-unrouted",
        "adc",
        "adc-zero",
        "adc-unclocked"
    ]
    .contains(&case.as_str()));
    // Peripheral cases run their own program; the others the empty program.
    let program = match case.as_str() {
        "colour" => "image-colour",
        "colour-short-lut" => "image-colour-short",
        "colour-no-lut" => "image-frame",
        "tcc-system" | "tcc-system-irqs-masked" => "image-tcc-system",
        "tcc-original" => "image-tcc-original",
        _ => match case.split('-').next().unwrap() {
        "pins" => "image-pins",
        "i2c" => "image-i2c",
        "pwm" => "image-pwm",
        "uart" => "image-uart",
        "audio" => "image-audio",
        "frame" => "image-frame",
        "input" => "image-input",
        "tcc" => "image-tcc-leaf",
        "hf2" => "image-hf2",
        "spi" => "image-spi",
        "adc" => "image-adc",
        _ => "image",
        },
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
    let base = match program {
        "image-colour" => "no-asf-board-colour.elf",
        "image-colour-short" => "no-asf-board-colour-short.elf",
        "image-uart" => "no-asf-board-serial.elf",
        "image-tcc-leaf" => "no-asf-board-tcc.elf",
        "image-tcc-original" => "no-asf-board-original-timer.elf",
        _ => "no-asf-board.elf",
    };
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
    // The TCC leaf has no MakeCode runtime: its absent entry points never match.
    let points: Vec<u32> = POINTS
        .iter()
        .map(|p| if program == "image-tcc-leaf" && p.contains("pxt") || program == "image-tcc-leaf" && p.contains("USB") { u32::MAX } else { symbol(p) })
        .collect();

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

    if case.starts_with("tcc-leaf") {
        // CODAL's SAMDTCCTimer on TCC0 at 1 MHz: MC0 re-armed 1000 counts ahead
        // from its own interrupt, eight times.
        let done = u64::from(symbol("bw_tcc_done"));
        let matches = u64::from(symbol("bw_tcc_matches"));
        let counts = u64::from(symbol("bw_tcc_counts"));
        let mut host = Host {
            m: &mut m,
            entries,
            panic: u64::from(symbol("target_panic")),
            step: 0,
            bound: 60_000_000,
            replies: 0,
            tag: profile,
            jumper: None,
            gate_off: (case == "tcc-leaf-mc-masked").then_some(case.as_str()),
            dc: None,
            keys: None,
        };
        if case == "tcc-leaf" {
            host.run_until("eight TCC0 compare matches", |m| m.bus.read_u32(done).unwrap() == 1);
            let seen: Vec<u32> = (0..8).map(|i| host.m.bus.read_u32(counts + 4 * i).unwrap()).collect();
            // Each capture lands just after its match: exactly 1000 counts apart.
            assert!((1000..1010).contains(&seen[0]), "{seen:?}");
            assert!(seen.windows(2).all(|w| w[1] - w[0] == 1000), "{seen:?}");
            // Compare interrupts arrive on TCC0_1 (MC0), never on line 0.
            assert_eq!(host.count("TCC0_1_Handler"), 8);
            assert_eq!(host.count("TCC0_0_Handler"), 0);
            println!("NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} counts={seen:?} mc0_entries=8", host.step);
        } else {
            for _ in 0..30_000_000u64 {
                host.step_once("MC-masked window");
            }
            // With TCC0_1 masked no compare interrupt is taken.
            assert_eq!(host.m.bus.read_u32(matches).unwrap(), 0);
            assert_eq!(host.count("TCC0_1_Handler"), 0);
            println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} matches=0", host.step);
        }
        assert_eq!(host.count("target_panic"), 0);
        return;
    }
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
    let spi_seen = Arc::new(Mutex::new(Vec::new()));
    if case == "spi" || case == "spi-miso-unrouted" {
        m.bus
            .attach_spi_device("sercom1", Box::new(ShiftDevice { last: 0x3c, received: spi_seen.clone() }))
            .unwrap();
    }
    let colour_panel = if case.starts_with("colour") {
        use labwired_core::peripherals::components::declarative_display::GenericDisplay;
        use labwired_core::peripherals::spi::SpiDevice;
        let mut panel = GenericDisplay::from_yaml(include_str!("bw_native_st7735r_memory.yaml")).unwrap();
        // Force the pin-level parser and supply only the actual sampled PB5.
        panel.set_dc_source(0x41008090, 5);
        Some(Arc::new(Mutex::new(panel)))
    } else { None };
    let tap = Arc::new(Mutex::new(Vec::new()));
    let dc = Arc::new(AtomicBool::new(false));
    if case.starts_with("display") || case == "frame" || case.starts_with("colour") {
        // The panel on SERCOM4; the wrong-CS control wires it to PB6 instead.
        let cs = if case == "display-wrong-cs" { "PB6" } else { "PB7" };
        m.bus
            .attach_spi_device("sercom4", Box::new(DisplayTap { cs, dc: dc.clone(), bytes: tap.clone(), panel: colour_panel.clone() }))
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
        dc: (case.starts_with("display") || case == "frame" || case.starts_with("colour")).then(|| dc.clone()),
        keys: case.starts_with("input").then_some((0, 0, true, true, false, case != "input-no-load")),
    };
    if case == "display-mosi-unrouted"
        || case == "uart-dre-masked"
        || case == "audio-tc3-stopped"
        || case == "audio-dma-irq-masked"
        || case == "tcc-system-irqs-masked"
        || case == "spi-miso-unrouted"
        || case == "uart-tx-unrouted"
        || case == "uart-rx-unrouted"
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
    if case == "tcc-original" {
        let panic_pc = host.panic;
        host.run_until("original-index factory reaches target_panic", |m| {
            u64::from(m.cpu.pc & !1) == panic_pc
        });
        assert_eq!(host.m.cpu.r0, 90, "specific wrong timer-class constructor panic");
        assert_eq!(host.count("Reset_Handler"), 1);
        assert_eq!(host.count("_ZN3pxt11initRuntimeEv"), 1);
        assert_eq!(host.count("_ZN8CodalUSB5startEv"), 0);
        assert_eq!(host.count("TCC0_1_Handler"), 0);
        println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} kind=original-index-tcc-constructor-panic code=90 native_program=true", host.step);
        return;
    }
    if case == "tcc-system-irqs-masked" {
        for _ in 0..70_000_000u64 {
            host.step_once("TCC system IRQ-masked window");
        }
        assert_eq!(host.count("TC0_Handler"), 0);
        assert_eq!(host.count("TCC0_0_Handler"), 0);
        assert_eq!(host.count("TCC0_1_Handler"), 0);
        assert_eq!(host.count("TCC0_2_Handler"), 0);
        assert_eq!(host.m.bus.read_u32(0xe000e108).unwrap() & (0x7f << 21), 0);
        assert_ne!(host.m.bus.read_u32(0xe000e208).unwrap() & (1 << 23), 0, "CODAL system-event MC1 remains pending");
        assert_eq!(host.count("_ZN8CodalUSB5startEv"), 0, "sleep cannot finish without system-timer IRQs");
        println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} kind=tcc-system-irqs-masked tc0_entries=0 tcc_mc1_entries=0 pending=true usb_started=false", host.step);
        return;
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
    if case == "tcc-system" {
        assert_eq!(host.count("TC0_Handler"), 0, "no fallback to the old TC0 timer");
        assert!(host.count("TCC0_2_Handler") > 0, "actual CODAL MC1 system-event compares delivered");
        assert!(host.step < 70_000_000, "positive attach precedes the IRQ-control window");
    } else {
        assert!(host.count("TC0_Handler") > 0, "system timer interrupts delivered");
    }
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

    let total = enumerate(&mut host);
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
    if case == "tcc-system" {
        assert_eq!(host.count("target_panic"), 0);
        assert_eq!(host.count("TC0_Handler"), 0);
        assert!(host.count("TCC0_2_Handler") > 0);
        println!("NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} system_timer=TCC0 tc0_entries=0 tcc_mc1_entries={} hf2_bininfo={bininfo:02x?}", host.step, host.count("TCC0_2_Handler"));
        return;
    }
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
    if case == "frame" || case.starts_with("colour") {
        host.serial_line(1, "BW-FRAME");
        // Two more reports: frames rendered after the sprite exists.
        host.serial_line(1, "BW-FRAME");
        host.serial_line(1, "BW-FRAME");
        let got = frames(&tap.lock().unwrap());
        // Replay up to the last complete main-area frame (rows 0..=159,
        // columns 0..=119, 160 x 120 RGB444 pixels).
        let last = got
            .iter()
            .rposition(|f| f[0] == 0x2c && f.len() - 1 == 160 * 120 * 3 / 2)
            .expect("a main-area frame");
        let mut panel = Panel::new();
        for f in &got[..=last] {
            panel.apply(f);
        }
        let mut codes = std::collections::BTreeMap::new();
        for row in 0..160 {
            for col in 0..120 {
                *codes.entry(panel.at(row, col)).or_insert(0u32) += 1;
            }
        }
        let inside = |row: usize, col: usize| (32..48).contains(&row) && (22..38).contains(&col);
        let sprite = panel.at(40, 30);
        let background = panel.at(0, 0);
        let mismatches = (0..160)
            .flat_map(|row| (0..120).map(move |col| (row, col)))
            .filter(|&(row, col)| panel.at(row, col) != if inside(row, col) { sprite } else { background })
            .count();
        assert_eq!(host.count("target_panic"), 0);
        // setAddrWindow puts screen x on rows and screen y on columns, so the
        // sprite at screen (32..48, 22..38) occupies rows 32..48, columns
        // 22..38. Codes are the palette's high nibbles: colour 7 #78dc52 and
        // colour 2 #ff2121 of the Arcade default palette.
        assert_eq!(background, Some(0x7d5), "{codes:03x?}");
        assert_eq!(sprite, Some(0xf22), "{codes:03x?}");
        assert_eq!(mismatches, 0, "{codes:03x?}");
        if case.starts_with("colour") {
            let packet = tap.lock().unwrap();
            let wire_bytes: Vec<u8> = packet.iter().flat_map(|&(dc, b)| [u8::from(dc), b]).collect();
            std::fs::write(folder.join(format!("{profile}-{case}.wire")), &wire_bytes).unwrap();
            drop(packet);
            let wire = &got[..=last];
            let luts: Vec<_> = wire.iter().filter(|f| f[0] == 0x2d).collect();
            let expected_lut: Vec<u8> = [32usize, 64, 32].into_iter().flat_map(|n|
                (0..n).map(|i| { let v = i.min(15) as u8; (v << 2) | (v >> 2) })).collect();
            match case.as_str() {
                "colour" => assert_eq!(luts.iter().map(|f| &f[1..]).collect::<Vec<_>>(), vec![expected_lut.as_slice()]),
                "colour-short-lut" => assert_eq!(luts.iter().map(|f| &f[1..]).collect::<Vec<_>>(), vec![&expected_lut[..127]]),
                "colour-no-lut" => assert!(luts.is_empty()),
                _ => unreachable!(),
            }
            let panel = colour_panel.as_ref().unwrap().lock().unwrap();
            let known = panel.known_pixels().expect("serial colour validity");
            let ram = panel.framebuffer();
            std::fs::write(folder.join(format!("{profile}-{case}.ram")), ram).unwrap();
            std::fs::write(folder.join(format!("{profile}-{case}.known")), known.iter().map(|&v| u8::from(v)).collect::<Vec<_>>()).unwrap();
            let mut observed = Vec::new();
            for row in 0..160 {
                for col in 0..120 {
                    // MADCTL=0x80 mirrors Y over full GM00 controller RAM.
                    // The 132x162 prototype is not an asserted glass aperture.
                    let at = (161 - row) * 132 + col;
                    assert_eq!(known[at], case == "colour", "row={row} col={col}");
                    if case == "colour" {
                        let expected = if inside(row, col) { [255, 32, 32] } else { [117, 223, 85] };
                        assert_eq!(&ram[3 * at..3 * at + 3], &expected, "row={row} col={col}");
                        observed.extend_from_slice(&ram[3 * at..3 * at + 3]);
                    }
                }
            }
            let kind = if case == "colour" { "NATIVE_FINITE_PASS" } else { "NATIVE_NEGATIVE" };
            println!("{kind} engine={engine} image_sha256={hash} case={case} profile={profile} step={} pixels=19200 known={} rgbset_bytes={} controller_rgb888_sha256={} glass_qualified=false", host.step, if case == "colour" { 19200 } else { 0 }, luts.first().map_or(0, |f| f.len() - 1), digest(&observed));
            return;
        }
        println!("NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} background=7d5 sprite=f22 rows=32..48 cols=22..38 pixels=19200", host.step);
        return;
    }
    if case.starts_with("input") {
        host.serial_line(1, "BW-INPUT");
        // A host keeps polling HF2 while keys are held: the program's reports
        // wait on the IN endpoint, so holding without reading would stall it.
        let report = |host: &mut Host, pressed: u8| {
            if let Some(k) = host.keys.as_mut() {
                k.0 = pressed;
            }
            (0..5).map(|_| host.serial_line(1, "BW-INPUT")).last().unwrap()
        };
        let (held, after) = if case == "input-none" {
            (report(&mut host, 0), report(&mut host, 0))
        } else {
            // A (pin 1006, input G), then LEFT (pin 1000, input A) instead.
            (report(&mut host, 1 << 6), report(&mut host, 1))
        };
        let expected = if case == "input" {
            ("BW-INPUT a=1 left=0 ah=1", "BW-INPUT a=1 left=1 ah=0")
        } else {
            // No key, or a register that never loads its inputs: all released.
            ("BW-INPUT a=0 left=0 ah=0", "BW-INPUT a=0 left=0 ah=0")
        };
        assert_eq!((held.as_str(), after.as_str()), expected);
        assert_eq!(host.count("target_panic"), 0);
        let kind = if case == "input" { "NATIVE_FINITE_PASS" } else { "NATIVE_NEGATIVE" };
        println!("{kind} engine={engine} image_sha256={hash} case={case} profile={profile} step={} held={held:?} after={after:?}", host.step);
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
        // TC3 MFRQ at 8 MHz with CC0 = 8e6/44100 - 1 = 180 wraps every 181
        // counts: one DMA half-word per 181 * 15 CPU cycles (44198 Hz, the rate
        // SAMDDAC reports to the mixer).
        let regular = gaps.get(&(181 * 15)).copied().unwrap_or(0);
        match case.as_str() {
            "audio" => {
                assert!(samples.len() > 4000, "{} conversions", samples.len());
                assert_eq!(regular as usize, samples.len() - 2, "every interval after the first is 2715 cycles: {gaps:?}");
                // The mixer's 440 Hz at the rate it was synthesised for.
                let hz = hz.expect("a periodic tone");
                assert!((hz - 440.0).abs() < 0.5, "tone {hz} Hz");
                assert!(lo < 2048 && hi > 2048, "centred on the 12-bit mid-scale: {lo}..{hi}");
                println!("NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} conversions={} interval_cycles=2715 tone_hz={hz:.2}", host.step, samples.len());
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
    if case.starts_with("adc") {
        let adc0 = host.m.bus.peripherals.iter_mut().find(|p| p.name == "adc0").unwrap();
        let adc0 = adc0.dev.as_any_mut().unwrap().downcast_mut::<labwired_core::peripherals::sam::adc::Samd51Adc>().unwrap();
        if case == "adc" || case == "adc-unclocked" {
            adc0.set_input_millivolts(5, 1650); // A1
            adc0.set_input_millivolts(2, 825); // A2
        }
        if case == "adc-unclocked" {
            // The guest waits on RESRDY without a timeout: no report arrives.
            host.gate_off = Some(case.as_str());
            let conversions = |host: &Host| {
                let adc0 = host.m.bus.peripherals.iter().find(|p| p.name == "adc0").unwrap();
                adc0.dev.snapshot()["conversions"].as_u64().unwrap()
            };
            // The gate takes hold at the next read; after that nothing converts.
            for _ in 0..20_000_000u64 {
                host.step_once("ADC-unclocked window");
            }
            let held = conversions(&host);
            for _ in 0..20_000_000u64 {
                host.step_once("ADC-unclocked window");
            }
            assert_eq!(conversions(&host), held, "no conversion without the generic clock");
            assert_eq!(host.count("target_panic"), 0);
            println!("NATIVE_NEGATIVE engine={engine} image_sha256={hash} case={case} profile={profile} step={} conversions_frozen_at={held}", host.step);
            return;
        }
        let line = host.serial_line(1, "BW-ADC");
        // floor(V * 1024 / 3300 mV) at 10 bits against VDDANA.
        let expected = if case == "adc" { "BW-ADC a1=512 a2=256" } else { "BW-ADC a1=0 a2=0" };
        assert_eq!(line, expected);
        assert_eq!(host.count("target_panic"), 0);
        let kind = if case == "adc" { "NATIVE_FINITE_PASS" } else { "NATIVE_NEGATIVE" };
        println!("{kind} engine={engine} image_sha256={hash} case={case} profile={profile} step={} line={line:?}", host.step);
        return;
    }
    if case.starts_with("spi") {
        let line = host.serial_line(1, "BW-SPI");
        let seen = spi_seen.lock().unwrap().clone();
        assert_eq!(host.count("target_panic"), 0);
        match case.as_str() {
            "spi" => {
                // Full duplex on SERCOM1 with RX DMA: each reply is the byte the
                // device received before (0x3c first, then 0x5a).
                assert_eq!(line, "BW-SPI rx=60,90");
                assert_eq!(seen, [0x5a, 0xa5]);
            }
            "spi-no-device" => {
                assert_eq!(line, "BW-SPI rx=255,255", "nothing drives MISO");
                assert!(seen.is_empty());
            }
            _ => {
                // MISO unrouted: the device still receives; its replies do not arrive.
                assert_eq!(line, "BW-SPI rx=255,255");
                assert_eq!(seen, [0x5a, 0xa5]);
            }
        }
        let kind = if case == "spi" { "NATIVE_FINITE_PASS" } else { "NATIVE_NEGATIVE" };
        println!("{kind} engine={engine} image_sha256={hash} case={case} profile={profile} step={} line={line:?} device_received={seen:02x?}", host.step);
        return;
    }
    if case == "hf2" {
        let first = host.serial_line(1, "BW-HF2");
        assert_eq!(first, "BW-HF2 marker=1234");
        // DMESG: the runtime's own log, ending with the USB start.
        let (status, log) = host.hf2_cmd(0x0010, 0x6801, &[]);
        let log = String::from_utf8_lossy(&log).to_string();
        assert_eq!(status, 0);
        assert!(log.contains("allocate TC0") && log.contains("USB start"), "{log}");
        // READ_WORDS: the authored CF2 magic at 0x3000.
        assert_eq!(host.read_words(0x3000, 2, 0x6802), [0x1e9e10f1, 0x20227a79]);
        // Debugger: find the program's globals, then the one holding 1234.
        let (status, state) = host.hf2_cmd(0x53fc66e0, 0x6803, &[]);
        assert_eq!(status, 0);
        let count = u32::from_le_bytes(state[0..4].try_into().unwrap());
        let base = u32::from_le_bytes(state[4..8].try_into().unwrap());
        let mut globals = vec![];
        while (globals.len() as u32) < count {
            let n = (count - globals.len() as u32).min(64);
            let at = base + 4 * globals.len() as u32;
            globals.extend(host.read_words(at, n, 0x6804));
        }
        let tagged = |v: u32| v << 1 | 1; // PXT small-integer encoding
        let slots: Vec<usize> = (0..globals.len()).filter(|&i| globals[i] == tagged(1234)).collect();
        assert_eq!(slots.len(), 1, "one global holds 1234: {globals:x?}");
        // WRITE_WORDS: the program sees its global change.
        let mut payload = (base + 4 * slots[0] as u32).to_le_bytes().to_vec();
        payload.extend_from_slice(&1u32.to_le_bytes());
        payload.extend_from_slice(&tagged(5678).to_le_bytes());
        assert_eq!(host.hf2_cmd(0x0009, 0x6805, &payload).0, 0, "WRITE_WORDS");
        let mut line = host.serial_line(1, "BW-HF2");
        for _ in 0..4 {
            if line == "BW-HF2 marker=5678" {
                break;
            }
            line = host.serial_line(1, "BW-HF2");
        }
        assert_eq!(line, "BW-HF2 marker=5678");
        // Natural refusals: no JACDAC transport, and no flashing in user space.
        assert_eq!(host.hf2_cmd(0x0021, 0x6806, &[0; 16]).0, 2, "JDS_SEND: invalid state");
        assert_eq!(host.hf2_cmd(0x0006, 0x6807, &[0; 8]).0, 1, "WRITE_FLASH_PAGE: invalid command");
        // RESET_INTO_APP: the guest resets through SYSRESETREQ before replying.
        let mut packet = vec![0u8; 64];
        packet[0] = 0x40 | 8;
        packet[1..5].copy_from_slice(&0x0003u32.to_le_bytes());
        assert_eq!(
            host.transact(HostToken::Out { address: 1, endpoint: 2, data: packet }),
            HostReply::OutAccepted { endpoint: 2, bytes: 64 }
        );
        // A host keeps polling IN; the guest drains its pending output, takes
        // the command and resets.
        while host.count("Reset_Handler") < 2 {
            usb_mut(host.m).queue_host(HostToken::In { address: 1, endpoint: 1 }).unwrap();
            // The reset device leaves the bus: a poll in flight gets no reply.
            while usb_mut(host.m).take_host_reply().is_none() && host.count("Reset_Handler") < 2 {
                host.step_once("reset into app");
            }
        }
        while host.count("_ZN8CodalUSB5startEv") < 2 {
            host.step_once("second USB start");
        }
        host.run_until("USB attach after reset", |m| {
            let u = usb(m);
            u.read(0).unwrap() & 2 != 0 && u.read_u16(8).unwrap() & 1 == 0
        });
        enumerate(&mut host);
        let again = host.hf2(2, 1, 0x0001, 0x5a11);
        assert_eq!(u32::from_le_bytes(again[0..4].try_into().unwrap()), 2, "user space after reset");
        let fresh = host.serial_line(1, "BW-HF2");
        assert_eq!(fresh, "BW-HF2 marker=1234", "a real reboot re-initialises the global");
        assert_eq!(host.count("target_panic"), 0);
        println!("NATIVE_FINITE_PASS engine={engine} image_sha256={hash} case={case} profile={profile} step={} globals={count} slot={} dmesg_bytes={} boots=2", host.step, slots[0], log.len());
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
        if case != "uart-no-rx" {
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
        // received line, or the fifth (half a second) when none arrives. The
        // positive case shows its line in the first two reports.
        let mut line = host.serial_line(1, "BW-UART");
        for _ in 0..4 {
            if line != "BW-UART rx=-" {
                break;
            }
            line = host.serial_line(1, "BW-UART");
        }
        // writeLine: ten characters and CRLF, one DRE interrupt per byte; with
        // the TX pad unrouted the SERCOM still sends them, onto no wire.
        let wire = if case == "uart-tx-unrouted" { "" } else { "BW-UART tx\r\n" };
        assert_eq!(tx(), wire);
        assert_eq!(host.count("SERCOM5_0_Handler"), 12, "one DRE entry per byte");
        assert_eq!(host.count("target_panic"), 0);
        if case == "uart" || case == "uart-tx-unrouted" {
            // One RXC interrupt (SERCOM5_2) per received byte.
            assert_eq!(line, "BW-UART rx=ping");
            assert_eq!(host.count("SERCOM5_2_Handler"), 5, "one RXC entry per byte");
            let kind = if case == "uart" { "NATIVE_FINITE_PASS" } else { "NATIVE_NEGATIVE" };
            println!("{kind} engine={engine} image_sha256={hash} case={case} profile={profile} step={} tx={:?} line={line:?}", host.step, tx());
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
