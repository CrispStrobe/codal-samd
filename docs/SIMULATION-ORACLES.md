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
  block erase leaves. The datasheet specifies no behaviour for an interrupted
  operation, and no simulator models it. The partial erases and torn programs
  in `store-delete`, `store-gc-torn` and `store-clear` are authored fault models.
- **Glass colour**: the panel's gamma, the module's aperture and backlight, and
  colour matching. The datasheet settles what the controller stores, not what
  the eye sees.
- **Analogue accuracy**: DAC gain, offset and linearity on this board, ADC
  front-end behaviour, and the speaker's acoustic output.
