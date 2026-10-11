# Software-pended IRQ0 mask control — qualification pending

This separate authored diagnostic extends the unconnected interrupt-mask
foundation in [PR5](https://github.com/CrispStrobe/codal-samd/pull/5). It starts
at historical simulation source, not current default master, and changes no
CODAL driver, platform header or runtime admission.

The ARMv7-M vector table has1024 bytes: core exception slots with reserved
zeros, IRQ0's owned handler, and unexpected-exception traps for other external
slots. Startup copies data, clears BSS and begins with PRIMASK set. Only IRQ0
is enabled. The probe writes its NVIC software-pending bit under two nested
masks, requires it to remain pending with no handler entry through inner
restoration, then requires exactly one handler entry after outer restoration.
The handler records stage, PRIMASK and IPSR; the probe requires stage2/mask0/
exception16 and pending clearance. Completion disables/clears IRQ0 and masks
interrupts again. Bounded32-iteration windows are diagnostic observation
windows, not physical latency requirements. Compiler barriers in the mask
primitive are not hardware DMA barriers; explicit DSB/ISB here only order this
diagnostic's NVIC operations.

The dedicated hosted workflow compiles/links only, without standard libraries
or unresolved symbols, and logs source/ELF/map hashes, vectors and disassembly.
No build or guest result is claimed until the actual corresponding run passes.

Next qualification must pin the exact source in LabWired, verify the entire
vector extent and IRQ0 handler address against ELF symbols, poison and observe
BSS startup, entry100, positive600d and exact negative assertions in both
scheduler profiles. Restore-unconditionally-enabled must stop at assertion6;
its host control must additionally observe count1/stage1/mask0/exception16 to
prove premature handler entry, not merely a wrong PRIMASK value. The positive
host control must observe count1/stage2/mask0/exception16. Do not accept an
alternate failure code, compiler failure, timeout or unexpected-exception trap
`0xf00d` as success. The initial source is intentionally unexecuted.

This exercises software-pended IRQ0 only: not a SAMD peripheral source, DMA
completion, priority/preemption, level-triggered acknowledgement, fibers,
all-vector execution, hardware timing or a production platform startup. Do not
claim real PyBadge interrupts or a permissive whole runtime from this fixture.

Origin/access: authored after reviewing the existing diagnostic startup,
LabWired's PRIMASK/NVIC model interfaces and Arm's
[NVIC documentation](https://arm-software.github.io/CMSIS_6/latest/Core/group__NVIC__gr.html)
and [Cortex-M4 register definitions](https://github.com/ARM-software/CMSIS_5/blob/develop/CMSIS/Core/Include/core_cm4.h).
No vendor or CMSIS implementation is compiled. This is integration/source
review, not isolated implementation or whole-repository licence clearance.
