# Actual no-ASF USB core/driver leaf

This authored diagnostic application compiles the real pinned CODAL USB core
and MIT SAMD USB driver. The driver has exactly one include adapted from the
vendor header to independently authored register/API declarations. No ASF4,
vendor device header or samd-peripherals implementation is selected.
The owned aligned allocator is copied with exactly one quoted platform include
changed to an angle include, so it selects this USB1 platform consistently.
Its historical service source and USB-disabled profile remain unchanged.

All translation units use the reproduced generated configuration with USB1 and
WebUSB1,120MHz,192KiB RAM and a10KiB stack. The leaf uses Cortex-M4/thumb/softfp/
fpv4-sp-d16 with `-O0`; this is deliberately separate from the historical `-O2`
board build. Full C/C++ compiler dependency output and two assembly units are
retained. The source-built image has real vectors at0x4000 and an authored
8-byte reset forwarding table at0. It includes no vendor bootloader or CF2.
The declared calibration input encodes documented defaults9/25/6; it is an
authored test input, never a capture from a physical device.

The application registers its own vendor echo interface through actual
`CodalUSBInterface` APIs. Actual core code constructs configuration, BOS and
MS OS descriptors. Actual SAMD code initializes clocks/pinmux/calibration,
receives setup packets, moves SRAM data and handles NVIC interrupts. The echo
program validates64 received bytes and finishes66 checks after its real IN API
returns `DEVICE_OK`.
Bulk mode must transmit its manual terminating ZLP; interrupt mode suppresses it.

One hosted producer compiles the ARM sources and explicit source variants once.
The same admitted images execute in fresh test processes for default and event
scheduler engines. Required cases are:

- Bulk and interrupt: actual data/BSS initialization, relocated vectors, runtime
  constructors, enumeration/address/configuration, core WebUSB BOS/MS OS data,
  the existing unsupported landing-page stall and subsequent recovery, and a
  full-size OUT/IN echo through actual driver/core code.
- Missing NVIC80 enable: the actual driver leaves reset pending while its source
  polls `isInitialised()` between WFI entries. Require 128 actual WFI entries,
  with the pending/disabled IRQ, no handler or endpoint constructors, unmasked
  thread mode and unchanged startup status checked at every intervening step.
  A frozen program counter is not a source polling loop; generic timeout is a
  failure, and no handler or setup completion is invented.
- Missing IN-bank arm: the actual descriptor sender remains in its real transfer
  wait, with no host IN completion.
- Misaligned descriptor pointer: a specific model admission refusal occurs before
  any setup DMA succeeds; unrelated panic/setup errors do not qualify it.
- Hidden WebUSB interface: actual core descriptor discovery stalls at BOS while
  the ordinary USB configuration still works.

The external host supplies bounded typed tokens and input payloads. Reset arrives
first; enumeration tokens are released only after the actual reset handler has
returned with initialized control buffers. Planned and actually queued tokens
are recorded separately. This arrival schedule makes no physical timing claim. Every reply
comes from actual guest descriptor-selected SRAM; no descriptor, class response
or completion is supplied as an expected value to the model. Positive cases
require completed calls and clean exception/mask/IRQ state. Negative cases require
their specific observed state; a generic timeout is failure. Independent readers
must bind official source/job/artifact, source/header/library/image and complete
host-event joins before merge. Preliminary mock/layout controls alone do not
qualify this leaf or its engine model.

This is an authored diagnostic interface, not HF2 or native PXT program startup.
It does not close the full64-file PXT header/source graph, generated native
program, actual CF2 runtime use, analog/audio/display/peripheral or board migration tasks.
The existing clock model supplies nominal register behavior; generator setup,
48MHz tolerance, electrical signaling, timing, retry/PID/isochronous modes,
physical accuracy and complete USB/peripheral reset are outside this boundary.
The no-ASF native consumer remains scoped in
[the board contract](../../docs/PXT-NO-ASF-BOARD-CONSUMER.md).

## Qualified source-built checkpoint — 2026-10-24

Paired Core PR256 and Codal PR82 qualify engine
`8631d4cbb31d06d9e73306d58b66bc998817ecc5` and reviewed consumer
`844fd7e439e6dd96a3d4a65c86c2952c6eba3483`. The actual tested consumer tree in
[run40093459743](https://github.com/CrispStrobe/codal-samd/actions/runs/40093459743)
includes the independently qualified CF2/reopen parent additions without selecting
them into this USB compiler closure. Both enabled consumer checks pass, and
frozen standalone readers pass all12 case/profile original audits, including
source/dependency/library members, six distinct ELF images, five actual official
checkouts and complete host input/response bytes.

| Actual source case | Terminal engine step, both profiles | Replies | Specific result |
| --- | ---: | ---: | --- |
| Bulk echo | 29424 | 27 | Complete descriptors and64-byte echo, real manual ZLP,66 caller checks |
| Interrupt echo | 29355 | 26 | Complete descriptors and64-byte echo, manual ZLP suppressed,66 checks |
| Hidden WebUSB | 17330 | 11 | Actual BOS discovery stall after the ordinary configuration succeeds |
| Missing NVIC80 enable | 11575 | 1 | Reset stays pending, no handler/endpoint constructors,128 actual WFI entries |
| Missing IN-bank arm | 12133 | 2 | Actual descriptor-send wait, no invented IN completion |
| Misaligned descriptor table | 11277 | 1 | Exact RAM/alignment admission refusal before setup DMA |

Positive cases observe constructors `[1, 2, 2]`, natural IRQ80/82 entries
`[9, 1]`, and clean terminal exception/mask/IRQ/fault state. Earlier source,
compiler and observer failures retain their original identities. The original
`writeEP` substring observer was corrected to exact ELF function ABI names;
poisoned BSS is observed only after real startup clears and verifies it; the
caller checks the actual `DEVICE_OK` API; and missing-IRQ waiting is measured
through actual WFI loop entries with every intervening invariant checked.
Timeout, wrong bytes and unrelated panic remain failures.

These engine-step counts are finite correctness observations, not USB timing or
representative native performance. General IRQ80 and OUT IRQ82 are exercised;
SOF generation and natural IN IRQ83 delivery remain unqualified. Authored CF2
source/data admission is separately complete in PR83; this leaf does not read it.
Full source/PXT/HF2 and endpoint/deallocation lifetime work have explicit
[remaining implementation lanes](../../docs/PXT-NO-ASF-NEXT-LANES.md). Preserve the
successful originals and read them without guest replay.

## Endpoint lifetime and deallocation — lane G

Two source findings from the qualified leaf are now reproduced through actual
driver callers, fixed, and guarded by omission mutants that restore the exact
original bytes.

**`UsbEndpointIn::wLength`.** The constructor assigned `ep` and `flags` only;
`write()` reads `wLength` before clearing it. Endpoints are re-created with
`new` on every bus reset, and startup clears only data/BSS, so the field takes
whatever the heap held. The `stale-heap` cases give the heap a deterministic
authored residue (every halfword 32) before the first instruction. This models
retained SRAM after a warm reset or a previous program; it is not a physical
capture. With the original driver (`stale-heap-omitted-init`, byte-identical
to the previously qualified adapted driver) the bulk echo reaches the host as
32 bytes with no terminating ZLP while the guest reports success: a silent
truncation. `src/SAMDUSB.cpp` now initializes `wLength = 0` in the constructor,
and the same residue passes the complete bulk campaign.

**`CodalUSB::sendConfig()` deallocation.** The pinned core allocates the
configuration descriptor buffer with `new[]` and frees it with scalar `delete`;
the selected compiler reports `-Wmismatched-new-delete`. The build applies one
exact owned adaptation to `delete[]`, bound to the pinned source hash; the pinned
checkout, notice and the reviewed service patches are otherwise unchanged. Both operators reach the
same `free()`, so protocol bytes cannot tell them apart. The guest observes the
actual operator entered with a return address inside `sendConfig()`: positive
cases require one `operator delete[]` per configuration request and no scalar
delete. `config-scalar-delete` restores the upstream source byte-for-byte; its
bytes still agree and it is detected by the deallocator observation alone. The
original compiler warning is retained in that case's compile log, not
suppressed.

**Repeated reset.** `re-enumerate` releases a second host bus reset only after
the first echo returned, then repeats the whole enumeration and echo. Actual
endpoint destruction and re-creation are observed (`constructors [1, 4, 4]`,
IRQ80/82 `[18, 2]`, 132 caller checks, two `delete[]` calls). The pinned core
keeps `usb_initialised` across a bus reset, so `isInitialised()` stays true
until the next SET_CONFIGURATION; the case requires the second echo itself.

| Case | Source | Specific result |
| --- | --- | --- |
| `stale-heap` | fixed driver | Complete bulk campaign over the heap residue |
| `stale-heap-omitted-init` | original driver bytes | Host receives 32 of 64 bytes, no ZLP, guest status success |
| `re-enumerate` | fixed driver | Two full enumerations and echoes across a real second bus reset |
| `config-scalar-delete` | upstream core bytes | Protocol bytes equal; scalar `delete` from `sendConfig()` detected |

These cases qualify the endpoint initialization and descriptor deallocation
paths they exercise. They do not qualify general heap reuse, every endpoint
lifetime, complete peripheral reset, or C++ lifetime correctness beyond these
callers.

## Register header correction — 2026-10-25

The independently authored [samd51_usb_mmio.h](samd51_usb_mmio.h) placed
`CTRLA.RUNSTDBY` at bit 6 and listed reserved bits in `USB_DEVICE_INTFLAG_MASK`.
Microchip's SAMD51 ATDF places `RUNSTDBY` at bit 2 and defines the interrupt
mask as `0x3fd`; the engine's USB model had the same bit-6 assumption and was
corrected with it. Every bitfield and literal of this header is now checked
against the ATDF by `tools/samd51-device-header/test_generate.py`. The ten
leaf cases keep their terminal steps.
