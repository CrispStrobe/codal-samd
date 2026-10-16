# Prospective finite allocation churn and native refusal caller

This is a separate candidate after the
[landed allocation-routing checkpoint](PXT-GC-ALLOCATION-ROUTING.md), implementing
only the first finite part of [failure/churn lane A](PXT-GC-FAILURE-LIFETIME-LANES.md).
No execution result is claimed here yet. Real fatal OOM, large public request
boundaries and allocated-DMA lifetime remain owed.

`pxt-gc-runtime/build_gc.py --case allocation-churn` selects `probe-churn.cpp`
with the unchanged baseline startup. Default and routing inputs/report schemas
remain unchanged. No selected library source, native allocator, dependency pin,
hardware acknowledgement, physical capture, existing caller or bound changes.

The new authored caller interleaves four public GC and four explicitly named
native allocations across eight rounds. Odd/aligned sizes vary by round;
growth, shrinkage, actual collection, interleaved frees and subsequent rounds
check every live payload and disjoint live ranges. Native arithmetic refusal
retains the original live allocation and all its bytes, followed by real valid
reallocation and release. Null/zero and arithmetic refusal are explicitly not
treated as real exhaustion or evidence for changing the selected fatal-OOM policy.

The separate Rust fixture must execute the actual image and observe all 308
outer API/collector entries and real return PCs. It binds ABI arguments and
public/native ownership to actual ELF symbols, reads actual realloc prefixes
before the caller fills new bytes, and independently checks all published live
slots and every payload byte at each boundary. Static initialization, vector
bindings, constructor entry, real thread/mask/collector context, 662 final checks
and a 12-million-step bound are mandatory. A generated observation sequence or
synthetic terminal is not actual qualification.

Two isolated native-source mutants remove the copy or overflow refusal. Each
must fail its intended assertion under both scheduler profiles with an actual
wrong returned prefix hash or nonnull overflow return observed first. A generic
panic, missing marker, setup/compiler failure or exhausted bound is not success.
The original source and positive image remain unchanged; the separate mutation
packet retains sole source, command, dependency and image deltas and raw process
streams. An independent frozen original-artifact audit remains mandatory before
merge. The earlier 60-check/six-mutant and 23-check/four-case campaigns also need
fresh qualification after the shared builder changes.

The [dedicated hosted workflow](../.github/workflows/pxt-gc-allocation-churn.yml)
does not admit full failure/churn coverage, real OOM, allocated DMA, generated
native Arcade, representative performance, app adoption or licence clearance.

## Preserved first campaign failure

[Run 38850512171](https://github.com/CrispStrobe/codal-samd/actions/runs/38850512171)
at source `9603d6aff3c99fee884e0563c033ebe530395477` remains a failed
qualification. Both positive profiles reached the 662-check, 308-boundary
terminal in 10,079,873 guest steps. All four mutant processes failed at the
intended observed wrong-result assertions, but the campaign matcher falsely
rejected their earlier valid `PXT_GC_CHURN_BOUNDARY` observations as if they
were `PXT_GC_CHURN_BOUND` timeout diagnostics. The original artifacts, result
flags and failed log are retained; these observations do not relabel that
campaign as qualified.

The correction distinguishes the complete timeout marker from a boundary name.
A new nonvacuous control fails before that correction, then accepts valid
boundary-prefixed negative observations while still rejecting bare and populated
timeout markers. No caller, fixture, allocation algorithm, request, assertion,
image input, step bound or dependency pin changes. Fresh complete hosted
qualification and independent original-artifact audit are still required.
