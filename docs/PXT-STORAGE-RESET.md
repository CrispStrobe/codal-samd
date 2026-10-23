# Modeled CPU/core-system reset with retained storage backing

This separate case owns `pxt-storage-reset/`,
`.github/workflows/pxt-storage-reset.yml` and this document. Preserve every
existing fixture, producer, pin, bound and historical receipt. Reuse the
unchanged actual settings/RAFFS/flash/GC/services link-only parent, replacing
only the authored caller.

The guest must execute an actual keyed AIRCR SYSRESETREQ store after completed
settings writes. Observe the real modeled CPU/core-system reset boundary,
original vector/MSP, masks/NVIC/SysTick/SCB, unchanged flash backing, a second
actual data/BSS/static-constructor/runtime startup, and a fresh filesystem
mount with retained values, deletion and listing. No host reset shortcut,
positive flash restore or injected boot marker.

A separate explicit lost-backing fault erases only storage after the actual
reset and must be detected by the real second-boot failed mount before any
format. Unrelated panic or timeout does not qualify the control. Both scheduler
profiles require source-bound hosted originals and frozen independent readers.

Peripheral models are not reset by this engine route. This qualifies only
modeled CPU/core-system SYSRESETREQ over retained in-memory bus backing after
completed writes. No physical full reset, power loss, interrupted-write recovery,
engine restart/backing reopen or hardware claim. The finite modeled reset case
is qualified below.

## Qualified finite checkpoint — 2026-10-23

[PR78](https://github.com/CrispStrobe/codal-samd/pull/78) merged as `ac43753a4ba7ae4d5f5f0af0cbcf154be151cff4` after all three
enabled exact-head checks passed. Both profiles' original source/image/stream and
explicit lost-backing observations passed independent once-retained audits.

| Profile | Retained-backing positive steps | Lost-backing refusal steps |
| --- | ---: | ---: |
| default | 12124791 | 12039093 |
| event-scheduler | 12124791 | 12039093 |

Each positive guest executes the actual keyed AIRCR store, re-enters original
vectors/MSP and runs real data/BSS initialization, static constructors and
runtime setup twice. The two storage callers reach 270/287 checks, two actual
filesystem constructors, one format and exact retained values/deletion/listing;
flash bytes and code/data/neighbor canaries remain unchanged. The separate fault
run erases storage only after the observed actual reset and sees the real
second-boot mount return `0` before another format instruction can execute.

This remains one Bus's modeled CPU/core-system reset after completed writes.
Peripheral models remain live. [Reopen, interrupted operations and full-board
work](PXT-STORAGE-NEXT-STEPS.md) retain separate contracts and qualification gates.
