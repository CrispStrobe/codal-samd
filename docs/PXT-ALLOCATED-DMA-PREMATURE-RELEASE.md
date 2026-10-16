# Expected premature-release ownership witness

The separate `allocated-dma-premature-release` caller tests the existing
borrowing contract. Its expected result is a finite ownership witness after a
caller deliberately frees its still-borrowed buffer. It does not expect the
driver to reject that free. Actual hosted qualification remains pending.
This is one part of the
[allocated-buffer lifetime lane](PXT-GC-FAILURE-LIFETIME-LANES.md#b-allocated-buffers-through-real-dma-completion-and-cancellation).

The caller allocates a real 4096-byte named-native buffer and a separate 32-byte
public-GC payload, starts actual asynchronous DMA, and gates the modeled SPI
clock at a genuinely partial writeback count. It performs 256 real GC
allocations, two collections and live-payload checks. It then calls the actual
matching `bw_native_free` while the clock remains gated. The observer lets
`device_free`, `bw_native_free` and the caller's wrapper actually return and
records each original LR boundary in order.

Only after those real returns does the observer require the still-enabled DMA
descriptor to retain the original allocation's source range, valid incrementing
byte descriptor, 4096-byte descriptor count, partial writeback count, selected
CS, gated clock and unchanged partial accept/wire counts. This is a retired
owner with a pending borrowed transfer. It is not a completed transfer, an
ownership-enforcing API, a detected hardware defect or evidence of heap reuse.
The observer checks only descriptor/register/trace state after release. Neither
the caller nor observer accesses the retired payload; the owned simulation stops
at the finite witness before the clock is restored. The separate public payload
remains live at that boundary. No cleanup/recovery outcome is inferred.

Three copied-source controls omit the caller's native free, omit the actual raw
free within the native API, or disable DMA before the actual free. They must
reach a concrete terminal with actual wrong free-return counts or disabled DMA,
respectively. Setup, compilation, unrelated architecture failure and exhausted
budget cannot qualify a detector. Both scheduler profiles and all six mutation/
profile cases require clean hosted runs and frozen independent original readers
before once-only intake.

Original completion, blocking, cancellation/error and unmasked async fixtures,
actual driver/allocator/model sources, startup and dependency pins are unchanged.
Pure host controls are preparation only. No full lifetime, physical accuracy,
release, performance or ownership-enforcement claim is made.

## Original observer admission failure retained

The first [premature-release run](https://github.com/CrispStrobe/codal-samd/actions/runs/38958242273) failed before guest execution because the observer required unused replacement/public-free symbols omitted by the linker. This caller intentionally performs no replacement allocation or public free. The revised observer binds only its actual data and forbids entry to public free functions if they are present in the image. Actual native/raw/caller free returns and every pending-descriptor ownership check remain mandatory. Fresh hosted qualification is pending.
