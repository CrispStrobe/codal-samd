# Prospective PXT allocation-routing caller

This is the next scoped case after the
[landed finite GC checkpoint](PXT-GC-RUNTIME.md#landed-caller-and-source-mutation-checkpoint--2026-10-15).
It is a candidate, not an executed or qualified result. The original 60-check
caller, its startup and terminal assertions remain unchanged.

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

## Required qualification

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
native/WASM performance remain separate lanes. No result is claimed here yet.
