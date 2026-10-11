# Strict paired IRQ simulation profile — qualification pending

This opt-in prototype exposes C-linkage, no-argument `target_disable_irq` and
`target_enable_irq` definitions, but is **not** a transparent replacement for
the historical board ABI. It starts on the authored PR6 simulation stack; it
does not select the default `master` build or roll back its five legacy changes.
The existing CMake/driver/header inventory and fixture pins remain unchanged.

## Explicit policy

The adapter atomically saves and masks PRIMASK before every operation. The
outer disable remembers the entry mask; inner pairs remain masked; final enable
restores that outer mask. Already-disabled entry therefore remains disabled.
Depth is bounded at 64, with no narrow-counter wrap. Unmatched enable, changed
thread/exception ownership, lost mask or invalid input latches a terminal fault
and leaves interrupts masked. The void ABI never returns invented success after
refusal. The fault is observable as `bw_simulation_irq_fault`; no reset/recovery
API exists. Startup enabling is not provided by unmatched `target_enable_irq`:
the integrating profile must explicitly establish startup mask policy.

Thread mode requires a scheduler-provided stable nonzero identity through
`bw_simulation_fiber_identity`. There is deliberately no weak/default provider.
IRQ/exception mode is identified through actual IPSR; NMI/HardFault are excluded
because PRIMASK cannot serialize them. The state is initialized at compile time;
the authored startup does not silently supply C++ constructor execution.

`bw_simulation_irq_may_switch_context` returns false while any pair is open or
after a fault. This is an advisory seam, not a connected scheduler guard: actual
fiber switching must be integrated and qualified before runtime selection.
The provided fixture changes a diagnostic identity; it does not switch stacks,
prove real fiber ownership or integrate a CODAL caller. No DMA barrier is claimed.

## Tests and acceptance still owed

Pure hosted policy tests cover enabled/disabled entry, nested pairs, IRQ owner,
underflow, depth limit, changed owner on enter/leave, lost mask and invalid owner.
They compile with both host compilers and sanitizers; local execution is not
required. The separate hosted ARM workflow builds a positive software-pended
IRQ0 control and four distinct refusal controls, without executing any guest.
It requires explicit profile selection, no unresolved symbols and no constructor
sections. Historical ARM workflows are not triggered by this separate namespace.

Four hosted pure-policy mutants must compile successfully and stop at their
exact expected control: always-enable, premature inner enable, cross-owner
admission and missing depth limit. Compiler failure or a different exit is not
accepted as mutation detection. These are policy controls, not ARM IRQ execution.

Next pin the exact qualified source in LabWired and run both scheduler profiles:
observe startup/BSS/vector/entry, positive `0x600d` with exactly one stage2/mask0/
exception16 IRQ and fault0, plus exact refusal statuses `0x201`/`0x202`/`0x203`/`0x204`
and fault codes3/5/4/6 respectively. Refusal loops must remain masked and never reach
`0xbad` or `0x600d`; timeout, compile failure and unexpected exception are not
success. Add independent restore-always-enable/early-inner-enable and ownership
mutants. A compiled fixture is not a guest qualification.

After those controls, integrate representative **actual** effective CODAL callers
(nested Timer, Fiber/Heap paths and early returns), startup and terminal panic,
then actual scheduler ownership enforcement in a named no-ASF profile. Do not
claim that authored patterns replace real consumer tests or that this completes
generated/indirect/assembly/component review. USB/WebUSB remains required by the
observed original configuration, not an optional success stub.

Origin/access: authored following review of our mask/IRQ controls and the public
[actual caller/interface review](https://github.com/CrispStrobe/labwired-core/blob/main/docs/engineering/samd-critical-section-interface.md),
including its historical board ABI and separate context-switch source review.
This is integration work with recorded access, not an isolated/clean-room claim.
No vendor implementation is included or compiled by this prototype.
