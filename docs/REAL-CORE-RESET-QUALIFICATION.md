# Actual CODAL core-reset qualification contract

This is a proposed source-selected caller, not a passing guest result or a
complete SAMD reset implementation. Preserve the [landed SPI lifetime results](REAL-SPI-LIFETIME-RESULTS.md)
and all previous caller pins/originals. The separate mode19 explicitly enables
`BW_SIMULATION_CORE_RESET=1`; modes0–18 retain their existing selected behavior.
The legacy default CMake/ASF selection is not changed or admitted.

## Named boundary

The actual public `target_reset()` declaration comes from CODAL core
`2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`. The selected implementation masks
interrupts, preserves AIRCR priority-group bits, issues the documented VECTKEY/
SYSRESETREQ write with barriers, then cannot return. It uses no vendor header or
CMSIS implementation. The [Arm Cortex-M4 guide](https://documentation-service.arm.com/static/5f2ac4ab60a93e65927bbdbf)
defines the reset-register values in table2-1 and AIRCR in section4.3.5; the
outer-system reset signal remains implementation-defined. This fixture names
the engine's **core-local** modeled reset only, not physical SAMD peripheral
reset, power loss, nonvolatile retention or hardware safety.

New startup captures PRIMASK/BASEPRI/FAULTMASK/CONTROL/IPSR/MSP/xPSR/LR before
CPSID or C initialization. A closed48-byte NOLOAD witness retains two boot epochs.
This is fresh simulated RAM, not a hardware power-on value or storage guarantee.
Both epochs must reach the real reset vector/startup, copy initialized data and
clear BSS. The first caller poisons its data/BSS and performs a real invalid-key
AIRCR write; the call must return without a new boot. It then changes VTOR,
core exception/NVIC/SysTick state and mask registers before real `target_reset`.
The second entry must observe reset mask/control/PSR/LR/MSP values **before**
startup masks interrupts, restored boot VTOR, cleared NVIC/core priority state
and stopped SysTick. No old-context return or extra exception may become success.
Peripheral models, their pending external sources, flash commands, USB/QSPI and
other board APIs are not qualified by these observations.

## Required gates

Build all20 modes with separate clean immutable core trees. Preserve compiler,
dependency, undefined-symbol, resource and notice gates. A separate build audit
must show old0–18 ELF hashes unchanged; changing common source alone is not
permission to borrow their qualification silently.

The engine source review identifies retained masks/IT/PSR/LR in `CortexM::reset`.
A focused architectural correction needs permanent controls plus clean actual
caller qualification. Because loading earlier fixtures calls this reset path,
repeat the affected real service/heap and SPI success/cancel/gated-timeout/error/
idle-dispatch/ownership/lifetime cases against their unchanged immutable caller
images in both scheduler profiles. A unit-only fix cannot close this boundary.
Run the new reset caller with single-step and actual bounded batching, under
both engine scheduler profiles. Report actual instruction/PC/MMIO/witness order,
source/images/configuration, startup/epoch/data/BSS/vector evidence, fault checks
and closed post-terminal bounds; batch width must not be a label without work.

Freeze independent source/image/provider rules and nonvacuous positive/refusal
controls before acquiring each sole official original. Preserve failures and
exact run/job/attempt/step/tested-tree provenance. Every enabled final-head
check must pass before matched-head merges and landed tree/parents/default refs.
Remaining compatible board/NVM/serial consumer compilation, no-ASF manifest,
component/header/inline/toolchain/map/image/CF2 admission, actual frames/input/
debugger, active RTx and installed consumer qualification remain separate.
