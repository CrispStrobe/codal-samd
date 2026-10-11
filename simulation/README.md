# Simulation-only transfer state prototype

Unconnected R1c foundation for the
[SAMD replacement plan](https://github.com/CrispStrobe/labwired-core/blob/main/docs/engineering/samd-runtime-replacement-plan.md).
This branch starts at historical CODAL pin
`5bd6b93c219c7e784e885ba2d6812809fb6289a8`, not current upstream master.
It does not modify or build the original driver or remove its ASF dependencies.
No runtime image, app adoption, admission, guest or performance result exists.

The pure C++11 state machine separates DMA completion from real SPI TXC,
preserves error/cancel/timeout results, rejects stale generation tokens within
one instance lifetime and allows a terminal result to be consumed only once.
RX-only transfers still need real TX clocks from the eventual driver. Zero
length is a caller no-op; mismatched duplex lengths are rejected. Generation
exhaustion fails rather than wrapping. Tokens are not cross-instance authority;
instances must outlive all associated IRQs/callbacks. The type cannot be copied.

The driver must serialize accesses, bind observations to the current transfer,
quiesce/clear old IRQs before rearming, manage buffers and clock/reset/deadline
behavior, and deliver results through a reviewed callback/fiber API. This class
does none of those jobs. In particular, a numeric generation alone cannot
distinguish a late hardware IRQ incorrectly attributed to a newer transfer.
Consume terminal state before invoking callbacks to permit deliberate reentry.
Reset integration must cancel active state and consume/report its outcome;
do not destroy/recreate an instance while stale notifications remain possible.

The original ZSPI completion callback ignores `DmaCode`. This prototype records
the distinction but is not a fix to that callback. Guest MMIO and asynchronous
consumer qualification are mandatory before connecting or merging a runtime
change. Pure hosted controls are not ARM, DMA, fiber or engine qualification.

Origin: authored here after reading pinned CODAL ZSPI/SAMDDMAC interfaces and
implementations and the public replacement plan. Not isolated/clean-room work;
the inherited sources and notices remain intact. New files carry MIT notices.

## Additional controls — qualification pending

The stacked test branch adds a table oracle over all one million length-six
traces of ten abstract actions, including invalid lengths and a stale-event
bundle. It is a bounded state test, not exhaustive integer/buffer/IRQ coverage;
generation exhaustion is not exercised. The table and implementation have the
same author and do not constitute an independent hardware oracle.

Five temporary header mutations must compile and fail via an actual assertion:
premature success, error reported as success, stale-token acceptance, duplicate
result consumption and overwriting pending state. Compiler failure, arbitrary
nonzero exit or sanitizer crash does not count as detecting the mutant.
Both ordinary test programs run under GCC/Clang with ASAN/UBSAN on hosted CI.
No production-header changes, MMIO or runtime connection are made here.

## ARM build control — qualification pending

The next stacked branch adds an owned Cortex-M4 state-control fixture, minimal
startup and diagnostic linker layout. Hosted CI compiles and links without
standard libraries and requires no unresolved symbols. It does not execute the
ELF or upload it. Toolchain versions and source/ELF/map hashes are logged.
This is target-build evidence only, not guest boot, MMIO, DMA or firmware
admission. The state-control calls are authored test events, not observations
of actual DMA; never describe this fixture as a working DMA driver.

For later engine execution, `control_status` is zero before startup, `0x100`
after probe entry, 1–18 on the first failed assertion and `0x600d` on success.
Resolve the symbol from the exact ELF; do not assume an address. Interrupts
remain disabled and the two-entry vector table is sufficient only for this
bounded diagnostic, not a runtime with exceptions/interrupts. Verify loaded
vectors, initialized data/BSS and observed termination separately before a
guest-execution claim. No ASF, CMSIS or CODAL implementation is compiled into
this fixture; that does not clear the inherited repository or a future image.

## Interrupt-mask foundation — qualification pending

`InterruptMask.h` is an unconnected privileged ARMv7E-M Thumb primitive. It
captures PRIMASK before disabling configurable interrupts and restores the
saved mask, preserving nesting and an already-disabled caller. The paired
authored probe tests disabled-entry and nested enabled-entry mask states. Every
exit leaves interrupts disabled. The dedicated hosted workflow only compiles,
links and logs disassembly/hashes; it does not execute the probe.

This is not an ASF atomic replacement in any distributed runtime. It does not
mask NMI/HardFault, provide a hardware memory/DMA barrier or multicore lock, or
test pending/active IRQs, interrupt delivery, fibers or peripherals. The assembly
memory clobber constrains the compiler, not bus ordering. Calls require the
same privileged execution context and reverse-order restoration; arbitrary
mask values are not application authority. No host fallback is supplied.

Origin/access: authored after reviewing the existing diagnostic startup and
Arm's [CMSIS register documentation](https://arm-software.github.io/CMSIS_6/latest/Core/group__Core__Register__gr.html)
and [GCC intrinsics source](https://github.com/ARM-software/CMSIS_5/blob/develop/CMSIS/Core/Include/cmsis_gcc.h).
This is not an isolated implementation or whole-repository licence claim.
Next: pin this source in an owned LabWired guest control, verify BSS/entry/pass
and a restore-to-enabled mutant failing the exact assertion in both scheduler
profiles. Actual interrupt delivery needs a separate complete vector/NVIC
fixture before any driver integration or runtime qualification.
