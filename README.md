# codal-samd

CODAL device support for Microchip ATSAMD21 and ATSAMD51 microcontrollers, as
used by MakeCode Arcade and other CODAL targets. It provides the SAMD drivers
for CODAL's device-independent APIs: pins with PWM, analog input and external
interrupts (`ZPin`, `SAMDEIC`), SPI, I2C and serial on SERCOM (`ZSPI`, `ZI2C`,
`SAMDSerial`, `ZSingleWireSerial`), DMA (`SAMDDMAC`, `DmaFactory`), TC/TCC
timers, DAC and PDM audio, NVM storage and the USB device controller.

This is CrispStrobe's fork of Lancaster University's CODAL SAMD port. Besides
the drivers, it builds the SAMD51 board for simulation in
[LabWired](https://github.com/CrispStrobe/labwired-core) without the ASF4
framework or vendor-restricted headers, and qualifies the selected runtime
pieces against real CODAL and MakeCode sources in that simulator.

## What is in the repository

| Path | Contents |
| --- | --- |
| `src/`, `inc/`, `asm/` | The CODAL SAMD drivers and the Cortex-M context switch. |
| `no-asf-board/support/` | An independently written support layer (GPIO, pin table, clock tree, TC/TCC, EIC, SERCOM, I2C/SPI/USART, ADC, startup) that lets the drivers build without ASF4 or samd-peripherals. |
| `no-asf-board/fullboard/` | Builds the complete MakeCode Arcade `samd51adafruit` board base from its exact native build request, and admits every compiler dependency by origin. |
| `no-asf-board/native/` | Assembles a natively compiled program with the authored boot region and runs it in LabWired: boot, USB enumeration, HF2. |
| `no-asf-board/usb/` | A USB device leaf: the real CODAL USB core and SAMD driver, exercised against a modelled USB host. |
| `tools/samd51-device-header/` | Generates SAMD51 device headers from Microchip's Apache-2.0 ATDF device description, with layout checks. |
| `tools/pxt-native-request/` | Reproduces MakeCode's native build request offline and compiles a MakeCode program onto an owned base. |
| `tools/generated-board-profile/`, `profiles/` | The reproduced generated board configuration and byte identities of the MakeCode framework sources. |
| `runtime-services/`, `simulation/`, `paired-irq/`, `timer-caller/`, `pxt-*/` | Simulation fixtures that run real CODAL and PXT callers (timers, fibers, heap, SPI, GC, storage) under LabWired. |
| `docs/` | Specifications and results for each of the above. |

## Building

The default CMake build is the upstream CODAL library build (it selects ASF4
and samd-peripherals for real hardware) and is unchanged.

The no-ASF full-board base is built by
`.github/workflows/pxt-no-asf-fullboard.yml`, which fetches and hash-verifies
every input (MakeCode payloads from npm, the Microchip pack, pinned CODAL
checkouts), then runs:

```sh
node tools/pxt-native-request/request.mjs target.json pxtworker.js samd51adafruit request.json
python3 tools/samd51-device-header/generate.py ATSAMD51J19A.atdf device/
python3 no-asf-board/fullboard/build.py --core CORE --generator CODAL --board BOARD \
    --request request.json --device device/ --out out/
node tools/pxt-native-request/compile.mjs target.json pxtworker.js samd51adafruit out/no-asf-board.hex main.ts out/program
```

It needs `arm-none-eabi-gcc` with newlib, CMake, Python 3 and Node.js. See
[the full-board base](docs/PXT-NO-ASF-FULLBOARD-BASE.md) for inputs, the
dependency categories and limits. Each simulation fixture has its own workflow
in `.github/workflows/` and a document in `docs/`.

## Status

The full-board base compiles and links, and natively compiled MakeCode programs
boot on it in simulation from an authored boot region with no bootloader code:
runtime initialisation, the system timer, USB enumeration and HF2
`BININFO`/`INFO` and the debug services, a reboot with re-enumeration, and the
board's peripherals under real programs — rendered frames, buttons, pins and
external interrupts, PWM on a pad, the display SPI byte stream, I2C, SPI, serial,
the ADC and DAC audio, each with controls; the TCC timer driver runs in a leaf
build ([native cases](no-asf-board/native/README.md)).
Also
qualified: the USB device leaf (enumeration, WebUSB descriptors, echo, repeated
reset, fault controls) and the bounded runtime, SPI, GC and storage callers
listed in [HISTORY.md](HISTORY.md). The authored RGB444 profile qualifies
controller-memory colour, and the [active game results](docs/SAMD-NATIVE-GAME-RESULTS.md)
cover three native and three Node WASM samples with measured limits. Physical
display colour, PDM, analog levels and HF2 flashing (no bootloader) remain open.
Simulation builds are not qualified for flashing to hardware. Open work is in
[TODO.md](TODO.md).

## Licensing, attribution, and scope

Preserve upstream attribution and assess the exact components selected by each
build. Historical build inputs include vendor-restricted ASF sources:
reproduction or preprocessing does not admit those sources into a distributable
runtime. Newly authored MIT components do not relabel their dependencies or
establish whole-firmware clean-room origin, legal clearance, or store eligibility.
MPL/LGPL components can be considered with their particular obligations retained.
Restricted/reference firmware is not bundled or offered for automatic download.
The generated SAMD51 device headers derive from Microchip's Apache-2.0 SAMD51
device family pack; its notice and the licence text are in
[tools/samd51-device-header/notices](tools/samd51-device-header/notices/microchip-samd51-dfp-3.9.276-LICENSE.txt).
