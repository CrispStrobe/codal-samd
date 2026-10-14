# Actual CODAL cancellation and recovery contract

Mode 9 is a separately selected caller. It does not replace mode 8 or relax its
ordering gate. Corrected source PR22 is landed; both modes passed actual guest
qualification under both scheduler profiles. See the [exact results and remaining
engine-adoption boundary](REAL-SPI-SERVICES-RESULTS.md#corrected-cancellation-and-recovery-checkpoint).
This does not qualify the complete driver, native Arcade, physical behavior,
installed consumers or performance.

The first actual cancellation caller delivered the typed failure and resumed
main, but recovery failed because the driver requested DMAC reset while the
global controller was still enabled. The original failure is retained. Recovery
initialization now explicitly disables the exclusively owned controller before
reset, rather than relaxing the model's reset guard. Because shared driver code
changed, both the mode-8 positive caller and mode-9 cancellation caller require
fresh affected guest qualification before this source can be adopted.

The main fiber starts a 32-byte RAM transfer and waits for an actual Event.
A distinct real fiber attempts a competing start (which must refuse busy),
then calls the real driver's cancellation API. The failure callback must report
exactly one typed cancellation, with the original argument, released CS and
the cancelling fiber's identity, in thread context with interrupts masked by
the cancellation critical section. Reentry from that callback must refuse busy.
No success callback may be substituted for the failure. Its actual MessageBus
Event wakes the enrolled main fiber after cancellation returns and the worker
releases its context.

The caller checks disabled DMA/SPI and released CS, then attempts the same real
asynchronous and blocking four-byte transfers as the positive caller. It requires
one success callback, no second failure, correct callback/worker/main identities,
and actual Event-based resumes. Every partial byte of the cancelled transfer
must remain visible in qualification; do not discard a prefix to fit successful
recovery ordering. Cancellation is not an assertion that no byte was transmitted.

Hosted source builds select fourteen dependency rules and the same immutable
core, explicit patches and compiler-header provenance gates. Mode 9 uses its
own clean core checkout and caller, with the SPI startup/vector profile. Existing
mode selections remain unchanged. Compiler success alone is not guest evidence.

Required actual qualification binds the immutable caller source/image, complete
startup/vector/data/BSS proof and all cancellation witnesses. Observe failure
callback entry/return, cancellation return, DMA/CS cleanup, actual main resume,
complete partial DATA/wire chronology and both recovery transfers, including
DMA completion distinct from final wire/TXC/CS/callback/fiber return. Both engine
profiles must pass a frozen independent reader and bounded actual guest tests.
Those finite actual gates passed for the corrected source; every enabled
engine PR229 check and its matched merge remain required for engine adoption.
Error, gated timeout, stale IRQ and blocking-result ownership
remain separate required slices, followed by full board/CF2/API/build/notices,
frames/input/debugger, representative RTx and installed consumer admission.
