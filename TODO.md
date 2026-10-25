# TODO

Open work for the no-ASF SAMD51 board. Detailed scope and acceptance criteria
per lane are in [the remaining lanes](docs/PXT-NO-ASF-NEXT-LANES.md); history is
in [HISTORY.md](HISTORY.md).

## Board peripherals (lanes B–D)

The board compiles, links and boots a native program; each peripheral below
still needs its own LabWired model behaviour and an actual guest case with
nonvacuous controls.

- [ ] Pins and EIC: real pin reads, a natural EIC handler with disabled
      clock/pin/IRQ controls.
- [ ] PWM on TC/TCC waveform outputs.
- [ ] SPI byte-level checks of the display path; I2C master and USART through
      the support layer. The I2C master runs from the 48 MHz generator (see
      [the base](docs/PXT-NO-ASF-FULLBOARD-BASE.md)).
- [ ] TCC timers: SAMD51 delivers compare interrupts on per-channel lines
      (`TCCn_MCx`); `SAMDTCCTimer` enables only line 0. Confirm and fix if TCC
      timers are used.
- [ ] DAC and PDM audio.

## Native program (lanes E–F)

- [ ] Display output, input and audio under a real program, not only boot.
- [ ] HF2 flashing, debugger commands and JACDAC transport.
- [ ] Complete peripheral reset, repeated USB reset under the native program.

## Limits still to address

- [ ] Interrupted writes and power loss for storage.
- [ ] Representative performance.
- [ ] Unselected sensor drivers changed by the core migration (accelerometers)
      remain unqualified.
- [ ] codal-core's `GhostFAT` reads the board target's legacy `UF2_INFO_ADDR`
      (0x3fd0), not the 0x3ffc pointer PXT uses; unused by this board.
