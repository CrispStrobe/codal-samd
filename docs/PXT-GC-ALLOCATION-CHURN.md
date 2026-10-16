# Qualified finite allocation churn and native refusal caller

This is a separate caller after the
[landed allocation-routing checkpoint](PXT-GC-ALLOCATION-ROUTING.md), implementing
only the first finite part of [failure/churn lane A](PXT-GC-FAILURE-LIFETIME-LANES.md).
[PR53](https://github.com/CrispStrobe/codal-samd/pull/53) landed after all four
enabled checks and three independent campaign audits passed. Real fatal OOM,
large public request boundaries and allocated-DMA lifetime remain owed.

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
streams. Independent frozen original-artifact audits are mandatory before merge.
The earlier 60-check/six-mutant and 23-check/four-case campaigns were also freshly
qualified after the shared builder changes; their source and assertions remain
unchanged.

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
image input, step bound or dependency pin changes. The corrected head then passed
fresh complete hosted qualification and independent original-artifact audits as
recorded below; the first campaign remains a failure.

## Landed checkpoint — 2026-10-16

Source `a78145a6f3303e5f69b573aef10cd4a4e104557a` qualified in
[run 38851467091](https://github.com/CrispStrobe/codal-samd/actions/runs/38851467091),
job `116813380185`. Actual tested merge
`a5605dbd243f84ff672053da8bc3f6eea89ad4a5` had ordered parents
`74293723866fe5fe4ba948fad4ab4afb8d481105` then that source. It landed as
`d87bab1ebfc806b107d9b40028e2133d91c3d17f` with the same ordered parents.
Reviewed, predicted, tested and landed trees all equal
`9865c18e47be82cce25aff405480531172af2a03`. All four enabled exact-head checks
succeeded; no dependency pin, physical capture or hardware acknowledgement changed.

Each scheduler profile completed all **662 checks and 308 boundaries**, with one
static constructor and **10,079,873 guest steps**, inside the unchanged 12-million
bound. Per-profile operation counts are:

| Actual outer operation | Count | Observed boundary |
| --- | ---: | --- |
| Normal allocation | 64 | Public GC and named native API entries, ABI sizes, actual returns and live payloads. |
| Grow/shrink reallocation | 128 | Actual returned prefix before refill; all remaining live bytes after each disturbance. |
| Native arithmetic refusal | 34 | Null returns; the 32 live-pointer cases retain the old allocation and all payload bytes before valid recovery. |
| Native zero/null-zero request | 2 | Null returns, explicitly not real OOM. |
| Release | 64 | Correct public/native free entry and real return PC, with other allocations still checked. |
| Actual collection | 16 | Real collector entry/return and preservation of the still-live permanent public/native buffers. |

The API trace is outer-call accounting: nested helpers are not invented as extra
caller operations. Void free/collector results are explicitly labeled zero in the
observation protocol, not claimed as meaningful ABI return values. All live slot
records and their byte patterns are independently checked by the fixture at each
boundary. This does not prove arbitrary allocation pressure, invalid-pointer
safety or concurrent collection behavior.

Both native-source mutants ran under both scheduler profiles. Omitting the native
copy caused the first native grow to return a wrong one-byte prefix at operation
21; omitting overflow refusal caused operation 2 to return a nonnull pointer for
the maximal unsigned request. The actual wrong result was observed before the
exact named assertion. All four cases failed for those intended reasons, not a
timeout, unrelated panic or setup failure. The isolated mutation images do not
overwrite or qualify as the positive firmware.

The independent audit verifies provider/source identity, all 46 dependencies,
selected source reconstruction, compile/link policy and sole public allocator
ownership, ELF vectors/load image, notices, all positive events and the separate
44-file mutation packet. It joins the actual source/command/dependency/image
delta and original stdout/stderr for each mutant. The first receiver stopped on
an invalid module attribute reference for the engine pin. That original error
is retained. A separate corrected receiver reads the same exact pin from the
frozen engine constants, adds positive/wrong fixture, runtime and engine binding
controls, and passes all five artifact-control methods plus the unchanged stream
controls. No expected pin, guest input, assertion, threshold or bound is relaxed.
The same official originals were read; none was re-downloaded or guest-replayed.

The ELF SHA-256 remains exactly that of the first, unchanged firmware:
`16a379594311fc986d4d401f1200ad376c20235f566a4e3dc77eb2c27d03c9ce`;
binary `e0929bed1ac3c493d19de63915c9868ec4fc3a05888669725b2403ed297a8e3c`.
Original official archive SHA-256 values are
`06bffc1fca27b8256cabaaf23b8c459467c04fa29866c7bb94a06c44c7e121ef`
(positive, 268,665 bytes) and
`a62b3f6c4b19572240aa59b53d1d6d92984259cafd3abeae6953db334c94310f`
(mutation, 95,387 bytes). The 807,256-byte original job log has SHA-256
`f07df72d5c8c4040241239752644c0899b04ace63c679894e64059cff884c947`.

The same source also passed the independently audited
[60-check/six-mutant baseline](https://github.com/CrispStrobe/codal-samd/actions/runs/38851467051)
and [23-check/four-case routing campaign](https://github.com/CrispStrobe/codal-samd/actions/runs/38851467041).
Their images, checks, step counts and prior assertions remain unchanged.

Next is the remaining [failure/churn lane A](PXT-GC-FAILURE-LIFETIME-LANES.md):
real selected-policy OOM and public large-request boundaries before allocated
DMA-buffer lifetime. This checkpoint is not full lane A closure, generated native
Arcade, representative RTx, installed adoption or licence clearance.
