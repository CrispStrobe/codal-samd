# Finite expected premature-release ownership witness

The separately selected `allocated-dma-premature-release` caller qualifies a
finite ownership witness for the existing borrowing API. It deliberately frees
its still-borrowed allocation; the expected result is the pending retired-owner
state below, not a driver refusal. This is part of the
[allocated-buffer lifetime plan](PXT-GC-FAILURE-LIFETIME-LANES.md#b-allocated-buffers-through-real-dma-completion-and-cancellation).

The caller allocates a real 4096-byte named-native transmit buffer and separate
32-byte public-GC payload, starts actual asynchronous DMA and gates the modeled
SPI clock while the writeback count is genuinely partial. It performs 256 real
GC allocations, two collections and live-payload checks. It then lets the real
matching `bw_native_free`, underlying `device_free` and caller wrapper return.
The passive observer records those original LR boundaries in order.

Only after those actual returns does the observer require the original
allocation's range in a still-enabled descriptor: valid incrementing byte
control 0x0409, original count 4096, no next descriptor, partial writeback count,
selected CS, gated clock and unchanged accept/wire prefix. It checks only
register/descriptor/trace state after release. Neither caller nor observer
accesses the retired payload. The owned simulation stops at that finite witness
without restoring the clock, reusing the allocation or inferring cleanup or
recovery. The public payload remains live.

## Independently audited paired witness

[Run 38971392746](https://github.com/CrispStrobe/codal-samd/actions/runs/38971392746)
qualifies source `6607235c57156f6e77f00eaecb4b32182fa91196`, tested as
`246698d17ba9762a17ef264fc62587e4a05039c0`, tree
`79b0ab7f46b69df1d0f1103018f9a6fc92ca8026`. Both profiles reach the witness
in 1747781 steps after two collections and one actual native/raw free pair.
There are two DMA acceptances, zero wire bytes and 4094 bytes remaining. The
standalone once-original reader binds source/image/commands, 52 dependency byte
identities, six notices, vectors/constructors, both raw streams and actual
native/raw/caller returns with the pending borrowed descriptor.

All six copied-source mutation/profile cases expose the required wrong result:

| Mutation, in both profiles | Actual wrong observation |
| --- | --- |
| Omit the native free | Zero native and raw free returns at the caller terminal. |
| Omit the underlying raw free | One native and zero raw free returns. |
| Disable DMA before freeing | The channel is disabled at the required pending-owner witness. |

Setup, compile, unrelated architecture and exhausted-budget failures cannot
qualify these detectors. The [first run](https://github.com/CrispStrobe/codal-samd/actions/runs/38958242273)
remains a preserved pre-guest failure: the observer required unused replacement
and public-free symbols omitted by the linker. The corrected observer requires
its actual data and forbids entry to public free functions if they are present.
The native/raw/caller return and pending-owner checks remain mandatory. No
original guest was replayed or original artifact downloaded again for audit.

Driver, allocator, model, startup and dependency pins are unchanged. This is not
a completed wire transfer, ownership-enforcing API, physical-accuracy,
unlimited-lifetime, hardware, performance or release result.
