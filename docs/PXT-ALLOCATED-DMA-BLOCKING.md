# Allocated DMA blocking caller: finite checkpoint and token follow-up

This separate `allocated-dma-blocking` case retains a first finite paired result. It keeps
the asynchronous caller, observer, source mutants, startup and dependency pins
unchanged. It does not close the remaining
[allocated-buffer lifetime plan](PXT-GC-FAILURE-LIFETIME-LANES.md#b-allocated-buffers-through-real-dma-completion-and-cancellation).

The caller allocates its 4096-byte transmit buffer through the actual named
native API and retains a separate 32-byte public-GC payload. An actual worker
fiber publishes readiness through an Event and calls the original blocking
`Sercom4TxSPI::transfer`. The main fiber resumes when that transfer blocks in the
real scheduler, explicitly gates the modeled SPI clock while the writeback
count is genuinely partial, and performs real allocations and two collections.
Both live payloads must survive. The caller restores the clock, waits for the
worker's completed transfer and performs matching frees from the main fiber.
The driver still borrows its buffer; no ownership-enforcing API was added.

The separate observer binds the actual original transfer entry, arguments,
worker identity and LR return, including success result and return to the same
worker in unmasked thread context. It requires actual DMA/TXC/CS observations,
IRQ retirement before the original transfer returns, then matching native/raw
and public/app free returns. A copied `worker_returned` flag is supplementary
and cannot replace those observations. Startup, constructors, source/image,
allocator domains, descriptor counts and all wire bytes retain explicit checks.

After freeing and filling only an actual newly returned allocation, the caller
runs 8192 checked loop iterations with interrupts admitted. The observer requires
at least 2048 actual unmasked thread steps in this post-release phase, with DMA
disabled, no new peripheral trace rows and no stale transfer IRQs. It then checks
the existing separate 2048-step masked terminal interval. The first finite paired result below satisfies these interval requirements;
it is not an unlimited absence-of-reads claim.

The new hosted workflow uses clean declared source checkouts, the unchanged
12000000-step bound and both scheduler profiles. No local ARM/engine build or
guest execution qualifies this preparation. The first hosted originals and frozen independent reader results are retained
below. The new token-observer head and its specific source mutations still need
fresh actual hosted results and independent original-artifact audits. Cancellation, an admitted driver error and a real premature-
release wrong-transfer or ownership witness remain separate unfinished work.

## First independently audited blocking result

[Run 38945854288](https://github.com/CrispStrobe/codal-samd/actions/runs/38945854288)
qualifies reviewed source `26b54917486fa6863b7dc146e3fd37f4afcbe137`, tested as
`0e0c39ea65a1fb6b015825524171b5cd12e1d139`. Both profiles complete in 6772391
steps with 4096 pattern bytes, two collections, two native allocations/frees,
two raw frees and one public/app free. Each original stream observes 606219
unmasked thread steps during the 8192-iteration post-release phase, without
new peripheral trace rows or stale transfer IRQs. The standalone reader binds
both original streams, source/image/commands, 52 dependency byte identities,
six notices, vectors/constructors and original blocking/IRQ/free returns.
No original guest was replayed to read the once-acquired originals.

This finite blocking result does not yet qualify the following token detectors,
cancellation, admitted driver error or an actual premature-release witness.

## Pending actual token observations and source mutations

The follow-up observer resolves the actual state `begin` and `take` symbols from
the new image. It captures original LR returns, actual nonzero 64-bit token,
state address and worker identity. Actual `take` must return true with a success
result before the outer blocking transfer returns. These observations distinguish
terminal-token consumption from notification, delivery and buffer ownership.

Three isolated driver-source mutations omit the original `take`, duplicate it
or change the actual blocking return result. The receiver requires the observed
zero consumptions, second actual `take` returning false, or real incorrect return
value, respectively, in both profiles. Setup/compile/architecture/budget failure
cannot qualify a detector. Actual paired token and six mutation/profile results
remain pending; the positive driver, allocator, original caller and model pins
are unchanged. The source draft remains unmerged.
