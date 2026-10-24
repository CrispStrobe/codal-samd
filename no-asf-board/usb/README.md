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
program validates64 received bytes and66 checks before its real IN call returns.
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
program, CF2 metadata, analog/audio/display/peripheral or board migration tasks.
The existing clock model supplies nominal register behavior; generator setup,
48MHz tolerance, electrical signaling, timing, retry/PID/isochronous modes,
physical accuracy and complete USB/peripheral reset are outside this boundary.
The no-ASF native consumer remains scoped in
[the board contract](../../docs/PXT-NO-ASF-BOARD-CONSUMER.md).
