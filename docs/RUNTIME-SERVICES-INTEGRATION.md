# Real CODAL service integration

Status: clean hosted service builds, the actual heap/Event/fiber baseline and
all four pending-clock cases passed in both engine profiles. Two earlier clock
failures remain retained. Whole-runtime admission, native Arcade and
app/performance qualification remain pending; this candidate is not yet landed.

The engine prerequisites through final wire/TXC/CS/authored callback ordering
landed in [LabWired PR220](https://github.com/CrispStrobe/labwired-core/pull/220).
Its 22 feature-off and 44 feature-on cases passed separate original audits;
all 21 enabled checks passed, and reviewed/tested/landed tree equality was
verified. This is not actual CODAL event/fiber delivery or native Arcade.

## First selected build

Introduce a separate `runtime-services` build entry without invoking the legacy
CMake ASF selection. Own-fork base is
`78bbaa807fed1327c0989b53be0bfcf999d18dab`. The actual CODAL core inputs are pinned to
[`2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`](https://github.com/lancaster-university/codal-core/tree/2ed19d64ec3359143409a4b0a9758b4aeef3ee7d):

- `source/driver-models/Timer.cpp`;
- `source/core/CodalComponent.cpp`, `CodalHeapAllocator.cpp`, `CodalFiber.cpp`
  and `CodalListener.cpp`, plus `MemberFunctionCallback.cpp`;
- `source/drivers/MessageBus.cpp` and `source/types/Event.cpp`.

These eight sources carry Lancaster University's MIT notices, which remain
intact. The existing MIT BBC/Lancaster `asm/CortexContextSwitch.s` has a
separate origin and ABI review. Review every additional required input and
transitive header; source selection alone is not an image licence receipt.

Seven units are unchanged. The explicit `runtime-services/fiber-guards.patch`
adds checks at actual `schedule`, assembly swap and restore call sites, counts
those real calls, and changes initial fiber stack reservation from four to
eight bytes for aligned entry. The actual original Fiber implementation is
retained; this is a recorded integration patch, not an unchanged-source claim.
The build checks that patch against the exact core pin before applying it to
its owned checkout and retains the actual diff. No source helper replaces
Event, MessageBus, components, heap or the fiber queues.

The initial authored caller builds distinct positive and closed-switch images.
It requests actual heap reuse and two real fiber identities; a 20 ms Timer
event must originate in SysTick, wake the worker, and then wake the waiting
main through the real bus. The closed-switch image attempts `schedule` while
the real pair is closed and must reject exactly before switching. These are
observed baseline outcomes in the qualified run below. Full reset/error and
required-board API controls remain acceptance work; build success alone is
insufficient.

Clock, Timer and MessageBus instances have real CODAL heap lifetime: the
actual fiber implementation pages its stack, so interrupt/global pointers
must not refer to objects on the replaced stack. The initial stack-local
caller was corrected by source review before any execution qualification.
The first link failure subsequently identified the actual method-callback
implementation as required by MessageBus; it is now a selected unchanged
upstream source, with its MIT notice preserved. No callback stub substitutes
for it. Dependency rules require all eleven C++ translation units.

Implement an explicitly selected platform header, real heap/new/delete binding,
startup and linker-owned heap/stack, matching TCB operations, clock and interrupt
adapters, and fail-closed scheduler boundary guards. Preserve the saved-mask
primitive and strict paired-policy contract; use real `currentFiber` identities
with a defined boot context, not the earlier diagnostic identity. Cover actual
switch and restoration paths, not just a switch-permission query.

The clock candidate reads actual modeled DWT CYCCNT and uses actual SysTick
exception delivery for compare service. Bind conversion to the selected
120 MHz engine configuration, review wrap/reprogramming/masking, and require
actual guest observations. This is a named Cortex-M simulation adapter, not
physical SAMD TC/clock-tree qualification. Primary register reference:
[Arm Cortex-M4 guide](https://documentation-service.arm.com/static/5f2ac4ab60a93e65927bbdbf).
No synthetic counter increment, host-forced interrupt or completion is allowed.

## Acceptance of the first service slice

Clean hosted builds must preserve source/toolchain/configuration, header
dependencies, allocated/discarded sections, exact ELF/map hashes and notices.
The prospective artifact preserves the exact selected source/header bytes
with their notices, both repository MIT texts and installed compiler-package/
GPL texts. Missing required notice capture fails. Those texts do not establish
the selected library's exception applicability or close allocated-member,
retained-inline or distribution-obligation review; image admission stays false.

The first provenance failure found exactly one external header in the eleven
original compiler dependency rules: GCC `stdbool.h`. Its reviewed source is
[`gcc/ginclude/stdbool.h` at c891d8dc](https://github.com/gcc-mirror/gcc/blob/c891d8dc23e1a46ad9f3e757d09e57b500d40044/gcc/ginclude/stdbool.h),
carrying its own GPLv3-or-later and GCC Runtime Library Exception3.1 notice.
The selected C++ branch defines a boolean alias and a standard feature macro,
not a device implementation. The installed compiler header must match those
exact reviewed1490 bytes/SHA256
`68b898fc39d5d2a450df9b7263d984e7d2ad4c02675f4043a62e762f169e4b85`;
other external headers still refuse. Preserve the unchanged
[exception text](https://github.com/gcc-mirror/gcc/blob/c891d8dc23e1a46ad9f3e757d09e57b500d40044/COPYING.RUNTIME)
alongside GPL and package notices. This file-specific notice review is not
clearance of every linked `libgcc` member or the whole image.
The original map selected the Cortex-M4 Thumb multilib, while the initial
recorder queried GCC without target flags. The corrected recorder uses the
actual link flags and must match the map-loaded archive; the earlier default
archive fingerprint is not selected-library evidence. Preserve that failure.
Unknown inputs, vendor-restricted contributions or an absent explicit selector
must refuse. Do not fall back to ASF directories or stub required services.

The actual guest must exercise real allocation/free/reuse, Timer events,
MessageBus listeners, two distinct real fibers and real assembly context
switch/return, and a hardware exception through Event to a waiting fiber.
Require meaningful masked-clock/interrupt and critical-section-switch refusal
controls, saved-mask restoration and no stale/error success. Qualify both
engine scheduler configurations against immutable sources and independently
audited originals before integration or landing.

This first slice replaces the earlier diagnostic clock/component/allocation/
Event/fiber adapters. It does not replace the remaining native board APIs or
constitute whole-runtime admission. After it passes, connect actual SERCOM DMA
completion through final TXC to exactly-once CODAL notification/fiber wake;
retain PR220's separate acceptance, writeback, wire, CS and callback boundaries.

## Remaining steps after real services

Finish the selected native Arcade build's GPIO/input, SPI/display, CF2 and any
required storage/audio/USB services. An excluded service needed by the selected
runtime blocks admission. Reconcile every selected source/header/inline,
startup, script, generated input, toolchain contribution and distributed notice
with the allocated binary; keep historical ASF4 images rejected.

Only then qualify original native Arcade frames/input/debugger, repeated
representative native/WASM performance, installed consumer pins/artifacts and
measured-bottleneck optimization with regression evidence. Keep translated/PXT
execution usable and separate; slow ordering fixtures, idle batching and
terminal spins are not native Arcade RTx measurements. Other consumer Arcade
compatibility/frame-scheduler lanes remain independently owned.

Follow the [LabWired replacement plan](https://github.com/CrispStrobe/labwired-core/blob/main/docs/engineering/samd-runtime-replacement-plan.md).
No hardware flashing, whole-firmware clean-room origin, blanket legal/store
clearance, physical timing, app adoption or performance result is claimed.

## Pending-clock adversaries — failures retained, corrected slice passed

The first actual service baseline passed both engine profiles: real heap reuse,
four assembly context switches, six SysTick entry/return pairs and real worker/
main event wake; the closed-critical-section caller refused with no switch.
This does not complete the first service slice's required clock/error controls.

New caller modes2/3 arm a real one-millisecond Timer event, cross its deadline
using actual DWT under PRIMASK, and require SysTick COUNTFLAG plus no handler
or Event while masked. Disable only the IRQ or stop the clock before unmasking
the already-pending exception. That entry may retire but must not fire the Event.
IRQ masking must leave time advancing; clock stop must freeze it. Actual
reenable must deliver once through Timer/MessageBus, without any fiber switch.
The expected failure for the old adapter is exactly the stale-event assertion,
not timeout or arbitrary panic. Source stage witnesses and architectural pend/
entry/return observations must establish this ordering. No host injection.

The clock implementation was deliberately unchanged for the first adversary
run. The [original run38450843463](https://github.com/CrispStrobe/labwired-core/actions/runs/38450843463)
then genuinely reproduced mode2's exact stale-event failure in304911 machine
steps: pending SysTick under PRIMASK, one actual entry/return after IRQ disable,
one event call and guest assertion`0xe0000b20`, while clock time advanced2428
to2469. The independently frozen original reader passed this narrow failure
binding; this is not a positive run or mode3 result. The raw failure is retained.

The proposed fix lets that real stale entry retire without consuming deadlines
or firing events when the clock or its IRQ is disabled. It does not clear the
CPU pending bitmap or invent architectural pending-clear behavior.

The [fixed-candidate run38453968112](https://github.com/CrispStrobe/labwired-core/actions/runs/38453968112)
retired the disabled pending IRQ with zero Event calls, but failed after actual
reenable: one Event, three IRQ entry/return pairs and assertion `0xe0000b29`
at308057 steps. Both baseline cases passed in that default profile; later
cases did not execute. This remains a failed qualification, not a clock pass.
Source review identifies periodic SysTick reload during overdue compare
service as a candidate cause. The next adapter stops SysTick at handler entry
and defers callback compare rearming until callback completion, retaining the
actual deadlines. It neither edits the CPU pending bitmap nor relaxes the
exact two-entry expectation.

The [actual run38458217987](https://github.com/CrispStrobe/labwired-core/actions/runs/38458217987)
passed all eight cases against runtime source
`c153f8dab17742588b1ad1a49ede26f3cf04ae14` and core
`2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`. Both profiles preserve the four real
assembly swaps and six baseline IRQ entry/return pairs; closed-switch refuses
before any switch or Event success. Each mode2/3 case retires the pending
disabled IRQ without Event, then delivers one Event on resume with exactly
two total IRQ entry/return pairs and no fiber switch. IRQ-only disable advances
time2428→2445; full clock stop freezes2429→2429. The frozen original-log
reader passed the source/image/checkout bindings and24 chronological clock
boundaries. See the [engine receipt](https://github.com/CrispStrobe/labwired-core/blob/feat/samd-real-services-20261013/docs/receipts/2026-10-13-samd-tx-completion/README.md).

This documentation update does not change the engine's tested runtime pin.
Full reset/error/required-board APIs, actual SERCOM completion-to-CODAL wake,
whole-image admission, native Arcade, RTx and app adoption remain pending.
No architectural pending-clear repair is claimed.

## Long masked interval — prospective diagnostic caller

The next caller mode4 tests a gap beyond65535 microseconds without polling
`Timer::getTimeUs` during that gap. It first arms a real20 ms Event, masks
interrupts, then waits80 ms using only actual low-level DWT-based capture.
The real Timer is read once before and once after the gap. Require real
SysTick COUNTFLAG/pending delivery, no handler or Event under the mask, and
Timer elapsed time within the source-defined bound of the low-level elapsed
time. On unmask, the overdue Event must deliver once through the actual
Timer/MessageBus with no context switch.

The pinned core's `Timer::sync` currently casts elapsed time to16 bits even
though this adapter reports `BitMode32`. That source observation motivates
the test; it is not an observed guest failure. The Timer and clock adapters
remain unchanged for the first diagnostic. A clean hosted fifth image build
is prospective build evidence only. Actual execution, an appropriately named
new long-case bound, preserved baseline cases and independently audited
originals are required before changing the Timer or claiming this boundary
works. Do not add host clock advances, IRQ injection or a success stub.

The previous eight-case result and runtime pin remain unchanged historical
evidence. This candidate is not complete native Arcade or app/performance
qualification, and does not alter the default physical build.
