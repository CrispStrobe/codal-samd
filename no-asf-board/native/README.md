# Native MakeCode programs on the no-ASF board

MakeCode programs compiled natively by the admitted Arcade 4.2.1 / PXT 13.2.1
compiler onto the [full-board base](../../docs/PXT-NO-ASF-FULLBOARD-BASE.md) boot
in LabWired, answer HF2 over USB and drive the board's peripherals.

## Image

[boot.py](boot.py) assembles one flash image from the compiler's UF2 output.
Below the application origin 0x4000 there is no bootloader code: an authored
8-byte reset forward (the program's own stack top and reset entry) and the
admitted [CF2 fragments](../../docs/PXT-CF2-AUTHORED-METADATA.md): configuration
words at 0x3000, the information string at 0x3400 and the 16-byte UF2
information block at 0x3ff0 (`config_data`, two absent handover hooks,
`info_uf2`). This matches the layout PXT reads through
`UF2_BINFO = BOOTLOADER_END - sizeof(UF2_BInfo)`.

## Programs

| Program | Base | Exercises |
| --- | --- | --- |
| empty `main.ts` | base request | boot, USB, HF2 |
| [pins.ts](programs/pins.ts) | base request | digital write/read, rise events through the EIC |
| [i2c.ts](programs/i2c.ts) | base request | I2C register write, repeated-start read, NACK |
| [pwm.ts](programs/pwm.ts) | base request | `analogWrite` on the LED pad (TC4 WO1) |
| [audio.ts](programs/audio.ts) | base request | `music.playTone` through the mixer to the DAC |
| [uart.ts](programs/uart.ts) | serial request | serial TX and RX on SERCOM5 |

The serial extension changes the native build request, so the UART program runs
on the base built for that request (`build.py --variant serial`). Programs
report over HF2 serial (`console.log`), which the host reads after `BININFO`.

## Cases

[guest.rs](guest.rs) runs in LabWired with the PyBadge chip descriptor and the
opt-in board models in [peripherals.yaml](peripherals.yaml). Declared authored
NVM inputs supply the SW0 calibration word and a serial number; they are not
hardware captures. Every observation comes from guest code driving a model;
nothing is synthesised. A control holds exactly one gate off (a pad mux, a
clock, an NVIC line, a stopped timer) once the guest has enabled the controller
under test, or removes one host-side input.

- **Boot**: architectural reset from address 0 through the reset forward,
  data/BSS, clock tree, runtime constructors, `pxt::start`, `initRuntime`
  (system timer on TC0 with delivered interrupts, CF2 reads, display and flash
  storage initialisation) and the scheduler; the USB controller attaches after
  the program's 500 ms `start_usb` sleep.
- **Enumeration and HF2**: the device descriptor equals PXT's source table; the
  configuration describes the HF2 interface (vendor class, subclass 42, bulk EP1
  IN / EP2 OUT) and the dummy interface; `BININFO` reports user-space mode, a
  260-byte message size and family `0x55114460`; `INFO` returns the authored
  information string through the 0x3ffc pointer. Controls: `no-info-pointer`
  (the guest's own `"N/A"`), `erased-reset-forward` (first fetch fails).
- **Pins and EIC** (`pins`): a host jumper from PA14 to PA16; the program reads
  low and high and counts two rises handled by the EIC interrupt. Controls:
  no jumper, PA16 PMUX off, EIC generic clock off, NVIC EIC_0 masked (the edge
  stays pending in INTFLAG).
- **I2C** (`i2c`): a BMP280 at 0x77 on SERCOM2 (SDA PA12, SCL PA13); the chip id
  0x58 reads back and the write to 0x10 is NACKed and retried once. Controls:
  no device, SERCOM2 generic clock off, SDA unrouted (no bus traffic, driver
  timeouts).
- **Display SPI** (`display`): every byte the panel receives on SERCOM4 with its
  D/C level; the command sequence equals codal-core `ST7735::init`,
  `configure(MADCTL, FRMCTR1)` and the RGB444 setup, followed by a complete
  128x160 RGB444 frame in two windows. Controls: the panel on another CS line,
  MOSI unrouted (initialisation blocks before USB start).
- **PWM** (`pwm`): the PA23 pad itself; period 2,400,000 cycles (50 Hz at
  120 MHz) and high time 18,765 x 64 cycles, as derived from codal's `pwmout`
  arithmetic. Controls: pad unrouted, TC4 stopped (flat pad).
- **USART** (`uart`): `"BW-UART tx\r\n"` leaves SERCOM5 with one DRE interrupt
  per byte, and the host's `"ping\n"` arrives with one RXC interrupt per byte.
  Controls: no host bytes, DRE line masked (no byte leaves).
- **Audio** (`audio`): DAC channel 0 conversions; one every 2,730 cycles (TC3
  MFRQ wraps at CC0 + 1 = 182 counts of 8 MHz, 43,956 Hz) and a 437.6 Hz tone
  (440 Hz synthesised for 44,198 Hz, which SAMDDAC assumes). Controls: TC3
  stopped (no conversions), DMAC_1 masked (exactly one 256-sample block).

## Limits

Step counts are guest instructions; cycle figures are nominal engine cycles. No
physical analog level, pad electrical timing, SPI/I2C bus timing, display
pixels, input, debugger, flashing over HF2 or complete peripheral reset is
qualified. The USART model does not check pad routing. TCC timers and PDM are
not selected on this board (see [TODO](../../TODO.md)).
