# TODO

Open work for the no-ASF SAMD51 board. Detailed scope and acceptance criteria
per lane are in [the remaining lanes](docs/PXT-NO-ASF-NEXT-LANES.md); history is
in [HISTORY.md](HISTORY.md).

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
- [ ] Pad electrical timing and oscillator tolerance; SPI slave mode.
- [ ] `ZSPI::dmaTransferComplete` discards received bytes while RXC is set after
      TX completes; with RX DMA active this could race the RX channel on
      silicon. Not observable in the simulator; review on hardware.

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
- [ ] Peripheral reset on `SYSRESETREQ` covers the SAM models; shared models
      (generic UART, SysTick beyond CSR, declarative registers) keep their
      state, since changing them drifts hardware-captured boards.

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
- [ ] Hardware re-captures: the simulator results are not silicon captures.

The [atomic-commit storage contract](pxt-storage-interruption/README.md) records
the qualified cuts and fresh recovery, with separate deletion/GC/erase scope.
