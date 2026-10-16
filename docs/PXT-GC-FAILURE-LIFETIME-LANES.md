# Next lane: actual allocation failures, churn and DMA buffers

This is an implementation handoff after the
[landed finite allocation-routing checkpoint](PXT-GC-ALLOCATION-ROUTING.md).
[PR51](https://github.com/CrispStrobe/codal-samd/pull/51) closes the earlier
finite ISR/in-GC routing lane. [PR53](https://github.com/CrispStrobe/codal-samd/pull/53)
adds [finite churn/native refusal qualification](PXT-GC-ALLOCATION-CHURN.md), not
the remaining real-OOM/public-boundary or allocated-buffer tasks below. Start from
landed source `d87bab1ebfc806b107d9b40028e2133d91c3d17f` and refresh current
branch/check state before editing. The overall order remains
[the GC/runtime roadmap](PXT-GC-RUNTIME.md#next-implementation-lanes-in-order).

## Existing evidence is not interchangeable

- The unchanged finite PXT GC caller already checks named native-domain
  zero/overflow behavior and native grow/copy (`pxt-gc-runtime/probe.cpp`).
- `pxt-native-allocation/native-aligned-heap.cpp` promises eight-byte native
  alignment, checked metadata arithmetic and old-allocation preservation when
  its replacement allocation returns null. It explicitly retains the selected
  fatal raw-heap exhaustion policy; arithmetic refusal is not an OOM test.
- `runtime-services/probe.cpp::public_heap_exhaustion` exercises a selected
  service-only public allocator, not the PXT-owned public GC allocator.
  Its existing real OOM and old-payload observation cannot qualify the new domain.
- `RuntimeSpiTransferState.h` requires DMA completion AND actual SPI TXC before
  success, and token-matched single terminal consumption. This pure state is
  not proof of notification, actual buffer ownership, or a real transfer.
- `Sercom4TxSPI.h` explicitly borrows buffers through delivery and exclusively
  owns its modeled controller/channel/pins. Existing `spi-lifetime-probe.cpp`
  covers object-lifetime cases; its static buffers do not qualify allocated
  PXT/native-buffer lifetime. Keep those exact checks and boundaries.

## A: a separate finite failure/churn caller

The first finite slice is landed: preserve `probe-churn.cpp`, `guest-churn.rs`,
their 662-check/308-boundary terminal, eight rounds and four native mutant cases
as regressions. The remaining acceptance work is not satisfied by those passes.
Next complete item 4's public-GC real-fatal-OOM case, then item 5's public-size
boundary investigation, with their item 6 mutations; only then proceed to B.
Inspect the actual selected allocation cap and heap sizing first. A request
above the per-allocation cap tests too-big refusal, not genuine exhaustion.
Use bounded admitted requests and keep prior allocations live to reach real OOM.
The separate [native fatal-OOM checkpoint](PXT-NATIVE-REAL-OOM.md) landed in
[PR55](https://github.com/CrispStrobe/codal-samd/pull/55): both profiles observe
the real raw null, panic 20 and every retained payload byte, with four actual
wrong-result mutation cases. It addresses only the named native domain and does
not close public-GC exhaustion or oversized-request investigation.

1. Add a new explicitly selected builder case; do not reduce or repurpose either
   baseline or allocation-routing terminal, bound, fixtures or source mutants.
2. Publish exact context, request size, allocator entry, returned pointer,
   alignment and content witnesses for public GC and named native allocations.
   Use varied odd and aligned sizes, repeated grow/shrink/free, mixed live
   allocations and collection pressure. Check all live payloads after each
   disturbance, not only a single final sentinel. Native and GC pointer domains
   must remain distinct, with correct matching free APIs.
3. Separate null/zero/arithmetic-refusal cases from real heap exhaustion.
   For native arithmetic refusal verify old allocation AND every old payload
   byte remain live, then prove recovery by valid realloc/free. Do not fake an
   allocation failure or disable the selected fatal-OOM configuration.
4. Qualify real OOM in a separate fatal case. Require the actual expected panic
   code, request/entry/stage and retained old payload observed from actual guest
   memory before admitting it as expected refusal. A timeout, unrelated panic,
   malformed metadata or generic failure is not success. Do not require a normal
   success terminal after an intentional fatal OOM.
5. Investigate large public request boundaries separately: the public signature
   is `size_t`, while `app_alloc`/`gcAllocateArray` accept `int`. This is a visible
   conversion boundary, NOT an observed corruption diagnosis. Use source-bound
   controlled refusal observations before claiming/fixing a defect; no arbitrary
   invalid-pointer or fabricated private-header operations.
6. Add source mutants specific to each positive/refusal observation: omit actual
   preservation/copy or mutate the selected refusal guard. Require a distinct
   observed wrong result, never setup/compile failure or an exhausted bound.
   Run both scheduler profiles and preserve every original failing candidate.

## B: allocated buffers through real DMA completion and cancellation

Only after A. Keep the current public API's borrowing contract explicit; do not
pretend the driver already prevents a caller from freeing its own buffer.

1. Add a separate selected runtime case that links the real PXT-owned allocator,
   named native allocation, existing SPI driver and actual required clock,
   timer/message-bus/fiber services. Enumerate all newly linked inputs and vector
   bindings. No silent pin or model change, no ASF4/restricted vendor source.
2. Allocate/fill the transmit buffer through `bw_native_alloc`, retain ownership
   until actual terminal callback or blocking return, and observe actual DMAC
   source/range/count plus transferred bytes. Check live payload after churn and
   real collection in unrelated GC allocations while the transfer is pending.
3. Separately qualify successful completion, cancellation and an admitted driver
   error with a still-live caller buffer; preserve exact CS/TXC/DMA state,
   callback/fiber identity, token and one-notification checks. Only then free and
   churn the buffer, proving no later DMA read, stale completion or notification
   can access the retired transfer. Observe actual engine behavior, not a mocked
   callback or synthetic controller success.
4. A premature-release negative must cause an actual observable wrong transfer
   or ownership witness, not rely on accidental heap reuse. Do not claim the
   borrowed API refuses invalid use unless a newly scoped ownership mechanism
   actually implements and qualifies that behavior. Model timing manipulation,
   if needed for a deterministic test, must use real declared MMIO and preserve
   the original positive path and peripheral acknowledgement assertions.
5. Add nonvacuous lifetime/source mutants and both scheduler profiles, then
   frozen independent original-artifact readers before ONCE acquisition.

## Completion and handoff

No host-only merge. Every changed exact head requires actual clean affected guest
qualification and independent original source/image/command/dependency/raw-stream
audit. Preserve old failures and every enabled check; merge only with matched
head and tested/reviewed/predicted/landed parent/tree composition. Public docs
may report only the finite observed boundary. Full board configuration, generated
native Arcade, representative RTx and installed Lite adoption remain later lanes.
