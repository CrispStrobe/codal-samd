# Actual RAFFS compaction contract

This separate case follows the qualified object-remount result in
[PXT-RAFFS-SETTINGS-RUNTIME.md](PXT-RAFFS-SETTINGS-RUNTIME.md). Its source envelope
is `pxt-storage-compaction/`, `.github/workflows/pxt-storage-compaction.yml` and
this document. Existing fixtures, producers, bounds, pins and historical census
remain unchanged. Qualification is pending.

Use the same real framework, GC/services, settings and ZFlash implementations
and exact engine79071f69. Reuse the existing link-only producer without executing
or relabelling its old guest. Replace only the authored caller and append a
separately selected accessor for the original public spacing policy. No original
filesystem method, header layout, timing policy or return value changes.

Select32KiB at `0x78000..0x80000`. Authored writes seed retained user/internal
values and a deleted key, then overwrite a4096-byte value with distinct patterns
until actual `FS::tryGC` moves banks. Observe real outer returns, header commits
and generations1→2→3, exact latest/retained/deleted values, listings and a fresh
wrapper remount. Keep native and managed payloads live across real memory GC;
check initialized-data, code and neighboring flash canaries.

A separate fresh execution keeps one settings instance and writes through the
fourth compaction attempt. Preserve `minGCSpacing=10000` and the original horizon
recurrence `H=max(H+10000,T-20000)`. Capture actual timer returns from inside
`tryGC`. Admit the exact spacing panic921 only when its observed horizon exceeds
the actual time, before erase/commit; both banks must remain unchanged from the
triggering-write boundary. No invented time, forced successful return, policy
reduction or generic timeout can qualify it.

The two authored execution modes are selected explicitly after startup and
before the fixture body, in separate fresh machines. No storage or output value
is injected for a positive result.

Two separately compiled actual-source mutants must expose specific wrong
results: missing final new-bank header after successful outer write, and shifted
moved payload after successful compaction. Setup failure, unrelated panic or
timeout is not mutation detection. Run positive/refusal/mutations in both
scheduler profiles, retain complete source/toolchain/library/member/notice
bindings, freeze independent readers before hosted execution and acquire each
official original once. New finite bounds belong only to this new case.

This qualifies bank compaction and its bounded refusal policy only. CPU reset,
backing reopen, interrupted writes, power loss, physical endurance, full-board,
Arcade and RTx require separate evidence. Preserve every original failure and
require all enabled exact-head checks and reviewed/tested/predicted/landed tree
and ordered-parent joins before merge.
