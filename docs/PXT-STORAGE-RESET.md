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
engine restart/backing reopen or hardware claim. Qualification is pending.
