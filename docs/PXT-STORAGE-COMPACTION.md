# Actual RAFFS compaction contract

This separate case follows the qualified object-remount result in
[PXT-RAFFS-SETTINGS-RUNTIME.md](PXT-RAFFS-SETTINGS-RUNTIME.md). Its source envelope
is `pxt-storage-compaction/`, `.github/workflows/pxt-storage-compaction.yml` and
this document. Existing fixtures, producers, bounds, pins and historical census
remain unchanged. The finite case is qualified below.

Use the same real framework, GC/services, settings and ZFlash implementations
and exact engine79071f69. Reuse the existing link-only producer without executing
or relabelling its old guest. Replace only the authored caller and append a
separately selected accessor for the original public spacing policy. No original
filesystem method, header layout, timing policy or return value changes.

Select32KiB at `0x78000..0x80000`. Authored writes seed retained user/internal
values and a deleted key, then overwrite a4096-byte value with distinct patterns
until actual `FS::tryGC` moves banks. Observe real outer returns, header commits
and generations 1→2→3, exact latest/retained/deleted values, listings and a fresh
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

## Qualified finite checkpoint — 2026-10-23

[PR76](https://github.com/CrispStrobe/codal-samd/pull/76) merged as `e700ee43424df5dfe630f3943e23e2707cb6de87` after all three
enabled exact-head checks passed. The actual source/image/stream and two source-mutant
originals passed independent audits in both scheduler profiles; each official
packet was retained once without guest replay. The primary reader was frozen
before execution; additional compiler/dependency/link refusals were frozen
before original intake and strengthen that unchanged reader. The parent was linked
without executing its old guest. Engine `79071f69` and existing pins stayed fixed.

| Profile | Positive steps | Spacing-refusal steps | Actual source mutants |
| --- | ---: | ---: | --- |
| default | 13678558 | 15231453 | 2 specific wrong results |
| event-scheduler | 13678558 | 15231453 | 2 specific wrong results |

Each positive run reaches five phases and 14,710 storage checks, two actual bank
commits/generations 1→2→3 and a fresh wrapper remount with exact retained/latest/
deleted/internal values and canaries. A separate fresh run reaches the original
panic `921` on version `10`, with four real clock observations, three completed
compactions and unchanged banks from the triggering-write entry. This preserves
the original 10,000 ms policy; it is not a substituted successful compaction.
Missing-header and shifted-move images each expose their specific wrong result.

[Remaining scoped work](PXT-STORAGE-NEXT-STEPS.md) separates actual generated
configuration/header replacement, required IRQ/services, backing reopen and
interrupted operations. Earlier failures and this case's explicit limits remain.
