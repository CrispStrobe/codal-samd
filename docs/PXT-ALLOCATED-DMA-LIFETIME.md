# Finite asynchronous allocated DMA completion

This is the original asynchronous completion case from the
[failure/lifetime plan](PXT-GC-FAILURE-LIFETIME-LANES.md), landed through
[PR69](https://github.com/CrispStrobe/codal-samd/pull/69). It qualifies a finite
masked terminal interval. The separately qualified
[unmasked caller](PXT-ALLOCATED-DMA-ASYNC-UNMASKED.md) preserves this distinction.

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
The independent guest audit checks the actual pending count and callback ordering. The callback-body flag deliberately does not assert that
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
after interrupt admission. The separate cancellation/error cases also qualify an
unmasked post-release interval. The paired audit checks these requirements. Observing callback entry or a zero token is insufficient:
`delivering_` remains true until the original callback returns.

Separate [blocking/token](PXT-ALLOCATED-DMA-BLOCKING.md),
[cancellation/error](PXT-ALLOCATED-DMA-TERMINALS.md), unmasked completion and
[premature-release](PXT-ALLOCATED-DMA-PREMATURE-RELEASE.md) cases are now qualified.
The existing API borrows buffers and does not prevent callers from freeing them.
Preserve all earlier regressions, assertions and finite observation limits.

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
setup, architectural failure and exhausted budget do not qualify. All eight mutation/profile cases pass their independent wrong-observation
audits; the other lifetime cases retain their separate result bindings. Mutated sources are separate copies and the positive image and
engine product sources remain unchanged.

## Retained first completion attempt

At reviewed source `beaec51c48061afae7de315019801556e2e61e0a` and tested merge
`db5788aa7a7087487cafbdf8145e19cddefb69b2`, the default profile completed in
6140012 steps: 4096 transmitted bytes, two observed collections, two native
allocations/frees, two raw frees and one public/app free. Its original stream,
source reconstruction, 52 dependencies, actual image/vector/constructor layout,
command records, fixture binding and six notices passed a standalone audit.
This is one finite profile only. The event-scheduler stream reached stage 3
without a terminal and remains incomplete.

The [original run](https://github.com/CrispStrobe/codal-samd/actions/runs/38929821394)
was cancelled after exceeding the provider's 45-minute job limit, as recorded
in its annotations. That cancellation is not an admitted guest refusal or a
passing paired qualification. The original log and artifact are retained; no
original guest was replayed to inspect them. The follow-up indexes immutable
ELF symbols once instead of scanning the table repeatedly during execution;
missing/duplicate-definition checks and the 12000000-step guest bound remain.
The [paired follow-up](https://github.com/CrispStrobe/codal-samd/actions/runs/38940230840)
qualifies source `ec58f67ddcb84e85a8048f4dde76ff64f4483ce2`, tested as
`b421f785cc5ccb81863c4b00343709e6e379eb4f`, tree
`52f6e52b189c0ccb4e8e7f229c8bbd4db0638fae`. Both profiles complete in 6140012
steps with the same 4096-byte pattern and allocation/collection/free counts.
All eight actual-source mutation/profile cases expose their required wrong
observation. The standalone original audit binds source/image/commands,
52 dependencies, six notices and both original streams. The original masked
quiet interval remains a finite limit, not unmasked or unlimited qualification.
