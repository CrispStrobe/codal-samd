# Real CODAL service integration

Status: initial implementation candidate; no clean build, guest execution, whole
runtime admission, native Arcade or app/performance qualification yet.

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
  and `CodalListener.cpp`;
- `source/drivers/MessageBus.cpp` and `source/types/Event.cpp`.

These seven sources carry Lancaster University's MIT notices, which remain
intact. The existing MIT BBC/Lancaster `asm/CortexContextSwitch.s` has a
separate origin and ABI review. Review every additional required input and
transitive header; source selection alone is not an image licence receipt.

Six units are unchanged. The explicit `runtime-services/fiber-guards.patch`
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
prospective assertions, not observed guest results. Further clock/reset/mask
and error controls remain acceptance work; build success alone is insufficient.

Clock, Timer and MessageBus instances have real CODAL heap lifetime: the
actual fiber implementation pages its stack, so interrupt/global pointers
must not refer to objects on the replaced stack. The initial stack-local
caller was corrected by source review before any execution qualification.

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
