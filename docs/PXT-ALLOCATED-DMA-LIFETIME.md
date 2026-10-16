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

The hosted workflow now compiles and links, then runs a separately bound owned
observer in both scheduler profiles. Its assertions independently bind startup,
allocator entries and original LR returns,
all live payloads, actual descriptor/source/range/count and transferred bytes,
DMA/TXC/CS state, callback/finish return or completed IRQ return, fiber identity
and both matching frees, plus the actual raw-free entries and returns. The
observer also requires a finite 2048-step quiet interval after the terminal;
this is not an unlimited absence-of-reads claim. The current terminal interval
runs with interrupts masked and does not qualify absence of stale notifications
after interrupt admission. The later cancellation/error cases require a
separate unmasked post-release interval. These are pending qualification
requirements, not reported passing results. Observing callback entry or a zero token is insufficient:
`delivering_` remains true until the original callback returns.

Separate blocking completion, cancellation and admitted-error cases remain
required, followed by post-release churn and absence of later DMA reads or stale
notifications. A premature-release negative must expose a real transfer or
ownership witness, without accidental-reuse assumptions, fabricated headers or
writes through retired pointers. The existing API borrows buffers and does not
prevent a caller from freeing them. Preserve all earlier regressions, freeze
standalone readers before official acquisition and require every enabled check
plus exact tested/predicted/landed tree and ordered-parent proof before merge.

The first hosted source image linked, but its following host memset control used
an incompatible output-directory name. That original job failure is retained;
the workflow now uses the control's existing directory without changing the
control. The completion caller selects a declared 1 MHz modeled SPI rate before
its first actual guest qualification; the unchanged 12000000-step bound is not a
physical throughput or performance claim. The original observer host-control
indexing failure is retained separately from any guest result.

Four explicitly isolated source mutants are now prepared: change one live native
byte, duplicate the original driver callback, omit matching native free and omit
collections while DMA is pending. Their receiver requires a source/image-bound
actual wrong observation and the named test failure in each profile. Compile,
setup, architectural failure and exhausted budget do not qualify. These mutation
runs are pending; they do not close cancellation, error or premature-release
qualification. Mutated sources are separate copies and the positive image and
engine product sources remain unchanged.
