# Remaining no-ASF native SAMD consumer lanes

Refresh current heads, PRs and ownership before implementation. Preserve original
failures and exact qualification boundaries. The generated configuration keeps
USB1/WebUSB1, 120 MHz, 192 KiB RAM, a 10 KiB stack and vectors at `0x4000`.
The explicit USB diagnostic leaf is an intermediate boundary; it does not run a
native Arcade program or HF2. Existing finite storage consumers keep their own
qualified engine selection until a reviewed pin change and affected guest pass.

## A. Complete the selected source and header closure

Status: compile and link closure implemented in the
[full-board base](PXT-NO-ASF-FULLBOARD-BASE.md), with the
[core migration review](PXT-NO-ASF-CORE-MIGRATION.md). The scope below is kept.

Inputs: `profiles/pxt-framework-sources.json` (64 public framework projections),
`profiles/pxt-generated-board-profile.json`, the original configuration producer,
`pxt.h`, `pxtbase.h`, `platform.h`, and the no-ASF USB leaf's admitted compiler
closure. Bind actual generated `main.cpp`, `pointers.cpp`, `pxtconfig.h` and
function metadata to the selected generator. Keep the historical inventories
unchanged; they do not describe a new compiler selection.

Run the selected ARM compiler with complete dependencies for each affected
translation unit. Classify inactive conditional includes separately from required
inputs. Each required header/type/implementation must bind permitted source or
one of the replacement lanes below. Reject ASF4, vendor-restricted device headers
and historical samd-peripherals selection. Retain selected library members,
component notices, exact compiler/ABI and include order. A successful declaration
parse alone does not establish class layout, source compatibility or linking.
Review the historical core `312ae57e` to `2ed19d64` API migration explicitly.

## B. Pin, external interrupt and PWM APIs

Status: implemented in the [native cases](../no-asf-board/native/README.md):
digital reads through a host jumper, rise events handled by the natural EIC
interrupt with pin-mux, clock and NVIC controls, and PWM measured on the LED pad
(TC4 WO1) with routing and stop controls. Analog input remains open.

Resolve actual `ZPin`, `SAMDEIC`, pin mapping and PWM dependencies selected by
`platform.h`, button and light consumers. Preserve required Pin APIs and actual
callers. Independently authored register definitions need source-backed offsets,
field masks and clock/pin admission. Do not substitute guessed object layouts or
successful return stubs. Qualify real pin reads/writes and a natural enabled EIC
handler with disabled-clock/pin/IRQ controls. Claim analog behavior only after
separate actual evidence. Compile-only unused exports remain unqualified.

## C. Serial, SPI, I2C and DMA adapters

Status: implemented in the [native cases](../no-asf-board/native/README.md):
byte-level display SPI (the ST7735 command sequence and a full frame) with CS
and MOSI controls, I2C against a BMP280 with device, clock and SDA-routing
controls, and USART TX/RX on SERCOM5 with interrupt-per-byte counts. The USART
case runs on the serial-request base and needed `SAMDSerial` to re-arm the DRE
interrupt after each byte. SPI receive and bus timing remain open.

Resolve `SAMDSerial`, `ZSPI`, `ZI2C`, `ZSingleWireSerial` and `SAMDDMAC` without
ASF HAL sources/types. Preserve real framework callers and the existing finite
DMA/TXC/CS/callback and allocation-domain contracts. Bind source-required SERCOM
clock/pin/reset, DMA descriptors, completion/error flags and natural IRQ delivery.
Qualify actual transfers and exact missing-completion/IRQ/buffer controls. Existing
storage or synthetic register tests cannot qualify these newly selected adapters.
Keep natural stale IRQ/timeouts and interrupted operations separately scoped.

## D. Timer and audio dependencies

Status: the CODAL system timer runs on TC0; DAC audio (TC3-paced DMA into DAC
DATA0, exact sample interval and tone) and `SAMDTCCTimer` (TCC leaf: compare
interrupts on the per-channel lines) are implemented in the
[native cases](../no-asf-board/native/README.md). The owned
[MakeCode discriminator patch](PXT-TCC-SYSTEM-TIMER.md) now qualifies TCC0 as
the runtime system timer in both scheduler profiles, with original-source panic
and masked-IRQ controls. PDM is not selected on this board; see [TODO](../TODO.md).

Resolve selected `SAMDTCCTimer`, `SAMDTCTimer`, `SAMDDAC`, `SAMDPDM`, Mixer and
Synthesizer source/API requirements without disabling generated features. The
service fixture's DWT/SysTick adapter supplies a declared nominal core service;
it does not prove physical SAMD timer/audio operation. Bind actual selected
callers, interrupt and memory paths. Separate compile closure from active timer,
DMA/audio behavior, analog output and timing claims.

## E. Genuine native PXT program and authored CF2

Status: implemented. An empty `main.ts` compiles natively onto the owned base;
the [native case](../no-asf-board/native/README.md) boots it with the authored CF2
boot region and observes actual runtime CF2 reads.

After a closed permitted base/template exists, use the admitted Arcade 4.2.1
target/worker and `samd51adafruit` to compile an empty public `main.ts`, supplying
that new base through `extinfo.hexinfo`. Bind target, worker, request, template
hash and patched executable bytes. The natural `PXT_MAIN` calls `pxt::start()`;
its unused `uBit.init()` branch does not establish a board-constructor execution.
A `PXT_SHIMS_BEGIN` sentinel alone is not a native program.

The admitted payload pair is `target.json` (Arcade 4.2.1 / PXT 13.2.1,
target commit `2d7d59b5226e8a1a1e05e8042dd9f5248ed68fea`) with SHA-256
`6f35aadf1456436b3318d2f2339cc3d1add6e6c3b0bf8566f86d7f7bb4f7722e`, and
`pxtworker.js` with SHA-256
`1d9e73fd4bc0c0939b938395537ff1934b398654f9026f88e9293c32ea5acb0e`.
These are payload byte identities, not Git object IDs; a latest-version download
does not inherit their admission. Bind both exact inputs in the new compile
receipt before executing the worker.

The [pinned Lite compiler-pipeline reference](https://github.com/CrispStrobe/brickwright-lite/blob/6a7027e117a6698866516f52feca259127c80c7d/overlay/scratch-gui/src/lib/bw-makecode/pxt-runtime.js)
shows `simpleInstallPackagesAsync`, `setHwVariant`,
`simpleGetCompileOptionsAsync` with native mode and the actual compiler service
operation. Use `samd51adafruit` and supply the newly admitted own base through
`extinfo.hexinfo`; preserve the request, worker/target identities, template hash
and patched program. The reference is a pipeline input, not a qualification of
that future no-ASF base or native execution.

The authored CF2 source/data component is already merged in #83. Its profile
binds public PyBadge configuration from `adafruit/uf2-samdx1` commit
`d4dc92889759c0c551683420e24c5ef535ac303e`,
`boards/arcade_pybadge/board_config.h`, retaining the reviewed notice. Reuse
`profiles/pxt-cf2-authored-metadata.json` and `no-asf-board/cf2/metadata.py`;
source/data controls alone did not execute ARM runtime or USB consumers.
Select its authored fragments in the new permitted base and observe actual
runtime CF2 reads through the 16-byte UF2 information layout at `0x3ff0`.
Check pointer, magics, entry count and bounds, with specific missing/corrupt
metadata controls. Do not acquire, bundle, patch or execute a vendor bootloader
image.

## F. Natural startup, enabled USB and HF2 qualification

Status: implemented in the [native cases](../no-asf-board/native/README.md):
natural boot, USB enumeration, HF2 `BININFO`/`INFO`/`DMESG`/`READ_WORDS`/
`WRITE_WORDS`, debugger global state, a reboot over `RESET_INTO_APP` with
re-enumeration, rendered frames and button input, with controls. Flashing over
HF2 needs a bootloader, which this board omits; performance remains open.

Freeze independent readers before the actual campaign. In both scheduler
profiles, qualify genuine reset/data/BSS/constructors, vectors at `0x4000`,
CF2 reads, native program entry and required scheduler/services/IRQs. Exercise
actual PXT/HF2 sources through typed host requests and real SRAM descriptors;
compare complete bytes produced by the guest. Preserve the existing unsupported
WebUSB landing-page response. An authored echo interface is not HF2 evidence.

Require specific nonvacuous configuration, completion, IRQ, descriptor and
interface-discovery mutations. Acquire each admitted official original once and
audit source/image/model/event joins without guest replay. Merge only with all
enabled final-head checks successful and reviewed/landed trees and parents equal.
Complete peripheral reset, physical accuracy, frames/input/debugger operation,
representative native/WASM performance and installed adoption require their own
actual workloads and qualification; they are not implied by empty-program boot.

## G. USB endpoint initialization and repeated enumeration

Status: implemented in the [USB leaf](../no-asf-board/usb/README.md#endpoint-lifetime-and-deallocation--lane-g)
with stale-heap, repeated-reset and deallocator-observation cases. The findings
below are kept as the original scope.

Review `UsbEndpointIn::UsbEndpointIn()` and `UsbEndpointIn::write()` in the
selected `src/SAMDUSB.cpp`: the constructor currently assigns endpoint and flags
but leaves `wLength` uninitialized; `write()` reads it before clearing it.
This is a source finding, not a retained nonzero-heap failure reproduction.
The finite first-reset leaf uses a fresh declared RAM state and does not qualify
heap reuse, repeated bus reset or every endpoint lifetime.

First reproduce the effect with actual driver callers and a justified nonzero
allocation state, preserving normal enumeration and echo. Do not patch private
object layouts from the host or substitute a successful transfer. Then initialize
required members explicitly and require an omission mutation to fail at the
specific transfer boundary. Qualify the changed actual driver in both profiles,
including repeated reset/re-enumeration and original first-boot positive cases,
before broadening its lifetime claim. Keep complete peripheral reset separate.

The pinned core's `CodalUSB::sendConfig()` also allocates its descriptor buffer
with `new[]` and frees it with scalar `delete`; the selected compiler reported
that mismatch. Preserve the original diagnostic. Review an explicit owned patch
to use the matching deallocator, retain the pinned source/notice and patch bytes,
and observe the actual caller/deallocation path. A mutation restoring the wrong
operation must be detected even if the protocol bytes still happen to agree.
Do not suppress the compiler warning or infer general C++ lifetime correctness
from the finite USB transcript.

## H. Open work after lanes A–G

Lanes A–G are implemented. Each task below is unclaimed and states what decides
it, where to start, and what counts as done. Before starting one: refresh
`master`, check open PRs here and in `CrispStrobe/labwired-core` for overlap, and
read [DRIVER-FIXES](DRIVER-FIXES.md), [SIMULATION-ORACLES](SIMULATION-ORACLES.md)
and the [native cases README](../no-asf-board/native/README.md). Every native
change runs in both scheduler profiles (`default`, `event-scheduler`); the
fullboard workflow does that in CI. Driver fixes go into this repository's own
sources or `build.py` adaptations, never as pull requests to upstream projects.

### H1. Faster in-spec display clock (approved 2026-10-28; qualification pending)

**Approval.** The owner approved this retune on 2026-10-28. The implementation
now uses DPLL1 at 88 MHz and a 14.75 MHz clamp; both-profile native and fullboard
qualification is pending. The following records the original decision and plan.

**Previous state.** CF2 `DISPLAY_CFG2` asks for 24 MHz; the ST7735R allows at most
15.15 MHz (datasheet v1.4, 8.4). `build.py` (`DISPLAY_CLOCK`) clamps an ST7735 to
14 MHz and ZSPI's ceiling divider turns that into BAUD 3, 12.5 MHz SCK, from the
100 MHz SERCOM clock: 76.8 CPU cycles per byte, about 41.7 frames a second (24 MHz
gave about 50). The SERCOM clock is GCLK2, DPLL1 at 100 MHz, referenced to DFLL/48;
without a USB host the DFLL runs open loop, up to 2.71 % fast (DS60001507N table
54-50). The in-spec ceiling is therefore 15.15 / 1.0271 = 14.75 MHz. Integer
dividers from 100 MHz reach nothing between 12.5 and 16.67 MHz.

**Decision needed from the owner:** whether about 47 frames a second is worth
retuning DPLL1. Do not start without it.

**Change, if approved.**
1. `no-asf-board/support/samd/clocks.c`: `enable_dpll(&OSCCTRL->Dpll[1], …, 88)`
   and the `nominal[]` entry for generator 2; `samd/clocks.h`:
   `SERCOM_CORE_FREQUENCY 88000000u`, the `CLK_GEN_100MHZ`/`CLOCK_100MHZ` names
   and the header comment. GCLK2 must stay within fGCLK_SERCOMx_CORE, 100 MHz
   (table 54-7). Grep for every other user of generator 2 first (`CLK_GEN_100MHZ`,
   `SERCOM_CORE_*`, a literal 2 passed to `turn_on_clocks`); I2C is on the 48 MHz
   generator (`i2c_m_sync.c`) and does not move.
2. `build.py` `DISPLAY_CLOCK`: clamp in hertz at the derived ceiling,
   `spi->setFrequency(... > 14750000 ? 14750000 : ...)`, because a 14 MHz request
   gives BAUD 3 (11 MHz) at 88 MHz. 14.75 MHz gives BAUD 2, 14.67 MHz, at 88 MHz
   (15.07 MHz on a fast DFLL, in spec) and still BAUD 3 at 100 MHz. Update
   `display_clock_clamp_mhz` in the report.
3. Re-derive, from the datasheet formulas already quoted in `guest.rs`, every
   case expectation that assumes 100 MHz: `display` (8 bits at 14.67 MHz = 65.45
   CPU cycles per byte; adjust the limit), `display-original`, `display-reclocked`,
   `spi`, `spi-dma-latency`, `uart` and `uart-reclocked` (SAMDSerial's arithmetic
   BAUD from 88 MHz and the floor + 1 frame rule, 33.6.2.3.1), and the `pads`
   oracle's half period. Re-measure `frame` and record the frame rate.
4. Update SIMULATION-ORACLES (display serial clock, serial frame timing),
   DRIVER-FIXES (display clamp row) and TODO.

**Done when** the fullboard workflow passes in both profiles with the new
expectations, `display-original` still shows the over-limit original, and the
oracle rows state the new margin.

### H2. Hardware observations (needs a PyBadge and instruments)

The simulator settles what the parts should do by specification; these need one
real board. Record each capture with board revision, part date code, supply, and
temperature; compare against the named case and update SIMULATION-ORACLES ("Still
needs hardware") with measured values and their tolerance.

- **Flash power loss.** Interrupt an erase and a quad-word program at controlled
  points (a GPIO-triggered supply cut through a MOSFET, or a debug-probe reset at
  a counted instruction) and dump the 8 KiB block. Compare with the authored
  fault models in `store-delete`, `store-gc-torn` and `store-clear` and the
  arbitrary-block bound in `store-gc-garbage`. A pattern the cases do not cover
  becomes a new case.
- **Glass colour.** Photograph or colorimeter-read the sixteen `palette` codes
  on the JDT1800 module with the RGB444 table loaded and without it
  (`colour-no-lut`). Records gamma and backlight, not controller state.
- **Analogue levels.** DAC pad voltage at known DATA values (gain and offset
  within table 54-30), the speaker node through C9/R12 at volumes 64, 128 and 255
  on battery and USB (the `mix` oracle predicts clipping at 255 on a 3.7 V cell),
  and ADC codes for a known source (table 54-26, the `adc` case).
- **Oscillator drift.** Measure the DFLL open loop (GCLK output on a pin) across
  temperature, and confirm the USB clock recovery lock (`usb-dfll-error`).

### H3. RAFFS data checksum (approved 2026-10-28; qualification pending)

The owner approved the format change on 2026-10-28. The implementation is a
sha-bound `REQUEST_FIXES` adaptation, preserving `--request-fixes none`.
[RAFFS v2](RAFFS-V2-CHECKSUM.md) appends CRC32 to payloads, reads legacy banks,
and migrates them atomically before a write. Host CRC/corrupt-entry tests and
the fullboard compile pass. Native corruption, old-bank migration and existing
interruption cases must pass in both scheduler profiles before closing H3.

### H4. SPI slave mode (only when a board needs it)

No consumer on the PyBadge: it has no SPI peer. When a board with one is added:
model the SERCOM SPI slave in labwired-core (its `sam_sercom_spi` model lists
slave mode as not modelled; see that repository's `docs/engineering/samd51-board-models.md`),
then select the driver path in ZSPI, and qualify a transfer with a host master
plus an unrouted-pad control, as the master `spi` case does.

### H5. Not selected on this board

- **PDM**: `SAMDPDM` compiles but no microphone package or pin exists on this
  board; qualify it only on a board with a microphone.
- **HF2 flashing and JACDAC**: flashing over HF2 needs a bootloader
  (`USB_HANDOVER`), which this board omits by design. Do not add a vendor
  bootloader image.
- **GhostFAT**: codal-core's `GhostFAT` reads the legacy `UF2_INFO_ADDR` (0x3fd0),
  not PXT's 0x3ffc pointer. Unused here; fix it as a `CORE_ADAPTATIONS` entry only
  if a board selects GhostFAT.

### H6. Engine drift acknowledgements (labwired-core, operator decision)

labwired-core's `validation/manifest.yaml` carries simulator-drift
acknowledgements renewed with owner approval on 2026-10-28 through 2026-11-27
for seven boards (nrf52840,
seeed-xiao-nrf52840-sense, stm32h563, nucleo-l476rg, nucleo-l073rz, stm32f103,
stm32f407). None is a SAMD board. Renewal is in labwired-core #267; its manifest generation,
check, drift gate and 27 generator tests pass. CI/merge is pending. After expiry
the validation-status gate fails.
Renewing is the operator's decision, never an agent's. The manifest header lists
the three options: re-capture on silicon; renew (new `drift_ack` date, then
`python3 scripts/generate_validation_status.py --write-ack-digests`); or set
`drift_ack_expires` with a note. Then regenerate `docs/boards/VALIDATION_STATUS.md`
(`python3 scripts/generate_validation_status.py`, then `--check`).
