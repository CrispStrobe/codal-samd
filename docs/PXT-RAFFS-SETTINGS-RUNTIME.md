# Actual RAFFS/settings — finite composition qualified

DONE within the declared object-remount boundary. [PR74](https://github.com/CrispStrobe/codal-samd/pull/74)
merged as `f14879dae76b31f717292fca46d6bf959fcaefc1`, after the qualified
fixed-engine adoption in [PR72](https://github.com/CrispStrobe/codal-samd/pull/72).
Reviewed source `153aeaaa35d761b485f90ccd5cbfef57086398e5`, actual tested merge
`a31122772e61809afadb55551c3f1a1dd64ebd6d`, predicted merge and landed merge share
tree `c5c966349ea07c61a0ae3495c592f32d1f3f2ccf`. The landed parents are qualified
base `b9187cce83f568f4d648701681cfa8229746c42f` and reviewed source. All three
enabled final-head checks passed; there were no skipped checks.

[Original campaign39270484180](https://github.com/CrispStrobe/codal-samd/actions/runs/39270484180)
passed both default and event-scheduler profiles. Each executed six ordered
phases and 1,871 authored checks, finishing in 31,200,088 machine steps within
the unchanged 80,000,000 bound. Both used ELF SHA256
`1c292c262d9d7b4f10ad7a01ff2f57261adae82981536358aa7b96068fa538e6`;
the flat image is 44,896 bytes. Independent original audits and selected
component/notice admission passed without guest replay or producer imports.
Step counts are finite execution evidence, not a speed or RTx measurement.

## Observed operations and negative controls

The caller retains the earlier real GC/global/current/suspended-root case and
its 60 checks, then exercises actual settings over the real ZFlash driver.
Completed writes, deletion, exact values and listings survive destruction of
the single old wrapper and construction of a fresh filesystem object over the
same flash. Native and rooted saved buffers retain all 257 pattern bytes
through real collections; the remounted user/internal values each retain their
19 bytes. Initialized data canaries and the entire flash range below storage
are checked at phase boundaries and the terminal. The 16 bytes immediately
below storage are explicitly disjoint from all load bytes.

| Actual method | Completed returns per profile |
| --- | ---: |
| Filesystem construction | 2 |
| Format | 1 |
| Try mount | 50 |
| Settings set / get / remove | 3 / 4 / 3 |
| Settings exists / list | 5 / 4 |
| Original FNV hash | 18 |
| Actual memory GC | 63 |

All four isolated source mutations were detected in each profile:

| Mutation | Required observed wrong result |
| --- | --- |
| Omit settings read-copy | Zero returned payload, phase 1, panic `0xb20` |
| Report removal without deleting | Key still exists after successful return, phase 3, panic `0xb30` |
| Omit saved-buffer root | Wrong retained buffer after real GC, phase 2, panic `0xb21` |
| Keep the old cache instead of remounting | Only one filesystem construction after all six phases |

These are eight specific wrong-result observations. Setup errors, compilation
failure, timeout or an unrelated panic cannot qualify a mutation. Memory GC in
this case does not establish filesystem bank compaction.

## Selected source and component admission

The producer uses actual full `base/core.cpp`, RAFFS/settings, string/buffer/array
objects and vtables, FNV, GC, roots, configuration and CODAL services at the
retained framework/CODAL/generator pins. The engine is explicitly
`79071f69cd6035a820cbdad93da7343d7d501e4d`; existing GC/DMA cases and the
historical census are unchanged. Newly selected RAFFS/settings and full-core methods remain intact apart from
declared include adaptations; the singleton lifecycle hook is separate. Existing
qualified GC/services adaptations are retained. The new settings translation unit uses `-O2` concrete ZFlash construction;
there is no invented implementation of the abstract Flash base's undefined
`totalSize`. Other recorded translation-unit flags remain unchanged.

Actual newlib `libc`/`libm` and GCC runtime archives, map members, source packages,
source candidates and notices are retained. Independent source-package/DSC byte
joins and the applicable GCC Runtime Library Exception 3.1/component notices
were reviewed. The actual ARM string include bodies and their notices are in
the original source envelope; generic candidate basenames alone do not identify
a compiled unit. This is selected component/notice admission, not an independent
distro library rebuild or project-wide licence-clearance claim. No operation-
success stubs or fabricated PXT object layouts were supplied.

## Preserved failures

The [first setup39265769562](https://github.com/CrispStrobe/codal-samd/actions/runs/39265769562)
refused a canary symbol discarded by link-time section collection, before any
ARM instruction executed. Explicit section retention corrected the fixture;
the canary assertion was retained.
[Campaign39268338935](https://github.com/CrispStrobe/codal-samd/actions/runs/39268338935)
then reached phase 3 in both profiles but exposed an incorrect authored
repeated-removal expectation. The pinned source returns `0` for this retained
tombstone, rather than `-1`. The caller now checks that exact distinction below;
the library and the 1,871-check total remain unchanged. Original failed source
frames, streams and artifacts remain preserved and are not positive evidence.

## Declared persistence boundary

Select exactly 32 KiB of manual-storage flash at `0x78000..0x80000`, matching the
actual SAMD51 settings default. Persistence means completed filesystem/settings
operations followed by destruction of the settings wrapper and a new object
mount over the same machine and same flash backing. There is no CPU reset,
engine restart, power loss, interrupted-write recovery or physical endurance
claim. Never hold two live filesystem caches on the same banks.

The original settings singleton has no public teardown hook. A separately
selected authored same-translation-unit hook may destroy it and clear its
pointer only after operations finish. Keep the actual settings and RAFFS method
bodies unchanged; the hook must call the original object APIs and cannot invent
filesystem returns or repair storage bytes. Retain the empty original FS
destructor and disclose its finite-object lifetime limit.

## Source and qualification envelope

The selected `pxt-storage-runtime/` producer, authored caller, observer, descriptor,
mutation controller and `.github/workflows/pxt-storage-runtime.yml` implement this
contract. Retain the admitted framework/CODAL source pins
and qualified engine, actual GC, objects, roots, configuration, clock, message bus,
fiber and scheduler services. Preserve the actual FNV implementation from
`base/core.cpp`, actual string/buffer/array definitions and vtables and any real
transitive implementation they retain. Enumerate selected dependencies and
notices; supply no success stubs, fake filesystem or fabricated object layout.
New toolchain library inputs, if needed, require explicit source/license and
archive-member admission before a positive result can qualify.

The authored caller must exercise actual format, mount, write, read, remove,
existence and listing, missing/refused operations, object remount persistence,
and both live native and rooted managed payloads across real GC pressure.
Observe real method returns, finite ordered stages, exact payloads and declared
code/data/neighbor canaries in both scheduler profiles. Scope source mutants to
specific data, removal/remount or retention outcomes; setup, compiler failure or
timeout cannot qualify a mutant. Freeze independent original readers before
hosted execution, acquire each official original once and preserve failures.

No existing GC/DMA case, bound, terminal, assertion, historical census, descriptor
default, capture or ACK changes. This does not qualify full board IRQ/USB,
generated Arcade, browser/installed application adoption or RTx. Every enabled
final-head check, affected clean guest, original audit and exact reviewed,
tested, predicted and landed tree/ordered-parent join remains required.

## Pinned removal result contract

The pinned RAFFS source distinguishes a never-created key from a tombstone.
`findMetaEntry` finds tombstones as well as live entries; `write` refuses deletion
with `-1` only when there is no metadata record. In this available-space sequence, repeating removal before
filesystem compaction writes another tombstone and returns `0`. This does not promise the
same result after compaction has discarded tombstone metadata. The caller must
check these two exact results separately, retain the independent deleted-key
absence and null-read checks, and verify listing/remount persistence. Do not
change the library or erase metadata to manufacture a different return value.
The first candidate's incorrect repeated-removal expectation and failed guest
remain historical evidence, without runtime qualification.

## Pending tasks and acceptance

The three follow-up slices are now complete within their separate boundaries:

1. [Actual compaction and original spacing refusal](PXT-STORAGE-COMPACTION.md):
   both profiles, actual bank/header/data observations and two specific source
   mutants. Memory GC/object remount alone did not qualify this result.
2. [Current full-board source/configuration census](PXT-FULLBOARD-SOURCE-CENSUS.md):
   reproducible source-only report, with conditional USB and bounded QSPI-use
   observations. It does not qualify a complete header closure or full board.
3. [Modeled CPU/core-system reset with retained backing](PXT-STORAGE-RESET.md):
   real guest AIRCR store, two startups/mounts and specific lost-backing refusal.
   Peripheral models remain live; backing reopen/power loss remain separate.

Start new work from [the remaining scoped queue](PXT-STORAGE-NEXT-STEPS.md).
Preserve this original PR74 case, its object-remount-only result, every failure,
bound, pin and historical receipt. Generated native Arcade, representative
performance and installed adoption retain their own qualification gates.
