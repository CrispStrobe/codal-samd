# TODO

Open work for the no-ASF SAMD51 board. Detailed scope and acceptance criteria
per lane are in [the remaining lanes](docs/PXT-NO-ASF-NEXT-LANES.md); history is
in [HISTORY.md](HISTORY.md).

## Board peripherals

Pins/EIC, PWM, display SPI and frame pixels, buttons, I2C, SPI receive, USART,
ADC, DAC audio and the TCC timer driver run with controls
([native cases](no-asf-board/native/README.md)). Still open:

- [x] MakeCode TCC system-timer selection in the owned runtime profile:
      [qualified discriminator patch and native controls](docs/PXT-TCC-SYSTEM-TIMER.md).
      The original byte-bound request is retained; this does not change upstream.
- [ ] PDM: `SAMDPDM` is compiled but not selected (no microphone package or pin
      on this board).
- [ ] Analog levels: DAC output voltage, ADC reference/front-end behaviour,
      touch.
- [ ] SPI/I2C/USART bus timing; SPI slave mode.
- [ ] `ZSPI::dmaTransferComplete` discards received bytes while RXC is set after
      TX completes; with RX DMA active this could race the RX channel on
      silicon. Not observable in the simulator; review on hardware.

## Native program

- [x] Controller-memory colour with the opt-in authored RGB444 profile:
      [qualified actual RGBSET upload and negative controls](docs/PXT-RGB444-COLOUR-PROFILE.md).
- [ ] Physical module colour: independently identify the panel and aperture,
      acquire lawful hardware observations, and compare RGBSET/gamma/output.
      Preserve the default no-upload path and unknown-colour refusal.
- [ ] Audio content beyond one tone: define a source-built multi-tone/program
      fixture and verify actual DAC samples and timing; analogue output remains separate.
- [ ] HF2 flashing needs a bootloader (`USB_HANDOVER`), which this board
      omits by design; JACDAC transport.
- [ ] Peripheral reset on `SYSRESETREQ` is modelled for the SAMD51 board models;
      models outside that set keep their state across a system reset.

## Limits still to address

- [x] Modeled atomic overwrite interruption: [qualified contract](pxt-storage-interruption/README.md).
      Both profiles passed exact payload/publication cuts, strict old/old/new
      fresh-process recovery and independent source/backing audits.
- [ ] Remaining interrupted storage: deletion, GC/erase and physical torn
      programming require separate fault models and exact recovery contracts
      [scoped here](pxt-storage-interruption/README.md).
- [x] Representative native/Node WASM game execution and measurement:
      [six qualified samples and measured limits](docs/SAMD-NATIVE-GAME-RESULTS.md).
- [ ] Core performance qualification, reproducible engine publication and Lite
      browser/installed adoption: [ordered acceptance criteria](docs/SAMD-NATIVE-GAME-RESULTS.md#remaining-work-and-acceptance).
- [ ] Unselected sensor drivers changed by the core migration (accelerometers)
      remain unqualified.
- [ ] codal-core's `GhostFAT` reads the board target's legacy `UF2_INFO_ADDR`
      (0x3fd0), not the 0x3ffc pointer PXT uses; unused by this board.
- [ ] Hardware re-captures: the simulator results are not silicon captures.

The [atomic-commit storage contract](pxt-storage-interruption/README.md) records
the qualified cuts and fresh recovery, with separate deletion/GC/erase scope.
