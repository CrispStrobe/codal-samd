# Remaining no-ASF native SAMD consumer lanes

Refresh current heads, PRs and ownership before implementation. Preserve original
failures and exact qualification boundaries. The generated configuration keeps
USB1/WebUSB1, 120 MHz, 192 KiB RAM, a 10 KiB stack and vectors at `0x4000`.
The explicit USB diagnostic leaf is an intermediate boundary; it does not run a
native Arcade program or HF2. Existing finite storage consumers keep their own
qualified engine selection until a reviewed pin change and affected guest pass.

## A. Complete the selected source and header closure

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

Resolve actual `ZPin`, `SAMDEIC`, pin mapping and PWM dependencies selected by
`platform.h`, button and light consumers. Preserve required Pin APIs and actual
callers. Independently authored register definitions need source-backed offsets,
field masks and clock/pin admission. Do not substitute guessed object layouts or
successful return stubs. Qualify real pin reads/writes and a natural enabled EIC
handler with disabled-clock/pin/IRQ controls. Claim analog behavior only after
separate actual evidence. Compile-only unused exports remain unqualified.

## C. Serial, SPI, I2C and DMA adapters

Resolve `SAMDSerial`, `ZSPI`, `ZI2C`, `ZSingleWireSerial` and `SAMDDMAC` without
ASF HAL sources/types. Preserve real framework callers and the existing finite
DMA/TXC/CS/callback and allocation-domain contracts. Bind source-required SERCOM
clock/pin/reset, DMA descriptors, completion/error flags and natural IRQ delivery.
Qualify actual transfers and exact missing-completion/IRQ/buffer controls. Existing
storage or synthetic register tests cannot qualify these newly selected adapters.
Keep natural stale IRQ/timeouts and interrupted operations separately scoped.

## D. Timer and audio dependencies

Resolve selected `SAMDTCCTimer`, `SAMDTCTimer`, `SAMDDAC`, `SAMDPDM`, Mixer and
Synthesizer source/API requirements without disabling generated features. The
service fixture's DWT/SysTick adapter supplies a declared nominal core service;
it does not prove physical SAMD timer/audio operation. Bind actual selected
callers, interrupt and memory paths. Separate compile closure from active timer,
DMA/audio behavior, analog output and timing claims.

## E. Genuine native PXT program and authored CF2

After a closed permitted base/template exists, use the admitted Arcade 4.2.1
target/worker and `samd51adafruit` to compile an empty public `main.ts`, supplying
that new base through `extinfo.hexinfo`. Bind target, worker, request, template
hash and patched executable bytes. The natural `PXT_MAIN` calls `pxt::start()`;
its unused `uBit.init()` branch does not establish a board-constructor execution.
A `PXT_SHIMS_BEGIN` sentinel alone is not a native program.

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
