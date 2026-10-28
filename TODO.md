# TODO

Open work for the no-ASF SAMD51 board. Lanes A–G are implemented; every open
item below is scoped, with where to start and what counts as done, in
[section H of the lanes](docs/PXT-NO-ASF-NEXT-LANES.md#h-open-work-after-lanes-ag).
History is in [HISTORY.md](HISTORY.md).

## Board peripherals

Pins/EIC, PWM, display SPI and frame pixels, buttons, I2C, SPI receive, USART,
ADC, DAC audio, the TCC timer driver, SERCOM byte timing, settings storage
across reboots and the LIS3DH accelerometer run with controls
([native cases](no-asf-board/native/README.md)). Still open:

- [x] MakeCode TCC system-timer selection in the owned runtime profile:
      [qualified discriminator patch and native controls](docs/PXT-TCC-SYSTEM-TIMER.md).
      The original byte-bound request is retained; this does not change upstream.
- [ ] PDM: `SAMDPDM` is compiled but not selected (no microphone package or pin
      on this board).
- [x] Nominal DAC and speaker levels from the datasheets and the schematic
      ([oracles](docs/SIMULATION-ORACLES.md), `mix` case).
- [ ] Measured analog levels: DAC gain/offset, ADC reference and front-end
      behaviour, touch, the speaker's output (hardware).
- [x] I2C rise time (declared per bus), clock stretching (declared per device)
      and the SCL low time-out, which the support layer now enables.
- [x] Oscillator tolerance: the DFLL48M's open-loop range (datasheet table
      54-50) is outside USB's +-0.25 %; SAMDUSB now locks it to the start of
      frame (`usb-dfll-error`, control `usb-dfll-open-loop`).
- [x] Datasheet compliance: I2C against UM10204 standard mode, SPI baud as a
      ceiling, whole-clock serial frames, DAC and ADC error bands
      ([oracles](docs/SIMULATION-ORACLES.md)).
- [x] Flash blocks left unpredictable by a reset (datasheet 25.6.13):
      `store-gc-garbage`, which found a RAFFS name read before validation.
- [x] The display clock within the ST7735R's 15.15 MHz: an ST7735 is clamped
      to 14 MHz, room for an open-loop DFLL (12.5 MHz SCK; `display`, control
      `display-original`). The cost: about 41.7 frames a second where 24 MHz
      gave 50.
- [ ] Faster in-spec display: 14.67 MHz SCK (about 47 frames a second) needs
      DPLL1, the SPI and USART SERCOM clock, at 88 MHz and a clamp in hertz.
      Owner decision; scoped in [H1](docs/PXT-NO-ASF-NEXT-LANES.md#h1-faster-in-spec-display-clock-owner-decision-first).
- [x] Pad timing against table 54-14: SPI SCK and MOSI at full drive (10 ns
      edges at 30 pF instead of up to 40 ns, a whole half period at 12.5 MHz).
      The engine stores DRVSTR without an electrical model.
- [ ] SPI slave mode: no consumer on this board (the PyBadge has no SPI
      peer); left until a board needs it ([H4](docs/PXT-NO-ASF-NEXT-LANES.md#h4-spi-slave-mode-only-when-a-board-needs-it)).
- [x] `ZSPI::dmaTransferComplete` stole the last received byte from the RX DMA
      channel; observable once the DMAC services requests late
      (`spi-dma-latency`), fixed ([driver fixes](docs/DRIVER-FIXES.md)).

## Native program

- [x] Controller-memory colour: RGBSET is random after power-on (ST7735R
      datasheet), so the owned base now loads the
      [RGB444 profile](docs/PXT-RGB444-COLOUR-PROFILE.md) table by default;
      `palette` decodes all sixteen colours, `colour-no-lut` keeps the original
      no-upload path as its control.
- [ ] Glass colour: gamma, aperture, backlight and colour matching of the
      JDT1800 module need hardware observation.
- [x] Audio beyond one tone: two notes and a rest at their frequencies and
      durations (`melody` case).
- [x] Digital audio levels and mixing: volume scaling, two voices, clipping
      (`mix` case). The analogue level at the pad stays open (see Analog levels).
- [ ] HF2 flashing needs a bootloader (`USB_HANDOVER`), which this board
      omits by design; JACDAC transport.
- [x] Peripheral reset on `SYSRESETREQ`: the SAM models and the SAM PORT
      profile of the shared `gpio` model reset (PORT keeps the outside world's
      levels). Of the other shared models on this board, SysTick may keep
      RVR/CVR (UNKNOWN after reset, ARMv7-M ARM) and the generic UART on
      SERCOM3 is not used by the firmware.

## Limits still to address

- [x] Modeled atomic overwrite interruption: [qualified contract](pxt-storage-interruption/README.md).
      Both profiles passed exact payload/publication cuts, strict old/old/new
      fresh-process recovery and independent source/backing audits.
- [x] Power cuts during deletion, two compactions, `settings.clear()` and the
      scope wipe, and authored partial erases and torn programs inside one
      operation (`store-delete`, `store-gc`, `store-gc2`, `store-gc-torn`,
      `store-clear`, `store-scope`).
- [x] A torn RAFFS metadata entry no longer HardFaults the next boot
      ([driver fixes](docs/DRIVER-FIXES.md); `store-delete-original` keeps it).
- [ ] A data write lost under a committed metadata entry still reads a wrong
      value (`store-data-lost`): detecting it needs a checksum in the RAFFS
      format. The data-before-metadata order means a power cut cannot cause it.
      Owner decision ([H3](docs/PXT-NO-ASF-NEXT-LANES.md#h3-raffs-data-checksum-owner-decision-format-change)).
- [ ] Physical power-loss behaviour of the flash (which bits a real
      interrupted program or erase leaves) needs hardware observation; the
      fault models here are authored.
- [x] Representative native/Node WASM game execution and measurement:
      [six qualified samples and measured limits](docs/SAMD-NATIVE-GAME-RESULTS.md).
- [ ] Core performance qualification, reproducible engine publication and Lite
      browser/installed adoption: [ordered acceptance criteria](docs/SAMD-NATIVE-GAME-RESULTS.md#remaining-work-and-acceptance).
- [x] CODAL's MMA8453, MPU6050 and FXOS8700 drivers run in a leaf build
      against host sensors (`accel-leaf`); the MPU6050 sign bug is fixed in the
      owned core ([driver fixes](docs/DRIVER-FIXES.md)).
- [ ] codal-core's `GhostFAT` reads the board target's legacy `UF2_INFO_ADDR`
      (0x3fd0), not the 0x3ffc pointer PXT uses; unused by this board.
- [ ] Hardware re-captures: the simulator results are not silicon captures
      (capture plan in [H2](docs/PXT-NO-ASF-NEXT-LANES.md#h2-hardware-observations-needs-a-pybadge-and-instruments)).
- [ ] Engine drift acknowledgements in labwired-core expire on 2026-11-12
      (operator decision, [H6](docs/PXT-NO-ASF-NEXT-LANES.md#h6-engine-drift-acknowledgements-labwired-core-operator-decision)).

The [atomic-commit storage contract](pxt-storage-interruption/README.md) records
the qualified cuts and fresh recovery, with separate deletion/GC/erase scope.
