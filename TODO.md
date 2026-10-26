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
- [ ] Analog levels: DAC output voltage, ADC reference/front-end behaviour,
      touch.
- [ ] Electrical bus timing (I2C rise time, clock stretching, pad delays) and
      oscillator tolerance; SPI slave mode.
- [ ] `ZSPI::dmaTransferComplete` discards received bytes while RXC is set after
      TX completes; with RX DMA active this could race the RX channel on
      silicon. Not observable in the simulator; review on hardware.

## Native program

- [x] Controller-memory colour with the opt-in authored RGB444 profile:
      [qualified actual RGBSET upload and negative controls](docs/PXT-RGB444-COLOUR-PROFILE.md).
- [ ] Physical module colour: independently identify the panel and aperture,
      acquire lawful hardware observations, and compare RGBSET/gamma/output.
      Preserve the default no-upload path and unknown-colour refusal.
- [x] Default no-upload colour path: all sixteen palette codes are sent and
      every pixel is an undefined lookup without RGBSET (`palette` case).
- [x] Audio beyond one tone: two notes and a rest at their frequencies and
      durations (`melody` case).
- [ ] Audio levels and mixing of overlapping sounds.
- [ ] HF2 flashing needs a bootloader (`USB_HANDOVER`), which this board
      omits by design; JACDAC transport.
- [ ] Peripheral reset on `SYSRESETREQ` covers the SAM models; shared models
      (generic UART, SysTick beyond CSR, declarative registers) keep their
      state, since changing them drifts hardware-captured boards.

## Limits still to address

- [x] Modeled atomic overwrite interruption: [qualified contract](pxt-storage-interruption/README.md).
      Both profiles passed exact payload/publication cuts, strict old/old/new
      fresh-process recovery and independent source/backing audits.
- [ ] Remaining interrupted storage: deletion, GC/erase and physical torn
      programming require separate fault models and exact recovery contracts
      [scoped here](pxt-storage-interruption/README.md).
      A lost data write under a committed metadata entry already reads a wrong
      value silently (`store-data-lost`; RAFFS has no checksum).
- [x] Representative native/Node WASM game execution and measurement:
      [six qualified samples and measured limits](docs/SAMD-NATIVE-GAME-RESULTS.md).
- [ ] Core performance qualification, reproducible engine publication and Lite
      browser/installed adoption: [ordered acceptance criteria](docs/SAMD-NATIVE-GAME-RESULTS.md#remaining-work-and-acceptance).
- [ ] The other accelerometer drivers the base compiles in (MMA8453, MPU6050)
      remain unqualified; LIS3DH runs (`accel` case).
- [ ] codal-core's `GhostFAT` reads the board target's legacy `UF2_INFO_ADDR`
      (0x3fd0), not the 0x3ffc pointer PXT uses; unused by this board.
- [ ] Hardware re-captures: the simulator results are not silicon captures.

The [atomic-commit storage contract](pxt-storage-interruption/README.md) records
the qualified cuts and fresh recovery, with separate deletion/GC/erase scope.
