# Allocated DMA caller: source and link preparation

This is an unqualified initial implementation of lane B in the
[failure/lifetime plan](PXT-GC-FAILURE-LIFETIME-LANES.md). A successful link or
host control does not qualify buffer lifetime, actual guest behavior, native
Arcade or physical hardware. Do not merge this source preparation until the
required actual guest cases, source mutants and independent original-artifact
readers are implemented and pass in both scheduler profiles.

The new explicitly selected `allocated-dma-completion` builder case links the
real GC-owned public allocator, distinct named native allocator, existing
`Sercom4TxSPI` driver and upstream `SPI.cpp`. Existing cases retain their exact
input selections. The new startup retains the GC reset/data/BSS/constructor body
and binds the existing DMA and SPI handlers at exception slots 47 and 79.
Dependency pins, allocator guards and the driver itself are unchanged.

The initial caller allocates a 4096-byte native transmit buffer and separate
32-byte public GC payload. It starts the real asynchronous transfer, explicitly
gates the modeled SERCOM clock while the transfer is pending, performs actual
GC allocations/collections and checks both live payloads. It then restores the
clock, waits through the actual Event/fiber path and calls matching native free
from thread context. Post-release writes use only an actual new allocation.
The pending count and callback assertions are requirements to test, not reported
passing guest results. The callback-body flag deliberately does not assert that
the callback or driver delivery has returned.

The initial hosted workflow compiles and links only. The next guest observer
must independently bind startup, allocator entries and original LR returns,
all live payloads, actual descriptor/source/range/count and transferred bytes,
DMA/TXC/CS state, callback/finish return or completed IRQ return, fiber identity
and both matching frees. Observing callback entry or a zero token is insufficient:
`delivering_` remains true until the original callback returns.

Separate blocking completion, cancellation and admitted-error cases remain
required, followed by post-release churn and absence of later DMA reads or stale
notifications. A premature-release negative must expose a real transfer or
ownership witness, without accidental-reuse assumptions, fabricated headers or
writes through retired pointers. The existing API borrows buffers and does not
prevent a caller from freeing them. Preserve all earlier regressions, freeze
standalone readers before official acquisition and require every enabled check
plus exact tested/predicted/landed tree and ordered-parent proof before merge.
