# Prospective actual SPI DMA-error recovery

Mode 12 adds a separately selected real CODAL caller, not a driver change or
qualification result. Refresh current source and engine heads before integration.
The [landed eight-case checkpoint](REAL-SPI-TIMEOUT-QUALIFICATION.md) remains
the positive/cancellation/timeout boundary; it does not establish DMA-error
handling, stale IRQs or blocking-result ownership.

## Named fault and caller

Start a valid 32-byte stable primary-RAM buffer through the unchanged TX driver.
A distinct real worker masks interrupts, refuses a competing start as busy,
then globally disables the exclusively owned DMAC before snapshotting partial
writeback progress. Masking alone does not freeze DMA ticks. Global disable is
also necessary because WBADDR is enable-protected. The live
channel must not be disabled/re-enabled: that would discard its cached descriptor
and restart from the original table rather than retaining partial progress.

The worker changes the disabled controller's writeback target to aligned unmapped
address `0x60000000`, records the actual readback and resumes global DMA. This is
an explicit controller-corruption/writeback AHB-fault fixture, **not** a valid
buffer request failing spontaneously or physical fault injection. The original
driver buffer and descriptor are not rewritten, and no host IRQ/Event/callback
or fake peripheral completion is injected. DMAC, SPI, GPIO and Timer stay clocked.

The next ready accepted DATA beat can precede its failed writeback. Preserve
every accepted and wire byte, the old RAM writeback snapshot and the actual
corruption window; do not assume cancellation's two-byte/no-wire prefix applies.
The old RAM writeback is no longer the authoritative post-fault remaining count.
Mapped forbidden destinations are profile refusals, not this AHB/TERR fixture.

## Required engine qualification

1. Build mode 12 with its separate clean pinned core tree and selected source,
   compiler, linker, map, notice and resource/provenance gates. Compare unchanged
   modes 0–11 against their retained separate-build originals before claiming
   unchanged callers. Compiler success alone is not guest qualification.
2. Add a bounded actual ELF fixture under both scheduler profiles. Observe actual
   guest DMAC disable/WBADDR corruption/re-enable writes and the unmapped target,
   retained partial descriptor, accepted/wire bytes and real TERR before the
   actual IRQ47 handler clears its flags. Never synthesize flags or IRQs.
3. Require exactly one typed `dma_error` failure in IRQ47/PRIMASK0 with CS released,
   disabled SPI MASTER `0x2000c`, INTEN0 and DMA TX `0x200d00` before recovery.
   Preserve callback entry/return, busy reentry, actual Event/main resume and
   the unchanged 28-row asynchronous/blocking recovery. No failed-transfer
   success callback is allowed; recovery has its own later CS epochs.
4. Freeze a source/image-bound independent reader and nonvacuous controls before
   original acquisition. Missing/premature TERR, corruption while enabled,
   restart of the initial descriptor, hidden DATA/wire progress, wrong callback
   context, synthetic resume and mixed failure/success evidence must refuse.
5. Preserve any actual failed original before repair. Every enabled exact-head
   check and affected actual guest must pass before matched-head source/engine
   merges and exact reviewed/tested/landed tree/parent/ref verification. A shared
   driver repair requires fresh affected success/cancellation/timeout coverage.

Stale pending IRQ, blocking terminal-result lease, destructor lifetime, native
Arcade board/API/CF2/build/notices/image, frames/input/debugger, repeated RTx and
installed-consumer adoption remain separate slices. No consumer pin, hardware
capture, acknowledgement, performance threshold or store/licence claim changes.
