# Driver fixes

Bugs the native cases found in the drivers the no-ASF board runs, and where each
is fixed. Upstream sources (codal-core, the pxt-common-packages files in the
native request) stay pinned and byte-bound; the owned build applies reviewed,
exact adaptations in [build.py](../no-asf-board/fullboard/build.py)
(`CORE_ADAPTATIONS`, `REQUEST_FIXES`), each tied to the original's sha256 and
reported in `build-report.json`. Our own sources are fixed in place. Every fix
has an omission control that still shows the bug.

| Bug | Fixed in | Case | Omission control |
| --- | --- | --- | --- |
| codal-core `MPU6050::requestUpdate` assembles each axis as `(high << 8) \| low` into Sample3D's 32-bit `int` without sign extension: -0.5 g read +3584 mg (every negative axis 4096 mg too high; the gyro likewise). | core adaptation: `(int16_t)` on all six axes | `accel-leaf` | `accel-leaf-original` (`--core-fixes deallocator-only`) |
| RAFFS follows a metadata entry torn between its two words (name hash and offset programmed, data offset and size still erased, 0xFFFF): the next boot HardFaults. | request fix: `validEntry` skips an entry that is erased, points outside its bank or names a string with no NUL inside it, in lookup, compaction (before the compaction's name filter reads the name) and listing; the key's previous entry is used | `store-delete`, `store-gc-torn`, `store-clear`, `store-gc-garbage` | `store-delete-original` (`--request-fixes none`) |
| pxt `accelhw.cpp` probes the MPU6050 at 8-bit address 0x68 (7-bit 0x34) while the driver uses `0x68 << 1`. Inactive here (`PXT_SUPPORT_MPU6050` is 0). | request fix: the probe uses `0x68 << 1` | none (not compiled on this board) | none |
| pxt `screen.cpp` sends RGB444 pixels without loading RGBSET, whose contents are random after power-on and hardware reset (ST7735R v1.4, 10.1.22). | colour profile `rgb444-lut`, now the default for request bases: the 128-byte table expands 4-bit values to 6 bits in entries 0..15 of R, G and B (datasheet 9.18.2) | `display`, `palette`, `colour` | `colour-no-lut` (`--colour-profile original`) |
| `ZSPI::dmaTransferComplete` (taken on the TX channel) waited for TXC, then drained RXDATA while RXC was set. TXC and the last RXC rise together, and the RX DMA channel takes that byte only once the DMAC services its request: draining first stole it, so the last received byte never reached the buffer. | `src/ZSPI.cpp`: with RX DMA active, wait (bounded) for the RX channel to finish before draining | `spi-dma-latency` (the DMAC services requests 16 cycles late; before the fix the first `spiWrite` read 0 instead of 0x3C) | none |
| The SPI support layer rounded the SERCOM BAUD to the nearest value, so a request could run faster than asked: CF2's 24 MHz display clock ran at 25 MHz. The samd-peripherals original treats the request as a ceiling. | `no-asf-board/support/sercom.c`: `ceil(f_core / (2 f)) - 1` | `display` (BAUD 2, 14.667 MHz, 65.455 CPU cycles per byte) | none |
| The no-ASF clock setup leaves the DFLL48M, which clocks USB, in open loop (45.8-49.3 MHz, DS60001507N table 54-50), outside full speed's +-0.25 %. | `src/SAMDUSB.cpp`: once USB is enabled, DFLLMUL.MUL = 48000 and DFLLCTRLB = MODE \| USBCRM \| CCDIS (28.6.4) | `usb-dfll-error` (+16875 ppm declared: enumerates) | `usb-dfll-open-loop` (the host forces open loop: no transaction completes) |
| After a reset during compaction the block holds arbitrary bytes (DS60001507N 25.6.13). With only the torn-entry fix, a garbage entry (name offset 0xf468 in a 16 KiB bank) still faulted: `tryGC` passed its name to the settings filter (`isSystem`) before checking the entry. | the same request fix, extended as above | `store-gc-garbage` (every operation of a compaction with its 8 KiB block randomised; 71 of 72 boots read, one HardFaulted before) | none (the garbage outcome needs the extended fix) |
| pxt `screen.cpp` runs the ST7735 at CF2's DISPLAY_CFG2 frequency, 24 MHz, above the panel's 15.15 MHz (ST7735R v1.4, 8.4: write cycle at least 66 ns). | request fix (with the colour profile, in the same file): an ST7735 is clamped to 14.75 MHz, leaving room for an open-loop DFLL (up to 2.71 % fast); DPLL1 at 88 MHz and ZSPI's ceiling give 14.667 MHz | `display` (65.455 CPU cycles per byte) | `display-original` (`--request-fixes none`: 43.636 cycles, 22 MHz) |
| ZSPI left SCK and MOSI at the default drive strength, whose edges may take 40 ns into 30 pF (DS60001507N table 54-14): the whole half period at 12.5 MHz. | `src/ZSPI.cpp`: PINCFG.DRVSTR on both outputs (10 ns) | `display` (`NATIVE_ORACLE` pads) | none |
| `SAMDSerial` and `ZSingleWireSerial` used a literal 100 MHz even when the owned clock changed: at 88 MHz, UART frames took 11,847.273 CPU cycles instead of 10,422.273. | use `SERCOM_CORE_FREQUENCY` when the support layer declares it; retain existing ASF clock selection otherwise | `uart`, `uart-reclocked` in both profiles; single-wire path compile only | initial failing local UART observations retained |
| `ZI2C::reset` disabled the master (SWRST) while the STOP of the NACKed retry was still on the bus, cutting it off. | `src/ZI2C.cpp`: wait, bounded, for BUSSTATE idle first | `i2c` (eight operations, the last STOP completes) | none |
| `wait_flag` treated MB/SB as success even with INTFLAG.ERROR set (a bus error or SCL low time-out still raises MB/SB, DS60001507F 36.6.2.4.6). | `no-asf-board/support/i2c_m_sync.c`: check ERROR after the flag | `i2c-stretch-timeout` | none |
| The I2C support layer never enabled the SCL low time-out, and its poll budget (about 10 ms) gave up before a 25-35 ms time-out could end a stretched transfer. | `i2c_m_sync.c`: `LOWTOUTEN`, and a poll budget beyond 35 ms | `i2c-stretch-timeout` (the address ends after 25 ms with LOWTOUT) | none |

The owner-approved [RAFFS v2 adaptation](RAFFS-V2-CHECKSUM.md) appends a CRC32
bound to each key, payload length and payload. Lookup, compaction and directory
iteration reject corrupt entries. `store-data-lost` and `store-data-bitflip`
recover the preceding valid value; `store-data-lost-original` retains the
unadapted failure. New firmware reads legacy banks and migrates them through an
atomic compaction before writing. Both native profiles pass the corruption and all 13 migration-cut cases.
