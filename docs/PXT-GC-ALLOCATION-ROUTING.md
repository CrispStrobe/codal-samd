# Qualified finite PXT allocation-routing caller

This is a separate scoped case after the
[landed finite GC checkpoint](PXT-GC-RUNTIME.md#landed-caller-and-source-mutation-checkpoint--2026-10-15).
[PR51](https://github.com/CrispStrobe/codal-samd/pull/51) landed after both actual
scheduler profiles, four source mutants and all three enabled checks passed.
The original 60-check caller, its startup and terminal assertions remain unchanged.

## Explicit selection and real execution boundaries

`pxt-gc-runtime/build_gc.py --case allocation-routing` selects only the new
authored `probe-routing.cpp` and `routing-startup.S`. The default case still
selects the previous probe/startup and emits the same baseline report schema.
The new report explicitly identifies the selected case and inputs, with all
guest/board/native-Arcade/performance/licence-admission flags false.
Core, framework, generator and owned engine revisions remain unchanged.

The new caller exercises three public `malloc` contexts:

| Context | Real trigger and intended route |
| --- | --- |
| Thread mode | A 17-byte request with actual IPSR and collector flag zero selects `app_alloc`. |
| IRQ0 | A real NVIC software-pended IRQ, exception 16, makes a 29-byte request selecting `device_malloc`. |
| Collector scan | The actual GC marks a global real `RefRefLocal` and invokes its authored allocating scan callback; a 37-byte request with the real collector flag set selects `device_malloc`. |

The callback descriptor uses PXT's actual `DEF_VTABLE` macro, real class/type
layout and original destroy/print/size methods. Its callback delegates to the
actual `RefRefLocal::scan` after allocating/checking/freeing the payload. The
descriptor is const in flash, respecting PXT's GC tag layout. This is explicitly
authored probe metadata, not generated Arcade bytecode or a replacement PXT type.
The caller never writes the collector's private flag or substitutes ISR state.

The separate startup changes only IRQ0's vector; data/BSS initialization and
actual static-constructor invocation are retained. IRQ entry is a real engine
exception, not a direct function call. Software pending clears through NVIC
entry; no peripheral acknowledgement or physical IRQ timing is invented.

## Qualification contract

The owned Rust fixture observes actual `malloc`, `app_alloc` and `device_malloc`
entry PCs, ABI request sizes, exception state, collector state and real return
pointers. It correlates these to the caller's published pointers and checks
startup, constructors, four ordered stages, exactly one IRQ and scan entry,
and all 23 new caller checks within 12 million steps. That is a separate case,
not a reduced witness or longer bound for the existing 60-check caller.

Both scheduler profiles must pass. Two isolated source mutants respectively
remove the ISR or in-GC term from the actual public allocator condition. Each
runs under both profiles and must fail the exact route assertion with the
actual wrong callee observed, not merely fail setup or time out. Positive and
mutation images, source/command/dependency deltas and original process streams
are retained separately. An independent reader must verify the original official
artifacts after exact provider/source admission; no host-only merge is allowed.

Hosted qualification is in
[the dedicated workflow](../.github/workflows/pxt-gc-allocation-routing.yml).
The existing baseline workflow also qualifies its unchanged caller and six
source mutants after the shared builder changes. No dependency pin, physical
capture or hardware acknowledgement changes. Wider failure/churn/DMA lifetime,
board services, generated native Arcade, installed adoption and representative
native/WASM performance remain separate lanes.

## Landed checkpoint — 2026-10-15

Source `26a8be45bcb0c63a5126120687b4713a596d3c6e` qualified in
[routing run 38817939403](https://github.com/CrispStrobe/codal-samd/actions/runs/38817939403),
job `116707184311`. The actual tested merge was
`3a56995c968e9e1c972850a39e13bcaa4cf87963`, with ordered parents
`f4348c58c4c0e03f7bb4ee7bcaefdca679f23610` then the source above.
It landed as `0e414ad6d581031668f7c103813779e0ce41e177`; reviewed, predicted,
tested and landed trees all equal `a42ce42dc1d79fe865fff8aa30e908991bf4594f`,
and the actual landed ordered parents match. No dependency pin changed.

Each scheduler profile completed all 23 checks in **946,262 guest steps**, with
one actual IRQ entry, one actual scan callback and one static constructor.
The public 17-byte thread request selected actual `app_alloc`; the 29-byte
exception-16 request and 37-byte real in-GC callback request selected actual
`device_malloc`. The callback observed the actual collector flag `2`, while the
thread and IRQ requests observed `0`. All three actual return pointers matched
their caller-published witnesses. Reuse of the freed raw allocation's address
between IRQ and scan is legitimate, not a requirement for pointer uniqueness.

Each of the two actual allocator-condition mutants ran in both profiles. The
ISR-removal and in-GC-removal variants entered actual `app_alloc` instead of
`device_malloc`, then failed the corresponding exact fixture assertion before
any success terminal. These are four observed wrong-route cases, not unrelated
panics, setup failures or timeouts.

An independent standalone reader verifies official provider/source admission,
all 46 dependency identities, selected source reconstruction, actual compile/link
commands, sole PXT allocator ownership, ELF vectors and flat load image, notices,
both positive streams and the separate 44-file mutation artifact. The mutation
audit binds the sole source delta, compile/link/dependency delta, isolated image,
result fields and original stdout/stderr joins for every case. No guest replay
or successful-original re-download was used for reading the evidence.

The first frozen reader refused because it required unused public `realloc` in
this final ELF. That refusal is retained: the declared `--gc-sections` link
legitimately drops it in this caller. A separate corrected reader requires its
absence from this image while still requiring all three public allocator
definitions in exactly the PXT GC object. Nonvacuous controls reject both a
missing object-level definition and an extra allocator owner. No guest, source,
guest assertion, step bound or image was changed to resolve this reader assumption.
The corrected reader and its seven control methods pass; the original remains
unchanged. It does not claim execution of public `realloc` in this routing case.

The unchanged 60-check caller and six original mutants were independently
requalified on the same source in
[baseline run 38817939302](https://github.com/CrispStrobe/codal-samd/actions/runs/38817939302),
job `116707184602`: both profiles still complete 5,853,557 steps, four context
swaps, one constructor and the observed 24-byte copy. The baseline image hashes
remain those of the earlier landed GC checkpoint. Its original failures remain
failures; the separate routing case does not replace or reduce those assertions.

The routing ELF SHA-256 is
`e6dabcf91906e71e4939ad6e78903e677b513029569b8e3a12ea9882dbb4a88a`.
Original official routing ZIP SHA-256 values are
`b3264f7165ae3eb8ed880c8f16365b735d0f03e632c1417f649555d9be2474af`
(positive, 225,285 bytes) and
`bfa5d9f5466cd6aa12f807443a5c2977dda328e037068d32e5389bf5f12b76c0`
(mutation, 98,800 bytes). The original job log is 176,521 bytes, SHA-256
`2780d2385f3dd87cd07d1437b11e68b993bd4ee9332120b8f90b69feb28a2df7`.

These finite authored calls do not qualify arbitrary IRQ nesting, NMI/HardFault
allocation, concurrent collection, full board firmware, generated Arcade, physical
timing, representative RTx, installed-app adoption or blanket licence clearance.
Next is the [failure/churn and allocated-DMA lifetime lane](PXT-GC-FAILURE-LIFETIME-LANES.md).
