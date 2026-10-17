# Finite unmasked asynchronous allocated DMA completion

The separately selected `allocated-dma-async-unmasked` caller qualifies a finite
unmasked interval after asynchronous completion. The original masked
`allocated-dma-completion` caller and its exact original result remain unchanged.
This is part of the
[allocated-buffer lifetime plan](PXT-GC-FAILURE-LIFETIME-LANES.md#b-allocated-buffers-through-real-dma-completion-and-cancellation).

The caller allocates a real 4096-byte named-native buffer and a separate 32-byte
public-GC payload. With real DMA genuinely partial and its modeled SPI clock
gated, it performs 256 GC allocations and two collections and checks both live
payloads. After restoring the clock, the passive observer requires all 4096
accept/wire pattern bytes, actual CS/TXC/callback ordering, original callback
LR return and transfer IRQ retirement before matching native/raw and public/app
free returns. Only an actually returned replacement allocation is filled.

After release the caller executes 8192 checked iterations with interrupts
admitted. The observer requires at least 2048 actual unmasked thread steps,
disabled DMA, no extra peripheral rows and no stale transfer IRQ. SysTick
handler steps do not count as thread steps. A separate 2048-step masked
terminal interval follows. These are finite observations, not an unlimited
absence-of-reads or ownership-enforcement claim.

## Independently audited paired result

[Run 38971392666](https://github.com/CrispStrobe/codal-samd/actions/runs/38971392666)
qualifies source `6607235c57156f6e77f00eaecb4b32182fa91196`, tested as
`246698d17ba9762a17ef264fc62587e4a05039c0`, tree
`79b0ab7f46b69df1d0f1103018f9a6fc92ca8026`. Both profiles finish in 6749413
steps with 4096 pattern bytes, two collections, two native allocations/frees,
two raw frees and one public/app free. Each observes 606220 unmasked thread
steps during the 8192-iteration post-release phase. The standalone once-original
audit binds source/image/commands, 52 dependency byte identities, six notices,
vectors/constructors, both raw streams and original callback/IRQ/free returns.

All 12 mutation/profile cases expose the required actual wrong observation:

| Copied-source mutation, in both profiles | Required wrong observation |
| --- | --- |
| Change a native byte | Actual live payload byte differs from its pattern. |
| Duplicate the original callback | A second original callback enters after the first returned. |
| Omit matching native free | Zero native frees at public release instead of two. |
| Omit pending collections | Zero actual collections instead of at least two. |
| Skip the unmasked loop | Actual unmasked interval is shorter than 2048 thread steps. |
| Mask the quiet phase | An actual thread step in that phase has interrupts masked. |

Setup, compilation, architecture and budget failures cannot qualify these
detectors. The [earlier run](https://github.com/CrispStrobe/codal-samd/actions/runs/38957411710)
remains a failed aggregate: the skipped-loop control lost its unused counter
symbol and stopped before guest execution. Its two positive streams were
separately audited but did not qualify that control. The correction retains a
real volatile counter read even when the loop is omitted, preserving all
interval assertions and the original budget. No original guest was replayed or
original artifact downloaded again to inspect it.

Driver, allocator, model, startup and dependency pins are unchanged. These
finite modeled results do not establish physical accuracy, unlimited lifetime,
full board execution, native Arcade, performance, hardware or release readiness.
