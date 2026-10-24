# TODO

Open work for the no-ASF SAMD51 board. Detailed scope and acceptance criteria
per lane are in [the remaining lanes](docs/PXT-NO-ASF-NEXT-LANES.md); history is
in [HISTORY.md](HISTORY.md).

## Board peripherals (lanes B–D)

The full-board base compiles and links; each peripheral still needs a LabWired
model and an actual guest qualification with nonvacuous controls.

- [ ] Pins, EIC and PWM: real pin reads/writes, a natural EIC handler with
      disabled-clock/pin/IRQ controls, TC/TCC PWM.
- [ ] Serial, SPI, I2C and DMA through the new support layer: actual transfers,
      missing-completion/IRQ/buffer controls. The I2C master runs from the
      48 MHz generator (see [the base](docs/PXT-NO-ASF-FULLBOARD-BASE.md)).
- [ ] Timers and audio: the CODAL system timer on TC0, DAC/PDM paths.
- [ ] TCC compare interrupts arrive on per-channel NVIC lines on SAMD51;
      `SAMDTCCTimer` enables only line 0. Confirm and fix if TCC timers are used.

## Native program boot and HF2 (lanes E–F)

- [ ] Authored boot region: reset forwarding and CF2 configuration at the
      UF2 information block (0x3ff0), without a vendor bootloader.
- [ ] Boot the natively compiled empty program through runtime initialisation
      to its main loop in both scheduler profiles.
- [ ] Enumerate USB with the real PXT HF2 interface and compare guest-produced
      HF2 responses for typed host requests.

## Limits still to address

- [ ] Complete peripheral reset; interrupted writes and power loss for storage.
- [ ] Display, input, debugger operation and representative performance.
- [ ] Unselected sensor drivers changed by the core migration (accelerometers)
      remain unqualified.
