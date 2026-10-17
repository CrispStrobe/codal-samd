# Allocated DMA cancellation and admitted error: pending qualification

These separate selected callers implement the next two finite cases in the
[allocated-buffer lifetime plan](PXT-GC-FAILURE-LIFETIME-LANES.md#b-allocated-buffers-through-real-dma-completion-and-cancellation).
They have not yet passed actual guest qualification. The asynchronous completion
and blocking callers, their bounds, regressions and mutations remain unchanged.
Driver, allocator, model and dependency pins are unchanged.

Both callers retain a real 4096-byte named-native buffer and separate 32-byte
public-GC payload through a genuinely partial modeled DMA transfer. Explicit
SPI-clock gating preserves that pending transfer during actual allocation churn,
two collections and live-byte checks. Each case observes only partial accepted
and wire-byte prefixes; it does not claim a completed 4096-byte wire transfer.

The cancellation caller invokes the original driver `cancel`, follows actual
failure delivery in masked main-thread context and requires original callback,
API and outer wrapper returns before freeing. The error caller uses the existing
declared writeback-error mechanism: globally disable DMA, capture the valid
partial writeback, set the unmapped `0x60000000` target, reenable DMA and restore
the SPI clock. The observer requires the actual protected-register state change,
real TERR flag and DMA IRQ before failure delivery and IRQ retirement. It never
dereferences the invalid target; later readback uses only the captured valid RAM
writeback. No reset/recovery or subsequent-transfer qualification is claimed.

The observer captures real state `begin`/`take` entries and LR returns, nonzero
64-bit token, actual owner/consumer identities and cancelled or DMA-error result.
Cancellation consumes in the original main context; error consumption occurs in
the actual interrupted fiber's DMA IRQ context. Both callbacks must see CS
released, DMA disabled, SPI cleanup and their correct exception/mask/result.
Real reentry during failure delivery must return `DEVICE_BUSY`. The observer
follows the original callback return rather than trusting its copied body flag.

Matching native/raw and public/app frees follow actual terminal return. Churn
writes only an actual newly returned allocation. A separate 8192-iteration
post-release interval requires at least 2048 actual unmasked thread steps with
no further peripheral rows or stale transfer IRQs, followed by the retained
2048-step masked terminal interval. These finite requirements do not promise
an unlimited absence of reads or enforce ownership in the borrowed-buffer API.

Three isolated source mutations per caller change the actual failure argument,
duplicate the original failure callback or omit matching native free. Both
profiles must expose the named wrong observation; setup, compiler, architectural
and exhausted-budget failures are refused. The twelve mutation/profile cases
and four positive profile streams are still pending. Freeze standalone readers
before acquiring originals once, retain all failures and require all enabled
exact-head checks and full source/image/command/dependency/notice/stream joins.
An actual premature-release ownership or wrong-transfer witness remains separate
unfinished acceptance work. Neither source draft nor full lifetime is closed.

## First hosted run retained

The first [terminal run](https://github.com/CrispStrobe/codal-samd/actions/runs/38954029066) failed before either caller executed a guest step: the linker removed the unused mode data symbol required by the observer. The revised callers perform a real volatile mode read; every terminal/lifetime assertion and dependency pin remains in place. Fresh paired guests and all specific mutants still require qualification.

## Preserved clock-gated cancellation observation failure

The [corrected-symbol run](https://github.com/CrispStrobe/codal-samd/actions/runs/38967557110)
passed its error child but failed cancellation: the observer expected the SPI
control value after the original cleanup had restored the gated clock. The held
model returns zero for an unclocked peripheral read. This failed aggregate and
both original child packets remain retained; no source merge follows from the
successful sibling alone.

The new observer samples actual SPI control, interrupt enables, DMA channel and
CS while the original cancellation cleanup temporarily admits the SPI clock.
It requires control 0x2000c, cleared SPI interrupts, disabled DMA and released CS
before original failure delivery. The gated callback separately requires the
clock off and its real zero MMIO read; the error callback retains its visible
control check. Actual cleanup ordering is now a required trace witness, not a
substitute copied caller flag. A fourth copied-source mutation omits the real
SPI disable write and must expose the actual wrong control value in both modes
and profiles. The unchanged driver, model, pins, lifetime assertions and step
bound remain in force. Both corrected cases and all 16 mutation/profile results
still require new hosted qualification and frozen independent original audits.
