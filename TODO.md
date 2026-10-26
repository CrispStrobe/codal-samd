# TODO

Open work for the no-ASF SAMD51 board. Detailed scope and acceptance criteria
per lane are in [the remaining lanes](docs/PXT-NO-ASF-NEXT-LANES.md); history is
in [HISTORY.md](HISTORY.md).

## Board peripherals

Pins/EIC, PWM, display SPI and frame pixels, buttons, I2C, SPI receive, USART,
ADC, DAC audio and the TCC timer driver run with controls
([native cases](no-asf-board/native/README.md)). Still open:

- [ ] TCC system timers from MakeCode: pxt-common-packages `allocateTimer`
      (`core---samd/platform.cpp`) tests `idx < 0x20` on the table index, not
      the timer id, so any TCC entry in `CFG_TIMERS_TO_USE` is constructed as a
      TC and panics (90). The file is a byte-bound request source here; report
      and fix it upstream. `SAMDTCCTimer` itself is fixed and qualified
      (TCC leaf).
      An owned request-copy discriminator patch and actual native TCC system-timer
      qualification are in [the candidate contract](docs/PXT-TCC-SYSTEM-TIMER.md).
- [ ] PDM: `SAMDPDM` is compiled but not selected (no microphone package or pin
      on this board).
- [ ] Analog levels: DAC output voltage, ADC reference/front-end behaviour,
      touch.
- [ ] SPI/I2C/USART bus timing; SPI slave mode.
- [ ] `ZSPI::dmaTransferComplete` discards received bytes while RXC is set after
      TX completes; with RX DMA active this could race the RX channel on
      silicon. Not observable in the simulator; review on hardware.

## Native program

- [ ] Displayed colour (RGBSET is never loaded; only 4-bit codes are checked),
      audio content beyond one tone.
- [ ] HF2 flashing needs a bootloader (`USB_HANDOVER`), which this board
      omits by design; JACDAC transport.
- [ ] Peripheral reset on `SYSRESETREQ` is modelled for the SAMD51 board models;
      models outside that set keep their state across a system reset.

## Limits still to address

- [ ] Interrupted writes and power loss for storage.
- [ ] Representative performance.
- [ ] Unselected sensor drivers changed by the core migration (accelerometers)
      remain unqualified.
- [ ] codal-core's `GhostFAT` reads the board target's legacy `UF2_INFO_ADDR`
      (0x3fd0), not the 0x3ffc pointer PXT uses; unused by this board.
- [ ] Hardware re-captures: the simulator results are not silicon captures.

Native controller-colour candidate: [explicit RGB444 simulation profile](docs/PXT-RGB444-COLOUR-PROFILE.md). Pending hosted qualification; original no-RGBSET and physical-module limits remain.
