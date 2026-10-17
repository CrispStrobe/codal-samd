# Finite allocated DMA blocking and token-consumption results

The separately selected `allocated-dma-blocking` caller keeps the original
asynchronous case and all dependency pins unchanged. It qualifies the finite
blocking/token boundary below, within the
[allocated-buffer lifetime plan](PXT-GC-FAILURE-LIFETIME-LANES.md#b-allocated-buffers-through-real-dma-completion-and-cancellation).

A real worker fiber publishes readiness through an Event and calls the original
blocking `Sercom4TxSPI::transfer` on a real 4096-byte named-native allocation.
Main resumes when the original transfer blocks, gates the modeled SPI clock at
a genuinely partial count, performs 256 real GC allocations and two collections,
and checks both the native payload and a separate 32-byte public-GC payload.
Restoring the clock allows actual DMA/TXC delivery, IRQ retirement and the
original blocking return to that worker. Matching native/raw and public/app
free returns follow from main. Only an actually returned replacement allocation
is filled after release; the retired payload is not accessed.

The passive observer captures actual state `begin` and `take` LR returns, the
nonzero 64-bit token, matching state/worker, successful consumption and original
outer blocking return. A copied caller flag cannot replace any of these facts.
During 8192 checked post-release iterations it requires at least 2048 actual
unmasked thread steps, disabled DMA, no new peripheral rows and no stale
transfer IRQ. A separate 2048-step masked terminal interval remains.

## Independently audited token result

[Run 38950295026](https://github.com/CrispStrobe/codal-samd/actions/runs/38950295026)
qualifies source `f9dd6ea8f7791387d9f162c0977f0175457a5d80`, tested as
`0805336d0d899bee86e6e23425ccd33747494b36`, tree
`11c8d58e333b3ea2b511df68843b350c5ce6fdef`. Both profiles finish in 6772391
steps with 4096 pattern bytes, two collections, two native allocations/frees,
two raw frees and one public/app free. Each observes 606219 unmasked thread
steps after release. The standalone once-original audit verifies source/image,
commands, 52 dependency byte identities, six notices, vectors/constructors,
both raw streams and actual token/worker/IRQ/API/free-return boundaries.

All six copied-source mutation/profile cases produce the required actual wrong
result: omitted `take` gives zero consumptions at the outer return, duplicate
`take` gives a second original take returning false, and a changed blocking
result gives actual r0=1 instead of success zero. Setup, compile, architecture
and budget failures cannot qualify a detector.

The [first blocking run](https://github.com/CrispStrobe/codal-samd/actions/runs/38945854288)
remains a separate finite paired result for source `26b54917486fa6863b7dc146e3fd37f4afcbe137`,
tested as `0e0c39ea65a1fb6b015825524171b5cd12e1d139`. It observed the same
6772391 steps, 4096 bytes and 606219 unmasked steps, before the later token
and six mutation observations. Original failures and reader corrections remain
retained; originals were not replayed or downloaded again for inspection.

These are finite modeled blocking observations, not unlimited absence of later
reads, an ownership-enforcing API, physical accuracy, hardware, native Arcade,
performance or release qualification. Cancellation, admitted error, unmasked
async completion and premature release each retain their own separate evidence.
