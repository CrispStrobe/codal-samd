# Allocated DMA blocking caller: source preparation

This separate `allocated-dma-blocking` case is not yet guest-qualified. It keeps
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
the existing separate 2048-step masked terminal interval. These are finite
requirements, not reported results or an unlimited absence-of-reads claim.

The new hosted workflow uses clean declared source checkouts, the unchanged
12000000-step bound and both scheduler profiles. No local ARM/engine build or
guest execution qualifies this preparation. Actual hosted originals, frozen
independent original-artifact readers and blocking-specific source mutations
remain necessary. Cancellation, an admitted driver error and a real premature-
release wrong-transfer or ownership witness remain separate unfinished work.
