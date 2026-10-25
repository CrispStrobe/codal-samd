# TODO

Open work for the no-ASF SAMD51 board. Detailed scope and acceptance criteria
per lane are in [the remaining lanes](docs/PXT-NO-ASF-NEXT-LANES.md); history is
in [HISTORY.md](HISTORY.md).

## Board peripherals (lanes B–D)

Pins/EIC, PWM, display SPI, I2C, USART and DAC audio run under natively compiled
programs with controls ([native cases](no-asf-board/native/README.md)). Still
open:

- [ ] TCC timers: on SAMD51 compare interrupts arrive on per-channel lines
      (`TCCn_MC0..5`, ATDF lines 86–91 for TCC0) while `SAMDTCCTimer` enables
      only its base line (`TCCn_OTHER`). Unreachable on this board: the SAMD51
      default timer list is TC0, TC1, TC2 and CF2 sets no `CFG_TIMERS_TO_USE`.
      Fix and qualify before a board selects a TCC timer.
- [ ] PDM: `SAMDPDM` is compiled but not selected (no microphone package or pin
      on this board).
- [ ] Analog output level, ADC input and touch.
- [ ] USART pad routing in the LabWired model; SPI receive (MISO) and SPI/I2C
      bus timing.
- [ ] `SAMDDAC` programs TC3 with `CC0 = 8 MHz / rate`, but MFRQ wraps at
      CC0 + 1: playback runs 0.55 % slow at 44.1 kHz (43,956 Hz). Same on
      silicon; correct in the driver if pitch matters.

## Native program (lanes E–F)

- [ ] Display pixels, button input and audio content beyond one tone.
- [ ] HF2 flashing, debugger commands and JACDAC transport.
- [ ] Complete peripheral reset, repeated USB reset under the native program.

## Limits still to address

- [ ] Interrupted writes and power loss for storage.
- [ ] Representative performance.
- [ ] Unselected sensor drivers changed by the core migration (accelerometers)
      remain unqualified.
- [ ] codal-core's `GhostFAT` reads the board target's legacy `UF2_INFO_ADDR`
      (0x3fd0), not the 0x3ffc pointer PXT uses; unused by this board.
