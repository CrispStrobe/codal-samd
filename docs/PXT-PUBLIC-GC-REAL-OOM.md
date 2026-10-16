# Public GC fatal-exhaustion qualification

This separate finite checkpoint follows
the landed [native fatal-OOM checkpoint](PXT-NATIVE-REAL-OOM.md) and addresses
only item A4 of the [failure/lifetime lane](PXT-GC-FAILURE-LIFETIME-LANES.md).
[PR57](https://github.com/CrispStrobe/codal-samd/pull/57) landed after all six
enabled exact-head checks and five independent campaign audits passed. Both
scheduler profiles observe real public-GC exhaustion with retained live payloads.

## Preserved first failures

The first hosted candidate compiled, but its default-profile observer stopped
before allocator entry: the caller published its first attempt before publishing
the request size, violating the per-instruction metadata assertion. The original
failure is retained. The corrected caller publishes request size before attempt;
allocator assertions, panic policy and step bound are unchanged. Both profiles
and actual-source mutation qualification were still pending at that checkpoint.
The original [first run](https://github.com/CrispStrobe/codal-samd/actions/runs/38870105208)
does not qualify allocator entry or exhaustion.

The second candidate reached the actual allocator chain and collection, then
failed its draft raw-allocation census: the retained `gcProcessStacks` source
allocates and frees a one-byte temporary when `currentFiber` is null. The
observer now separately requires that exact real root-scan allocation, nonnull
return, matching free and completed retirement for every collection. The sole
GC backing-block allocation remains a separate counter; arbitrary extra raw
allocations or a raw failure are still refused. The second original failure is
retained in the [second run](https://github.com/CrispStrobe/codal-samd/actions/runs/38870841394).
It does not qualify exhaustion. The corrected census passes the checkpoint below.

## Selected source and expected boundary

The explicit `public-gc-real-oom` builder selection retains the existing startup
and links `probe-public-oom.cpp`. The baseline, routing, churn and native-OOM
callers and their bounds remain separate. Held core, framework, generator and
engine inputs are unchanged; no allocator policy or per-allocation cap changes.

The caller initializes the real raw heap with `device_malloc(4)` and matching
free, then requests up to 32 public `malloc(8192)` arrays. Every returned array
remains live and permanent, filled with its slot-specific byte. Surviving all
32 requests is an explicit failure. The selected `pxtcore.h` cap is 128 KiB;
each aligned array plus its four-byte header requests 8196 bytes, below that cap.
This tests aggregate exhaustion, not oversized-request refusal.

Selected metadata reserves 32768 system-heap bytes from the actual reported raw
heap. The first GC backing block should therefore provide
`heap_bytes - 32768 - 8` usable bytes. The observer derives the completed-array
count from that capacity divided by 8196. The next block attempt cannot extend
beyond the selected RAM end; the actual source should collect with dump flag 2
and reach `target_panic(21)`, then publish status `0xe0000015`.

## Required observations

The hosted workflow must compile and execute the actual source-bound ELF in
both scheduler profiles within the unchanged 12000000-step bound. The observer
checks reset/data/BSS/constructors, thread context, all four allocator entries
and LR returns (`malloc`, `app_alloc`, `gcAllocateArray`, `gcAllocate`), the real
raw backing allocation, both block-helper calls and actual collections.
Every scheduler-absent root scan must allocate and release its real one-byte
temporary, with no live temporary at allocation boundaries or fatal admission.
It checks each published live pointer, permanent array header and all 8192
payload bytes at every completed-allocation boundary and final fatal status.
Expected calls include the fatal attempt; returns include only completed arrays.
A second raw allocation failure, wrong panic, fault, timeout or normal survival
cannot qualify public GC exhaustion.

Two isolated actual-source mutations must fail at named wrong observations in
both profiles: removing permanent ownership must expose the first returned
header `0x40002004` instead of `0x60002004`; changing only the exhausted-block
panic site must expose reason 22 instead of 21. The ownership mutant stops
before exhaustion and must not be reported as an OOM observation. Host matcher
controls are manufactured inputs, not guest evidence.

Qualification requires all enabled exact-head checks, affected earlier guest
campaigns and a frozen independent original-artifact audit of source, commands,
dependencies, images and raw streams. Record actual results separately after
qualification; retain failures without weakening these requirements.

## Landed source and independently audited result — 2026-10-16

Reviewed source `52c608ae65d6b628d458442a8f5a93eb0f873a09` was tested as
`42eb1c11888c48c10f3a907d5d89aaf675361354` by
[run 38873330805](https://github.com/CrispStrobe/codal-samd/actions/runs/38873330805).
The normal merge `c007988ee24a13ae1b03715cf3f8591b6e3d7ba6` has the same tree
`eafccc3bd5cd8f5124f64ec9370ee7ccc8684342`, with ordered parents
`22cc75590375bc359459d563c71556f5da904e4c` then the reviewed source.

Both scheduler profiles produced the same bounded observations:

| Observation | Actual result |
| --- | --- |
| Public allocations | 18 completed live arrays of 8192 bytes; attempt 19 exhausts the GC block. |
| Allocator chain | All four entries observed 19 times; all four returns observed 18 times. |
| Real GC backing | Raw heap 188040 bytes; one successful 155272-byte backing request; usable capacity 155264 bytes. |
| Collection and temporary lifetime | Six real collections; six one-byte root-scan allocations and six matching completed frees. |
| Fatal terminal | Actual panic 21 and status `0xe0000015`; two block-helper entries, including the exhausted extension attempt. |
| Retention | Every live payload byte and permanent array header intact at each completed-allocation boundary; all 147456 payload bytes intact at fatal admission. |
| Finite execution | 21 caller checks, 18 boundaries, 421 raw markers, one constructor, 1099271 guest steps within the unchanged 12000000-step bound. |

Actual ELF SHA-256:
`1c1802b86d061dd24ed92b1c69eb71db926acc0a332f97353a40a4c585b49be5`.
Flat binary SHA-256:
`df5b4c19729995b8ca5430dd144a8e721ed71aea4cdf18f38b919d8af7d8fa90`.

Both ownership-removal cases observed the first returned header `0x40002004`
and failed the permanent-ownership assertion before exhaustion. Both panic-policy
cases reached actual exhaustion, observed reason 22 and failed the expected-code
21 assertion. Compile/setup failures, bounds and unrelated failures were refused.

Frozen independent readers, tested before original acquisition, verified all
46 selected dependency identities, exact source adaptations, component notices,
compiler/link/dependency commands, ELF load-image/binary equality, actual fixture
and checkout bindings, raw stream order and every mutation packet. Official
originals were acquired once and audited without guest replay or producer imports.

The unchanged baseline, routing, churn and native-fatal-OOM campaigns also
requalified on this exact source and passed separate original-artifact audits:
60 checks/5853557 steps/six mutants; 23 checks/946262 steps/four mutants;
662 checks/308 boundaries/10079873 steps/four mutants; and native panic 20 in
396399 steps with four mutants. The six enabled checks, reviewed/predicted/tested
tree and actual landed parents were verified before admitting this finite result.

## Remaining scope

Public `size_t` to `int` oversized boundaries remain a separate investigation.
Allocated DMA/TXC/CS/callback lifetime follows that work. Full board services,
generated native Arcade, installed Lite adoption and representative RTx remain
unqualified by this finite caller. No hardware or licence-clearance claim follows.
