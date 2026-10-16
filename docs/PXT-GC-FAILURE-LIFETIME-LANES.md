# Next lane: actual allocation failures, churn and DMA buffers

This is an implementation handoff after the
[landed finite allocation-routing checkpoint](PXT-GC-ALLOCATION-ROUTING.md).
[PR51](https://github.com/CrispStrobe/codal-samd/pull/51) closes the earlier
finite ISR/in-GC routing lane. [PR53](https://github.com/CrispStrobe/codal-samd/pull/53)
adds [finite churn/native refusal qualification](PXT-GC-ALLOCATION-CHURN.md), not
the remaining real-OOM/public-boundary or allocated-buffer tasks below. Start from
landed source `bfa249c0f0d09df1fecd2e3829dbac25778406cf` and refresh current
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
Item 4's separate [public-GC fatal-exhaustion checkpoint](PXT-PUBLIC-GC-REAL-OOM.md)
landed in [PR57](https://github.com/CrispStrobe/codal-samd/pull/57): both profiles
retain 18 live arrays and reach actual panic 21 on attempt 19, with all four
ownership/panic-policy mutation cases detected. Item 5's separate
[public-size diagnostics](PXT-PUBLIC-SIZE-BOUNDARIES.md) landed in
[PR59](https://github.com/CrispStrobe/codal-samd/pull/59): both profiles observe
the selected cap refusal and the nonnull `SIZE_MAX` return backed by only a
four-byte array block; all four new mutation/profile cases were detected.
[PR61](https://github.com/CrispStrobe/codal-samd/pull/61) corrects that behavior
before signed conversion and alignment/header arithmetic. Both profiles qualify
seven oversized refusals, valid maximum/near-cap payloads and fresh ordinary
recovery, with all 16 new mutation/profile cases and five earlier campaigns
independently audited. The finite A5/A6 slice is closed; proceed to B while
preserving these exact regressions and the remaining borrowing limits.
Inspect the actual selected allocation cap and heap sizing first. A request
above the per-allocation cap tests too-big refusal, not genuine exhaustion.
Use bounded admitted requests and keep prior allocations live to reach real OOM.
The separate [native fatal-OOM checkpoint](PXT-NATIVE-REAL-OOM.md) landed in
[PR55](https://github.com/CrispStrobe/codal-samd/pull/55): both profiles observe
the real raw null, panic 20 and every retained payload byte, with four actual
wrong-result mutation cases. It addresses only the named native domain and does
not qualify the separate public-GC domain or oversized-request investigation.

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
   is `size_t`, while `app_alloc`/`gcAllocateArray` accept `int`. The separate
   [qualified diagnostics](PXT-PUBLIC-SIZE-BOUNDARIES.md) now observe the selected
   cap's actual panic 22 and a nonnull `SIZE_MAX` return backed by only a four-byte
   array block. The PR61 correction now guards before signed conversion and
   alignment/header arithmetic. Retain zero behavior, the selected cap policy and IRQ/in-GC raw
   routing; add adjacent cap, signed-limit and `SIZE_MAX` boundary qualification.
   Preserve the original diagnostic; no requested oversized payload range was
   accessed, and no arbitrary invalid-pointer or fabricated-header test is needed.
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

## B checkpoint: one original completion profile, paired qualification pending

The draft [PR63](https://github.com/CrispStrobe/codal-samd/pull/63) adds a
separately selected caller using actual named-native and public-GC allocations,
the existing SPI driver, real clock gating, collection and Event/fiber delivery.
Its source has not merged. The driver retains its buffer-borrowing contract.

At reviewed source `beaec51c48061afae7de315019801556e2e61e0a`, tested as
`db5788aa7a7087487cafbdf8145e19cddefb69b2`, the original default profile
completed in 6140012 steps with 4096 transmitted pattern bytes, two observed
collections, two native allocations/frees, two raw frees and one public/app free.
The independent original-artifact audit bound its actual source, image,
52 dependency bytes, six notices, command records, callback/IRQ returns and
matching frees to the original stream. The image SHA-256 is
`69e5c46744172e5ea1259149e0d7c1615b2c9643b4669e6dc83637986bc1ba66`.

The [original run](https://github.com/CrispStrobe/codal-samd/actions/runs/38929821394)
was cancelled when it exceeded the provider's 45-minute job limit. Its
event-scheduler profile reached stage 3 without a terminal. This is a partial
result, not a paired pass or an admitted guest refusal. The original failure,
log and artifact remain retained; no original guest was replayed to read them.

The follow-up source `ec58f67ddcb84e85a8048f4dde76ff64f4483ce2` indexes immutable
ELF symbols once and resolves instruction-loop entry PCs before execution,
retaining missing/duplicate-symbol checks and the unchanged 12000000-step bound.
It also prepares four isolated actual-source mutants in both profiles: change a
live native byte, duplicate the original callback, omit matching native free,
and omit pending-transfer collections. Its paired guest and mutation results
remain pending. Host controls and source preparation do not qualify them.

Blocking completion, cancellation, an admitted driver error and an actual
premature-release wrong-transfer or ownership witness remain required. The
observed 2048-step post-terminal quiet interval has interrupts masked; it does
not establish absence of stale notifications after interrupt admission. Add a
separate unmasked post-release interval, freeze standalone readers before each
original intake, and preserve all existing assertions and failures. Lane B and
the full runtime remain incomplete.

## Completion and handoff

No host-only merge. Every changed exact head requires actual clean affected guest
qualification and independent original source/image/command/dependency/raw-stream
audit. Preserve old failures and every enabled check; merge only with matched
head and tested/reviewed/predicted/landed parent/tree composition. Public docs
may report only the finite observed boundary. Full board configuration, generated
native Arcade, representative RTx and installed Lite adoption remain later lanes.
