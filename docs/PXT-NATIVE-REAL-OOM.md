# Separate native fatal-exhaustion candidate

This implements the next finite slice of
[allocation failure and buffer-lifetime work](PXT-GC-FAILURE-LIFETIME-LANES.md).
It is a **qualification candidate**, not a successful guest result. Preserve
the existing baseline, routing and churn callers, fixtures, bounds and mutations.
No engine, framework, core or generator pin changes are included.

## Actual observations required

The explicit `native-real-oom` builder case selects
`pxt-gc-runtime/probe-native-oom.cpp` and the unchanged real startup. It links
the same selected PXT/CODAL service profile and named native allocation domain.
It retains a 32-byte allocation and a 131072-byte guard allocation, fills all
their bytes, then requests a 131072-byte replacement for the original allocation.
Both originals remain owned and live. This must exhaust the selected raw heap,
not a public GC allocation cap or an arithmetic refusal.

`pxt-gc-runtime/guest-native-oom.rs` must observe actual named allocator entry
and arguments, both initial returns, three raw requests (47, 131087, 131087),
real `device_malloc_in` returns (two nonnull, then null), and the actual
`target_panic(DEVICE_OOM)` call and fatal status `0xe0000014`. It checks every
byte of both retained payloads at the earlier boundaries and fatal observation,
real startup/data/BSS/constructors, alignment, interrupt context and fault state.
The 12000000-step bound is a refusal boundary, never an expected-OOM success.
No allocator return, panic or live payload is injected by the observer.

`DEVICE_PANIC_HEAP_FULL` remains enabled. This test intentionally does not require
a normal `0x600d` terminal after fatal exhaustion and does not promise recovery
from a fatal policy. It does not change or qualify public `malloc` exhaustion.

## Nonvacuous controls and hosted execution

`.github/workflows/pxt-native-real-oom.yml` executes the authored caller in both
existing scheduler profiles, then separately compiles and links two isolated
source mutants in each profile. One corrupts an old payload byte before the
real exhausted replacement. The other changes the actual raw allocator's fatal
panic reason from 20 to 21. Both must reach a real third raw-allocation null,
then fail their named payload/policy assertion with the actual wrong observation.
Timeout, compiler/setup error, unrelated failure, or missing null witness is not
a detected mutant. Original positive outputs are not overwritten.

Pure Python and manufactured Rust controls qualify only admission logic. Before
any merge, require all enabled exact-head checks, independently audit the original
source/image/dependency/command/raw-stream packet, and verify reviewed, tested,
predicted and landed trees and ordered parents. Retain original failed campaigns.

## Still pending, in order

1. Qualify this native fatal-OOM case; it is not public-GC OOM evidence.
2. Add separate bounded, below-cap public-GC exhaustion, preserving permanent
   live allocations and observing its selected actual fatal policy.
3. Investigate public `size_t` to `int` oversized boundaries before diagnosing
   or fixing them. A request above `GC_MAX_ALLOC_SIZE` is not real exhaustion.
4. Only after allocation-failure work, qualify allocated DMA buffer lifetime,
   cancellation, TXC, CS and callback/fiber ownership.

Full board configuration, generated native Arcade, paired performance/RTx and
installed application adoption remain later roadmap steps. This candidate
provides no hardware, physical-accuracy, performance or licence-clearance claim.
