# Actual SPI destruction-policy qualification contract

The bounded caller/build slice now landed and passed ten actual guest cases
under both scheduler profiles, with a frozen independent original audit and
all enabled final-source checks. Read the [exact sources, results, preserved
failure and remaining limits](REAL-SPI-LIFETIME-RESULTS.md). The contract below
describes the required boundary, not native Arcade admission. Preserve the
separately qualified
[blocking terminal-result ownership checkpoint](REAL-SPI-BLOCKING-LEASE-RESULTS.md).
The driver, interrupt model and old callers are unchanged. New callers select
the same immutable CODAL core and explicit no-ASF simulation build.

## Separate source-selected callers

| Mode | Actual API path and required boundary |
| --- | --- |
|15 | Start a real partial RAM asynchronous transfer, observe writeback progress, delete through the virtual `codal::SPI` interface while its token remains owned. |
|16 | Complete actual DMA and split TXC IRQ delivery, then delete inside the real success callback while `delivering_` is owned. |
|17 | Block an original owner, cancel on a distinct worker with interrupts masked, then delete before that owner can consume its terminal result. |
|18 | Complete actual asynchronous and blocking transfers, delete while settled idle, require the actual deleting destructor and free to return with the incoming mask preserved, reconstruct against the same live Timer/MessageBus, then complete fresh asynchronous and blocking work. |

Modes15–17 must reach the selected driver's deliberate panic reason `0xc42`.
They must not return from deletion, resume a blocked owner, report normal
`0x600d` success, or turn a fault/budget expiry into an expected refusal.
Mode18 alone must reach normal `0x600d`. A fresh allocation may reuse an address;
that does not establish simultaneous identity or unrestricted lifetime safety.
Keep dependencies and buffers alive. Active deletion is refused, not supported
through cancellation-and-destruction.

## Required independent actual qualification before landing

Build all nineteen modes with separate clean immutable dependency checkouts;
preserve the existing input/resource/compiler/undefined-symbol/notice gates.
Verify unchanged old caller ELF hashes rather than silently adopting a changed
driver. Build evidence is not guest execution or whole-image licence clearance.

Before acquiring the first actual original, freeze an independent source/image/
provider-bound reader and synthetic positive/refusal controls. The hosted engine
test must run all four new cases and the unchanged positive control in both
default and event-scheduler profiles. Bind actual virtual-delete/derived
destructor/panic/return PCs, panic `r0`, live `currentFiber`, masks/IPSR and
chronological accepted bytes, wire, DMA/TXC, channel/CS/IRQ effects. Preserve
startup/BSS/data/vector checks, exact bounds and closed observation censuses.
Observe the actual listener-ignore return. CODAL marks a listener for deletion
before later freeing it: do not require immediate node disappearance, inspect
private listener memory or fabricate event delivery. Do not host-clear pending
interrupts; a deliberately pending-edge lifetime experiment remains separate
from the settled-idle control.

Preserve failures and official original provenance. All enabled final-source
checks and independent original audits must pass before normal matched-head
merges and reviewed/tested/landed tree verification. A shared driver/model
repair requires all affected earlier clean success/cancel/timeout/error/stale/
ownership cases, not borrowed historical qualification. No consumer pin,
threshold, acknowledgement or physical capture changes. Remaining board/API/
build/CF2/notices/image/frames/input/debugger, active RTx and installed-consumer
qualification remain separate.
