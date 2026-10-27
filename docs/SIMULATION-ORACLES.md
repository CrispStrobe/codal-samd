# Oracles for what simulation cannot measure

The native cases observe the guest's bus traffic and the models' registers.
Some questions the simulator cannot answer by itself: what the panel shows,
what voltage reaches the speaker, what an interrupted flash operation leaves.
This page records which of them a datasheet, the board schematic or another
simulator settles, and which still need hardware.

## Settled by datasheet and schematic

| Question | Oracle | Answer | Where it is checked |
| --- | --- | --- | --- |
| Panel colour on the default path | ST7735R datasheet v1.4, 10.1.22 (RGBSET, 2Dh): default after power-on **random**, after H/W reset **random**, after S/W reset the table is kept. 9.18.2: 4K colour uses parameters 1-16 (R), 33-48 (G), 97-112 (B). The schematic's LCD1 is a JDT1800 module, an ST7735R panel. | A driver that never loads RGBSET shows colours the controller leaves undefined. The owned base now loads the table by default; each RGB444 code n is stored as (n << 2) \| (n >> 2). | `palette`, `display`, `colour`; control `colour-no-lut` |
| DAC pad level | SAMD51 DS60001507F 47.6.2: VOUT = DATA / 4095 x VREF. CODAL's SAMDDAC selects REFSEL 0 (VREFA). PyBadge schematic: PA03/VREFA and VDDANA are on the AVCC net, the 3.3 V rail through ferrite FB1. | Full scale 3.3 V; a voice at volume 255 peaks 1636.7 mV around mid-scale, at 64 410 mV, at 128 821 mV. | engine `Samd51Dac::output_millivolts`; `mix` |
| Speaker level | Schematic: PA02 -> C9 0.1 uF -> R12 47 k -> PAM8301 IN. PAM8301 datasheet: gain 2 RF / RI, RF 80 k internal, RI 10 k internal plus the external resistor; high-pass 1 / (2 pi RI CI); supply 2.5-5.5 V (V+, battery or USB). | Gain 2.81 (RI 57 k), corner 27.9 Hz. At volume 255 the bridge output would need about 4.6 V peak, beyond a 3.7 V cell, so a battery-powered PyBadge clips there; at 128 (2.3 V) it stays inside. | `mix` (`NATIVE_ORACLE`) |
| I2C SCL period | DS60001507F 36.6.2.4.1: f_SCL = f_core / (10 + BAUD + BAUDLOW + f_core t_rise). The driver allows 215 ns of rise. | With the rise the driver allows, 480.32 core clocks: 99.93 kHz. | `i2c-rise` |
| SCL low time-out | DS60001507F 36.6.2.4.6: 25-35 ms; MB/SB as normal, STOP sent, LOWTOUT and BUSERR. | The model takes 25 ms. | `i2c-stretch-timeout` |
| I2C bus compliance | NXP UM10204, standard mode: f_SCL <= 100 kHz, t_LOW >= 4.7 us, t_HIGH >= 4.0 us, t_r <= 1000 ns. DS60001507N 36.6.2.4.1: t_LOW = (BAUDLOW + 5) / f, t_HIGH = (BAUD + 5) / f. | With the BAUD the driver writes (230/230 at 48 MHz) and 215 ns of rise: t_LOW = t_HIGH = 4.896 us, f_SCL = 99.93 kHz, compliant. On an open-loop DFLL at its fast end (49.3 MHz, below) f_SCL becomes 102.58 kHz, 2.6 % over the standard-mode maximum; t_LOW still holds. | `i2c-rise` (`NATIVE_ORACLE`) |
| Display serial clock | ST7735R v1.4, 8.4: write cycle TSCYCW >= 66 ns (15.15 MHz), SCL high/low >= 15 ns. | CF2 DISPLAY_CFG2 asks for 24 MHz; with the request as a ceiling ZSPI runs 16.67 MHz (it ran 25 MHz before the fix). Both exceed the panel's datasheet limit: the board configuration's choice, which shipping boards also make. | `display` |
| Clock accuracy and USB | DS60001507N table 54-50: the DFLL48M in open loop is 45.8-49.3 MHz over -40..85 C, 47.2-48.81 MHz over 0..60 C; 28.6.4: USB clock recovery (DFLLCTRLB MODE and USBCRM, DFLLMUL.MUL 48000) locks it to the host's 1 kHz start of frame. USB 2.0 full speed needs +-0.25 %. | Every clock on this board derives from the DFLL. Open loop is up to 7x outside USB's tolerance at room temperature; SAMDUSB now locks the DFLL once USB is enabled. (The original ASF configuration and the Adafruit Arduino core set USBCRM without MODE, which by the datasheet stays open loop.) | `usb-dfll-error`, control `usb-dfll-open-loop` |
| Serial frame timing | DS60001507N 33.6.2.3.1, table 33-3: in arithmetic mode a frame takes a whole number of clocks, floor(f_ref (D + S) / f_BAUD) + 1. | 115200 from 100 MHz (BAUD 64329): 8688 clocks, 10,425.6 CPU cycles per 8N1 frame; the baud error is 1 - 115200 / 115108.5 = -0.08 % before any clock error. | `uart`, `uart-reclocked` |
| DAC accuracy | DS60001507N table 54-30 (single-ended): INL +-4.0 LSB, DNL +-6.1 LSB, gain error +-1.5 % FSR, offset +-21 mV (max). | The speaker's AC level moves by the gain error only: 4.59 V +-1.5 % at volume 255, still beyond a 3.7 V cell. | `mix` |
| ADC accuracy | DS60001507N table 54-26 (single-ended, VREF = VDDANA): gain +-0.30 % FSR, offset -18..+7 mV, total unadjusted error +-18.3 LSB (12-bit). | The `adc` case's ideal 512 (1650 mV at 10 bits) has a silicon band of about +-5 codes (TUE / 4). | `adc` |

These are nominal values: no gain or offset error, tolerance, load, speaker
impedance or amplifier non-linearity. They settle what the parts should do by
their own specification, not what one board does.

## Other simulators

Renode 1.16.1 ships an `atsamd51g19a` platform: the Cortex-M4F core, memory, one
SERCOM USART (SERCOM3) and stubs for a GCLK and PAC register. It models no DAC,
display, NVMCTRL, TC/TCC, EIC, I2C or SPI, so it cannot answer any question on
this page; it could cross-check CPU execution and USART bytes only. QEMU has no
SAMD51 machine. Neither is used as an oracle here.

## Still needs hardware

- **Flash power loss**: which bits a real interrupted quad-word program or
  block erase leaves. The datasheet (DS60001507N 25.6.13) only bounds it: the
  content of the block is "unpredictable", and the block must be erased before
  it is read. `store-gc-garbage` takes that bound literally (the whole block
  arbitrary, for every operation of a compaction); the partial erases and torn
  programs in `store-delete`, `store-gc-torn` and `store-clear` are authored
  fault models within it. No simulator models the real bit pattern.
- **Glass colour**: the panel's gamma, the module's aperture and backlight, and
  colour matching. The datasheet settles what the controller stores, not what
  the eye sees.
- **Analogue accuracy on one board**: where within the datasheet's DAC and
  ADC bands a given part sits, the ADC front end with this board's source
  impedances, and the speaker's acoustic output.
- **Oscillator drift on one board**: where within the DFLL's open-loop range
  a given part runs, and how it moves with temperature.
