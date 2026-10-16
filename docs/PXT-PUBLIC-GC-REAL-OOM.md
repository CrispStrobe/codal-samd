# Prospective public GC fatal-exhaustion caller

This separate candidate has not yet been compiled or guest-qualified. It follows
the landed [native fatal-OOM checkpoint](PXT-NATIVE-REAL-OOM.md) and addresses
only item A4 of the [failure/lifetime lane](PXT-GC-FAILURE-LIFETIME-LANES.md).
Source predictions below are acceptance requirements, not observed results.

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

Before merging, require all enabled exact-head checks, affected earlier guest
campaigns and a frozen independent original-artifact audit of source, commands,
dependencies, images and raw streams. Record actual results separately after
qualification; retain failures without weakening these requirements.

## Remaining scope

Public `size_t` to `int` oversized boundaries remain a separate investigation.
Allocated DMA/TXC/CS/callback lifetime follows that work. Full board services,
generated native Arcade, installed Lite adoption and representative RTx remain
unqualified by this finite caller. No hardware or licence-clearance claim follows.
