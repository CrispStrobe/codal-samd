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
| [frame.ts](programs/frame.ts) | base request | a rendered frame: background and one sprite |
| [input.ts](programs/input.ts) | base request | buttons through the 74HC165 shift register |
| [hf2.ts](programs/hf2.ts) | base request | HF2 debug services and reset |
| [spi.ts](programs/spi.ts) | base request | full-duplex SPI on the board's default SPI |
| [adc.ts](programs/adc.ts) | base request | `analogRead` on A1 and A2 |
| [palette.ts](programs/palette.ts) | base request | all sixteen palette colours as bands |
| [melody.ts](programs/melody.ts) | base request | two notes and a rest through the mixer |
| [store.ts](programs/store.ts) | base request | a settings counter read at startup and written back |
| [accel.ts](programs/accel.ts) | base request | `input.acceleration` through CODAL's LIS3DH driver |
| [mix.ts](programs/mix.ts) | base request | volume levels, two voices at once, a clipping sum |
| [storage.ts](programs/storage.ts) | base request | settings seed, deletion, then forty rewrites that compact the bank |
| [uart.ts](programs/uart.ts) | serial request | serial TX and RX on SERCOM5 |
| [TCC leaf](../tcc-leaf/main.cpp) | `--variant tcc-leaf` | `SAMDTCCTimer` compare interrupts (no MakeCode runtime) |
| [accelerometer leaf](../accel-leaf/main.cpp) | `--variant accel-leaf` | CODAL's MMA8453, MPU6050 and FXOS8700 drivers, which MakeCode does not select here |

The serial extension changes the native build request, so the UART program runs
on the base built for that request (`build.py --variant serial`). The
accelerometer sources are already part of the base request, so `accel.ts` only
names the extension on its first line (`// extensions: accelerometer`), which
the request and compile tools add to the project. Programs report over HF2
serial (`console.log`), which the host reads after `BININFO`; the storage
programs report through the runtime log instead, read from RAM before USB starts.
The leaves link the base's compiled core, drivers and support layer with their
own `main` and no MakeCode runtime; their image is an 8-byte reset forward to
the application.

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
  0x58 reads back and the write to 0x10 is NACKed and retried once, its last
  STOP completing before ZI2C resets the master. Controls:
  no device, SERCOM2 generic clock off, SDA unrouted (no bus traffic, driver
  timeouts).
- **Display SPI** (`display`): every byte the panel receives on SERCOM4 with its
  D/C level; the command sequence equals codal-core `ST7735::init`,
  `configure(MADCTL, FRMCTR1)`, the RGB444 setup and the 128-byte RGBSET table,
  followed by a complete
  128x160 RGB444 frame in two windows. Controls: the panel on another CS line,
  MOSI unrouted (initialisation blocks before USB start).
- **PWM** (`pwm`): the PA23 pad itself; period 2,400,000 cycles (50 Hz at
  120 MHz) and high time 18,765 x 64 cycles, as derived from codal's `pwmout`
  arithmetic. Controls: pad unrouted, TC4 stopped (flat pad).
- **USART** (`uart`): `"BW-UART tx\r\n"` leaves SERCOM5 with one DRE interrupt
  per byte, and the host's `"ping\n"` arrives with one RXC interrupt per byte.
  Controls: no host bytes, DRE line masked (no byte leaves), TX pad unrouted
  (sent, onto no wire), RX pad unrouted (nothing received).
- **Audio** (`audio`): DAC channel 0 conversions, one every 2,715 cycles (TC3
  MFRQ at 8 MHz wrapping every 181 counts, 44,198 Hz) and a 440 Hz tone.
  Controls: TC3 stopped (no conversions), DMAC_1 masked (exactly one
  256-sample block).
- **Frame** (`frame`): the panel memory decoded from the SPI stream (CASET/RASET
  windows, column-fastest RAMWR, RGB444): all 19,200 pixels of the 160 x 120
  main area are the background code `7d5` except a 16 x 16 square of `f22` at
  rows 32..48, columns 22..38 (screen x on rows, y on columns, as
  `setAddrWindow` writes them). The codes are the palette's high nibbles; the
  panel expands them to 6 bits through the RGBSET table the base loads (the
  [RGB444 profile](../../docs/PXT-RGB444-COLOUR-PROFILE.md)).
- **Input** (`input`): a host 74HC165 on LATCH PB00 / CLOCK PB31 / DATA PB30;
  holding A counts one press and reads held, then LEFT reads held. Controls: no
  key, a register that never loads (all released).
- **HF2** (`hf2`): `DMESG` returns the runtime log; `READ_WORDS` reads the CF2
  magic; `DBG_GET_GLOBAL_STATE` plus `READ_WORDS` find the program's global and
  `WRITE_WORDS` changes it, which the program reports; `JDS_SEND` answers
  invalid state and `WRITE_FLASH_PAGE` invalid command (no JACDAC, no flashing in
  user space); `RESET_INTO_APP` reboots through `SYSRESETREQ`, the board
  re-enumerates and the global is back at its initial value.
- **SPI** (`spi`): SERCOM1 (MOSI PB23, MISO PB22, SCK PA17) with TX and RX DMA
  against a shift-register device: the replies are 0x3C then 0x5A. Controls: no
  device, MISO unrouted (the device still receives; replies read 0xFF).
- **ADC** (`adc`): A1 (PA05, AIN5) and A2 (PB08, AIN2) driven at 1650 mV and
  825 mV read 512 and 256 at 10 bits against VDDANA. Controls: 0 V, ADC0 generic
  clock off (the guest waits on RESRDY; conversions stop).
- **Bus timing**: bytes are paced by the live clock tree. Display SPI: CF2
  asks for 24 MHz, ZSPI's rounding gives BAUD 1 (25 MHz SCK), so the pixel
  stream of a full frame moves one byte per 38.4 CPU cycles. USART: SAMDSerial's
  arithmetic BAUD 64329 for 115200 on the 100 MHz core clock gives one 8N1 frame
  per 10,424.85 CPU cycles, back to back. I2C: 100 kHz on 48 MHz, allowing the
  driver's 215 ns of SCL rise, leaves BAUD = BAUDLOW = 230, and each operation
  takes its whole number of SCL periods (START plus nine bits per address, nine
  more for a read's first byte, nine per byte, one for STOP). Without a
  declared rise time a period is 470 clocks (1,175 CPU cycles, 102.1 kHz).
  `i2c-rise` declares the bus's 215 ns: 480.32 clocks (1,200.8 cycles), the
  driver's intended 100 kHz less the rise it rounded down (99.93 kHz).
  `i2c-stretch`: the BMP280 holds SCL 25 us after each byte it acknowledges and
  before each byte it sends, and each such byte takes exactly 3,000 cycles
  longer. Controls (`*-reclocked`): the host moves one SERCOM's core clock to
  another generator after the driver configured it (SPI and USART to 48 MHz,
  I2C to 8 MHz); the byte times scale by exactly that ratio while the driver's
  BAUD stays. `i2c-stretch-timeout`: a 100 ms hold meets the SCL low time-out
  the driver enables (LOWTOUTEN): each address ends after exactly 25 ms with
  LOWTOUT and BUSERR, the driver reports the error, and the read fails
  (`id=undefined`) rather than hanging.
- **Palette** (`palette`): sixteen bands; every pixel of the 160 x 120 area
  carries its band's palette colour as the RGB444 high nibbles. RGBSET is
  random after power-on and hardware reset (ST7735R v1.4, 10.1.22), so the base
  loads it; replayed through the engine's ST7735 colour decoder, every pixel is
  its band's code expanded to RGB666, (n << 2) | (n >> 2) per channel. Without
  the table (`colour-no-lut`, the original base) every pixel is undefined.
- **Melody** (`melody`): 440 Hz then 880 Hz, each for 150 ms within the
  mixer's block granularity, then no crossing of mid-scale for over 100 ms.
- **Levels and mixing** (`mix`): with OUTPUT_BITS 12 a voice at volume `vol`
  peaks at (32767 x V) >> 14 around mid-scale 2048, V = (255 x vol) >> 6
  (`addNote`, `sineTone`). Measured from the DAC conversions over whole periods:
  A4 at 255 reaches 2031, at 64 510; A4 and E5 together at 128 each reach 1020
  with nothing at 880 Hz and the sum inside the range; together at 255 the sum is
  held at +-2047 (2,545 clipped samples, distortion appearing at 880 Hz). The
  nominal analogue chain from datasheets and the schematic
  ([oracles](../../docs/SIMULATION-ORACLES.md)): 1636.7 mV peak at the pad at
  volume 255, gain 2.81 into the PAM8301, about 4.6 V peak across the speaker,
  beyond what a 3.7 V cell can drive; 2.3 V at volume 128.
- **Storage across reboots** (`store`): boot A from erased flash reads nothing
  and writes 41; boot B from what A left reads 41 and writes 42. Controls: the
  new value's data bytes erased while its metadata entry stays
  (`store-data-lost`: it reads neither value, silently, since RAFFS has no
  checksum), and no retained window (`store-erased`). Power cuts at commit
  boundaries are qualified in [pxt-storage-interruption](../../pxt-storage-interruption/README.md).
- **Deletion, compaction and clearing across power cuts** (`store-delete`,
  `store-gc`, `store-gc2`, `store-clear`, `store-scope`): `storage.ts` boots in
  turn from what the previous boot left: A seeds `bw-keep=7` and `bw-a=41`, B
  removes `bw-a`, C rewrites the 500-byte `bw-gc` forty times, which fills the
  16 KiB bank and compacts it (RAFFS erases the other bank, copies the live
  entries and their metadata, then writes the bank header last), D forty times
  more, compacting back into the first bank, and E calls `settings.clear()`,
  itself a compaction that keeps only the system keys. The host records every
  NVMCTRL erase and quad-word program as it is issued; for every cut, the flash
  as it stood after that many committed operations boots a fresh board, which
  reports what it reads. The deletion (3 operations), each compaction (73) and
  the clear (8) read the old state or the new one, switch once and never back,
  and `bw-keep` survives every cut it should. `store-scope` boots the same
  program under another name (`// name:`): `settings` sees a new program,
  clears the user settings and records the scope; cut anywhere in it (10
  operations), the renamed program reads nothing and the original one reads its
  settings until the clearing commits.
- **Inside one operation** (`store-delete`, `store-gc-torn`, `store-clear`):
  authored fault models, not silicon behaviour: an erase that reached only its
  first 1..15 pages of 16, and a program that reached only its first one, two
  or three of four words (programming clears bits). Every tear reads the old
  state or the new one. Torn erases of the new bank, its torn header and its
  torn copied metadata leave the old bank valid. A tear that splits a metadata
  entry (its name hash and offset programmed, its data offset and size still
  erased) is skipped by the owned RAFFS, which reads the key's previous entry;
  the original RAFFS follows its 0xFFFF offset and size and the next boot
  **HardFaults** (`store-delete-original`, two of nine tears). See
  [driver fixes](../../docs/DRIVER-FIXES.md).
- **Accelerometer** (`accel`): a host LIS3DH at 0x18 holding +0.25 g, -0.5 g,
  +1 g raises INT1 on PB14 at its configured data rate; CODAL configures it
  (50 Hz XYZ, data-ready on INT1, +-2 g) and the program reports 256, -512 and
  1024 mg (the board's NORTH_EAST_UP space leaves ENU unchanged). Controls: no
  sensor, INT1 never raised (both report zeros).
- **Accelerometer leaf** (`accel-leaf`): CODAL's MMA8453 (the MMA8653 driver at
  0x1C, INT1 active low on PB15), MPU6050 (0x68, INT active high on PB14) and
  FXOS8700 (0x1E, INT1 active low on PB13, hybrid accelerometer and
  magnetometer) on the board's I2C bus with TC0 as the system timer, against
  host sensors at +0.25 g, -0.5 g, +1 g (the FXOS8700's own axes rotated as its
  driver maps them to ENU). Each driver's configuration reaches its sensor and
  WHO_AM_I reads back (0x3A; 0x34 after the MPU6050 driver's shift; FXOS8700
  detected). All three report 256, -512, 1024 mg. The pinned MPU6050 driver
  reported -0.5 g as **3584 mg** (no sign extension into Sample3D's `int`); the
  owned core sign-extends (`accel-leaf-original` keeps the bug). Controls: no
  sensors (WHO_AM_I -1010, 0xFFFF, not detected; zeros), all INT lines held
  inactive (zeros).
- **Frame rate** (`frame`, measurement only): the interval between full-area
  frames in nominal cycles and the simulator's steps per second are printed;
  nothing is asserted.
- **TCC leaf** (`tcc-leaf`): `SAMDTCCTimer` on TCC0 at 1 MHz, MC0 re-armed from
  its interrupt: eight captures exactly 1,000 counts apart, all on `TCC0_1`.
  Control: `TCC0_1` masked (no match is taken).

## Limits

Step counts are guest instructions; cycle figures are nominal engine cycles. Bus
timing follows the register settings and nominal clock rates; I2C rise time is
a declared bus input and stretching a declared device input, not a measured
waveform; the SCL low time-out is the datasheet's earliest (25 ms); pad
electrical timing and oscillator tolerance are not modelled. Analogue levels
and stored colours are nominal datasheet values
([oracles](../../docs/SIMULATION-ORACLES.md)); no physical level or glass
colour is qualified. Flashing
over HF2 needs a bootloader, which this board deliberately omits. TCC system
timers need the owned runtime profile ([TCC system timer](../../docs/PXT-TCC-SYSTEM-TIMER.md));
PDM is not selected on this board.
