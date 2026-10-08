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
