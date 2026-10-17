# Finite allocated DMA cancellation and admitted error

These separate selected callers implement the next two finite cases in the
[allocated-buffer lifetime plan](PXT-GC-FAILURE-LIFETIME-LANES.md#b-allocated-buffers-through-real-dma-completion-and-cancellation).
Both cases have passed paired actual guest qualification and landed through
[PR69](https://github.com/CrispStrobe/codal-samd/pull/69). The asynchronous completion
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

Four isolated source mutations per caller change the actual failure argument,
duplicate the original failure callback, omit matching native free or omit the
actual SPI cleanup disable write. All sixteen mutation/profile cases expose
their named actual wrong observation. Setup, compiler, architectural and
exhausted-budget failures are refused.

## Independently audited paired result

[Run 38971392854](https://github.com/CrispStrobe/codal-samd/actions/runs/38971392854)
qualifies source `6607235c57156f6e77f00eaecb4b32182fa91196`, tested as
`246698d17ba9762a17ef264fc62587e4a05039c0`, tree
`79b0ab7f46b69df1d0f1103018f9a6fc92ca8026`.

| Case, same result in both scheduler profiles | Guest steps | Accepted / wire bytes | Collections | Unmasked post-release thread steps |
| --- | --- | --- | --- | --- |
| Cancellation | 3026258 | 2 / 0 | 2 | 573451 |
| Admitted writeback error | 3038133 | 3 / 1 | 2 | 573450 |

Frozen standalone readers bind both original streams, source/image/commands,
52 dependency byte identities, six notices, vectors/constructors, original token,
callback/API/IRQ returns and matching frees. Both cases retain their 8192 checked
post-release iterations and separate 2048-step masked terminal interval.
The actual cleanup mutant exposes control 131086 instead of required 131084;
cancellation fails before token take, while error retains its actual IRQ take.
Neither reader invents consumption after an earlier cleanup failure.

The separate [premature-release witness](PXT-ALLOCATED-DMA-PREMATURE-RELEASE.md)
qualifies pending retired ownership without dereferencing a retired buffer.
These finite results do not enforce ownership, promise unlimited absence of
stale reads, or qualify recovery, full runtime, hardware or release readiness.

## First hosted run retained

The first [terminal run](https://github.com/CrispStrobe/codal-samd/actions/runs/38954029066) failed before either caller executed a guest step: the linker removed the unused mode data symbol required by the observer. The revised callers perform a real volatile mode read; every terminal/lifetime assertion and dependency pin remains in place. The final run above qualifies the corrected paired guests and all specific mutants.

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
bound remain in force. Both corrected cases and all 16 mutation/profile results pass the final hosted
run and frozen independent original audits reported above.
