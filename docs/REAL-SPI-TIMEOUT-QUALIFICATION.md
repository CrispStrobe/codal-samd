# Prospective actual clock-gated SPI timeout

Modes 10 and 11 separately gate SERCOM4 MCLK and GCLK during an active transfer;
a real worker then snapshots partial DMA progress in a short masked transaction.
DWT, SysTick, DMAC and GPIO remain clocked.
The real Timer's one-shot deadline must deliver one typed timeout through its
immediate MessageBus listener in exception 15, followed by an actual Event wake
and main-fiber resume. No host callback, Event or invented time is injected.

The caller preserves writeback count and clock boundaries across the gate.
Its failure handler restores the clock before inspecting SPI ENABLE/INTEN,
DMA enable and CS, before recovery initialization can overwrite that state.
Gated reads returning zero are not cleanup proof. Source inspection indicates
the existing driver's gated SPI cleanup stores can be dropped; retain any
actual failed original before attempting a repair. Do not weaken the model,
cleanup assertion, four-million-step bound or existing positive ordering.

After successful cleanup and actual callback return/main resume, the caller
waits beyond its original one-shot deadline, requires no extra terminal delivery,
preserves actual wait-begin/end DWT and clock-counter witnesses,
then performs the unchanged positive asynchronous and blocking recovery calls.
The already-fired Timer slot is released before its listener; do not infer
failure from cancellation of a slot that has already been consumed.

Compiler checks and source controls are not guest evidence. Both separately
built modes require bounded actual execution in both engine profiles, preserved
raw gate/progress/IRQ/callback/fiber observations, immutable image/source/map
bindings, a frozen independent reader and every enabled exact-head check before
merge. Any shared-driver repair also needs affected positive and cancellation
qualification. This is not full TX service, native Arcade, physical safety,
installed-consumer, performance or distribution/licence admission.

## Preserved first actual failure and candidate repair

The first actual [run38554696298](https://github.com/CrispStrobe/labwired-core/actions/runs/38554696298)
failed all four cases at the strict cleanup assertion. The frozen independent
reader classified each as qualification failed: one typed timeout, two DATA
acceptances, zero modeled slave wire bytes and writeback remainder 30, but SPI
still ENABLED after restoring the clock. Callback return, main resume, wait
and recovery did not complete. The failed original is retained, not relabelled.

Candidate cleanup now briefly masks interrupts, disables DMA and releases GPIO
CS before temporarily restoring both SPI clock bits. It clears SPI interrupts,
disables to the named MASTER configuration, retires TXC/error flags, then
restores the exact previous gate words and entry mask before failure delivery.
This is an explicit exclusive-SERCOM4 clock policy, not shared-clock arbitration
or cleanup with a gated DMA controller. It requires new qualification; source
review does not make this repair a pass. Both success and cancellation callers
also need fresh execution on this same corrected driver source.
