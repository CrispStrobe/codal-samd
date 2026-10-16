# Separate native fatal-exhaustion checkpoint

This implements the next finite slice of
[allocation failure and buffer-lifetime work](PXT-GC-FAILURE-LIFETIME-LANES.md).
The finite native case landed in
[PR55](https://github.com/CrispStrobe/codal-samd/pull/55). Preserve
the existing baseline, routing and churn callers, fixtures, bounds and mutations.
No engine, framework, core or generator pin changes are included.

## Actual observations

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

1. Preserve this qualified native fatal-OOM case; it is not public-GC OOM evidence.
2. Add separate bounded, below-cap public-GC exhaustion, preserving permanent
   live allocations and observing its selected actual fatal policy.
3. Investigate public `size_t` to `int` oversized boundaries before diagnosing
   or fixing them. A request above `GC_MAX_ALLOC_SIZE` is not real exhaustion.
4. Only after allocation-failure work, qualify allocated DMA buffer lifetime,
   cancellation, TXC, CS and callback/fiber ownership.

Full board configuration, generated native Arcade, paired performance/RTx and
installed application adoption remain later roadmap steps. This checkpoint
provides no hardware, physical-accuracy, performance or licence-clearance claim.

## Landed result — 2026-10-16

Both actual scheduler profiles passed in **396399 guest steps**, observing all
18 required stream markers: two successful initial native allocations, the real
third raw-allocation null, actual panic reason 20 and fatal status `0xe0000014`.
Every old payload byte (32) and guard payload byte (131072) remained intact at
the admitted fatal observation. All four isolated mutation/profile cases reached
the real third raw null and failed their intended actual wrong-result checks:
the corrupted old byte or changed fatal panic reason. None was admitted from a
timeout, compiler/setup error or a producer-only `detected` flag.

The same source requalified the unchanged baseline (60 checks / 5853557 steps /
six negative cases), routing (23 / 946262 / four), and churn (662 checks /
308 boundaries / 10079873 steps / four) campaigns in both profiles. All five
enabled exact-head checks succeeded. Independent readers audited the official
original archives and logs without guest replay or producer imports, including
46 dependency bindings, exact source adaptations, commands, actual ELF/binary,
fixture/checkout bindings, retained notices and the 44-member mutation packet.

- Reviewed source: `16a55b5c716a53b0bdca8829bb892a589e82bad9`.
- Actual tested synthetic merge: `b2fa324cf78b31b47fce8b9e6f897382995d4d09`.
- Landed normal merge: `9d1438f4144a93d2501b302b3194cda1978fdc81`.
- Ordered parents: `1e15c48de9cb8e962ddca469ba7526f99c4fbc5e`, then the reviewed source.
- Reviewed, predicted, tested and landed tree: `94f5d1c1ec3b56dae7c9bdd8fc9b61424d349dc9`.
- Native-OOM ELF SHA-256: `9c5805eb4e2ce21e373fd7ce691077c4cda3d8599deb8e4f8f2cbdaf2aba2137`.
- Native-OOM binary SHA-256: `e6c47c64b162854cf8a6c966199fb10f2699e14c51f898312fd0e907fc2527c2`.
- [Native campaign](https://github.com/CrispStrobe/codal-samd/actions/runs/38860593217),
  [baseline](https://github.com/CrispStrobe/codal-samd/actions/runs/38860593071),
  [routing](https://github.com/CrispStrobe/codal-samd/actions/runs/38860593074),
  [churn](https://github.com/CrispStrobe/codal-samd/actions/runs/38860593136).

These are finite authored-caller correctness results, not elapsed-time or RTx
measurements. No dependency pin, fatal policy, hardware acknowledgement or
physical capture was changed. Public-GC exhaustion and public oversized-size
behavior remain unqualified and precede allocated DMA lifetime work.
