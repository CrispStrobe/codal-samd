# Separate unmasked asynchronous allocated DMA completion

The selected `allocated-dma-async-unmasked` caller adds a finite unmasked
post-release interval to asynchronous completion. The original
`allocated-dma-completion` caller and its masked terminal observations stay
unchanged. This new source has no actual guest result yet and does not close the
[allocated-buffer lifetime lane](PXT-GC-FAILURE-LIFETIME-LANES.md#b-allocated-buffers-through-real-dma-completion-and-cancellation).

The caller uses the real named-native allocator for a 4096-byte transmit buffer
and retains a separate 32-byte public-GC payload. It gates the modeled SPI clock
while DMA is genuinely partial, performs 256 real GC allocations and two
collections, and checks both live payloads. It restores the clock and waits for
the original asynchronous callback. The passive observer requires the actual
callback LR return and transfer IRQ retirement before matching native/raw and
public/app free returns. After release, only an actually returned new allocation
is filled; the retired original payload is never accessed.

The new caller then executes 8192 checked iterations with interrupts admitted.
The observer requires at least 2048 actual unmasked thread steps in that phase,
disabled DMA, no extra peripheral trace rows and no stale DMA/SPI transfer IRQ.
SysTick handler steps do not count as thread steps. A separate 2048-step masked
terminal interval remains. These are finite observations, not an unlimited
absence-of-reads claim or evidence that the borrowing driver enforces ownership.
The original full 4096-byte accept/wire pattern and completion/CS/callback/IRQ/
free ordering remain required.

Six isolated copied-source mutations target a wrong native byte, a duplicate
original callback, omitted matching native free, omitted pending collections,
omitted unmasked loop and masking the quiet phase. The last two require actual
insufficient unmasked steps or an actual masked thread observation. Compile,
setup, architecture and budget failures cannot qualify these detectors.

Pure source-selection, mutation-receiver and actual Rust source-cut controls are
host checks only. Actual clean hosted runs in both scheduler profiles and all
12 mutation/profile cases are pending, followed by once-only original acquisition
and frozen independent source/image/command/dependency/notice/stream audits.
The engine, driver, allocator, startup, other callers and dependency pins are
unchanged. No local heavy build, hardware, release or performance result is
claimed. The separate cancellation/error cases and actual premature-release
witness retain their own acceptance requirements.

## Original skipped-loop setup failure retained

The first [unmasked async run](https://github.com/CrispStrobe/codal-samd/actions/runs/38957411710) failed aggregate mutation qualification. Its skipped-loop source control lost the unused counter symbol and stopped during ELF admission, which cannot qualify a lifetime detector. The revised caller retains an actual volatile counter read even when the loop is omitted. The receiver still requires an actual insufficient unmasked interval; no setup failure is accepted.
