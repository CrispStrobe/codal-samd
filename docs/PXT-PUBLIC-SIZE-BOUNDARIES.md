# Public request-size guard and historical diagnostics

[PR61](https://github.com/CrispStrobe/codal-samd/pull/61) qualifies the guard
correction and finite boundaries below. The historical separate investigation
follows the qualified
[public fatal-exhaustion checkpoint](PXT-PUBLIC-GC-REAL-OOM.md) and addresses
item A5 of the [failure/lifetime lane](PXT-GC-FAILURE-LIFETIME-LANES.md).
[PR59](https://github.com/CrispStrobe/codal-samd/pull/59) landed after all eight
enabled exact-head checks and seven independent campaign audits passed. Both
new callers executed in both scheduler profiles. The cap case observes the
selected fatal refusal; the conversion case demonstrates that the selected
`SIZE_MAX` request is not refused. No allocator-policy correction was included.

## Qualified guard correction and boundaries

The new public adapter adds a guard in the GC branch of `malloc(size_t)`, before
`app_alloc(int)` and array alignment/header arithmetic. Requests larger than
`GC_MAX_ALLOC_SIZE - sizeof(void *)` select the existing fatal reason 22.
A compile-time assertion requires the selected cap to contain a header word and
be word aligned. The selected 131072-byte cap therefore admits payloads through
131068 bytes. The ISR/in-GC `xmalloc` route remains before this guard. Zero/null,
permanent-array ownership and the earlier payload-copy correction are preserved.

The hosted qualification matrix now selects seven oversized final requests:
131069, 131072, 2147483644, 2147483647, 2147483648, 4294967292 and 4294967295.
Each fresh caller first checks zero, retains a 32-byte payload, and allocates,
fills and frees 1-byte, 131065-byte and 131068-byte payloads. The oversized
request must reach public `malloc` and fatal reason 22 without entering
`app_alloc`. A separate fresh ordinary recovery caller then completes those
boundaries followed by a filled 17-byte allocation and matching free. It does
not recover within a guest instance that has already entered a fatal panic.

Both scheduler profiles observed actual calls/returns, matching frees,
ownership headers and every live payload byte, using the unchanged 12000000-step bound.
Source mutants remove the guard, exclude the valid maximum, change the refusal
reason, or remove permanent ownership; all 16 mutation/profile cases failed
at their named actual wrong observations. No timeout, compile failure or
exhausted bound was admitted. Dependency pins and engine sources are unchanged.

The seven oversized cases stop before signed conversion. The accepted maximum
remains live and byte-correct before its real matching free. The separate fresh
recovery case is an ordinary new instance, not resumption after fatal panic.

| Case | Final request | Steps default / event scheduler | Boundaries / frees | Final observation |
| --- | ---: | --- | --- | --- |
| `cap` | 131069 | 1259868 / 1259868 | 4 / 3 | Panic 22 before `app_alloc`, no final pointer |
| `cap-exact` | 131072 | 1259868 / 1259868 | 4 / 3 | Panic 22 before `app_alloc`, no final pointer |
| `signed-aligned` | 2147483644 | 1259868 / 1259868 | 4 / 3 | Panic 22 before `app_alloc`, no final pointer |
| `signed-max` | 2147483647 | 1259868 / 1259868 | 4 / 3 | Panic 22 before `app_alloc`, no final pointer |
| `signed-over` | 2147483648 | 1259868 / 1259868 | 4 / 3 | Panic 22 before `app_alloc`, no final pointer |
| `wrap-aligned` | 4294967292 | 1259868 / 1259868 | 4 / 3 | Panic 22 before `app_alloc`, no final pointer |
| `conversion` | 4294967295 | 1259868 / 1259868 | 4 / 3 | Panic 22 before `app_alloc`, no final pointer |
| `recovery` | 17 | 1727832 / 1727832 | 5 / 4 | Normal return, filled payload and fourth matching free |

Reviewed source: `0e0b56f2c357e2cd290c0037333ba34dbcb763ab`. Actual tested merge: `e479f3bdc43e6a52039c05ac7d6d88ff3df290fb`.
Landed merge: `4096ff8ab1c503fedb0b2519146644460b630406`. Reviewed, predicted, tested and landed tree:
`05160fd52ebc718b089772da0c6f5df73f9753a4`. Ordered merge parents: `8532a111414fb7006ba0d9e9f7b4a301b984bf60`,
then the reviewed source. All 14 enabled exact-head checks passed, including
five earlier affected campaigns and this eight-case size matrix. Standalone
readers independently admitted all 13 campaign packets in both profiles, their
38 actual mutation/profile cases, exact source/image/command/dependency/notices
and original raw streams. Official artifacts/logs were acquired once; no
successful guest was replayed merely to inspect its result.

The [official corrected-size run](https://github.com/CrispStrobe/codal-samd/actions/runs/38911675005)
binds these finite observations. The initial observer overflow on 4294967292 is
preserved as a separate failure; the corrected observer aligns an argument only
at an actual inner allocator entry. That original failure is not refusal evidence.

## Historical separate callers and observed boundaries

The builder explicitly selects `public-size-cap` or `public-size-conversion`.
Each uses the existing reset, static constructors and unchanged held dependency
pins. No allocator implementation, cap, panic policy or previous caller changes.
Each first checks public zero allocation, retains a 32-byte permanent allocation,
then allocates, fills and frees 17-byte, 33-byte and 131068-byte arrays. The
observer checks all live payload bytes and actual ownership headers at each
boundary and the surviving payload after actual collection returns and terminal
publication. The three temporary frees must traverse the real public and
`app_free` entries and complete their actual returns before terminal admission.

The selected cap in `pxtcore.h` is 131072 bytes. After alignment and the four-byte
array header, a 131068-byte payload exactly fits and all its payload bytes were
observed intact. The cap caller then requests 131069 bytes, reaches the actual
`gcAllocate(131076)` entry and `target_panic(22)`, followed by status
`0xe0000016`. This is a per-allocation cap refusal, separate from aggregate OOM.

The conversion caller instead finally requests the selected 32-bit `SIZE_MAX`,
4294967295. The retained public signature accepts `size_t`; `app_alloc` and
`gcAllocateArray` accept `int`. Selected source tests only zero before alignment
and header addition. Both profiles observed actual arguments `0xffffffff` at the
first three chain entries and 4 at `gcAllocate`, followed by a nonnull public
return with permanent array header `0x60000004`. The selected source therefore
returns a four-byte array block, including its header, for this oversized public
request. The caller never writes or reads the final requested payload range.
Its normal diagnostic terminal records the returned pointer; it does not declare
the request satisfied. This is an observed conversion/refusal defect at the
tested boundary, not an observed payload overrun or exhaustive integer analysis.

| Case | Steps in each profile | Completed boundaries / frees | Actual collections | Final observation |
| --- | ---: | --- | ---: | --- |
| `public-size-cap` | 399306 | 4 / 3 | 1 | Panic 22, status `0xe0000016`, no final return |
| `public-size-conversion` | 867133 | 4 / 3 | 2 | Nonnull public return, header `0x60000004`, diagnostic status `0x600d` |

At every completed allocation boundary, after actual collection returns where a
live payload exists, and at the terminal, the observer checked the surviving
32-byte allocation. It also checked every byte of each live temporary payload
before its real matching free. Both profiles retained the real public and
`app_free` entries and completed returns for all three temporary allocations.

## Qualification and provenance

Both cases executed their own source-bound ELF within the unchanged
12000000-step bound. The original streams and source-bound fixture cover real
startup/data/BSS/constructors, thread context, all allocator chain entries, LR
returns, exact arguments, ownership/capacity headers, real frees, surviving
bytes and fault registers. Neither case exhausted the raw heap.

The cap's isolated source mutant changes only its unique too-big panic reason
from 22 to 21 and exposed actual reason 21 at the final cap request. The
conversion case's isolated mutant removes permanent ownership and exposed
the first 32-byte allocation's actual header `0x40000024` instead of
`0x60000024`. That ownership mutant stops before the final oversized request.
Both mutants failed their named actual observation in both profiles, four
mutation/profile cases in total. No timeout or setup failure was admitted.

Reviewed source: `47626ebce3fd799dd1f4c95217dc1001ca0294fe`.
Actual tested merge: `d802bda1aa5c9f7c15f9d1bfff580e4157090529`.
Landed merge: `bfa249c0f0d09df1fecd2e3829dbac25778406cf`.
The reviewed, predicted, tested and landed tree is
`5a4145b822d2b418a3082d447ee2f996ea247f45`; ordered merge parents are
`3646f332c412e43cc1dbecdda94a3bfc8a0e363d`, then the reviewed source.
The [official hosted run](https://github.com/CrispStrobe/codal-samd/actions/runs/38886674277)
contains both new jobs. The independent readers froze before acquiring each
official artifact and job log once, after exact metadata admission. They audited
source adaptations, all 47 dependency identities per new case, commands,
notices, fixture/checkout bindings, ELF-to-binary reconstruction and raw streams.
Their 72 control methods passed before original acquisition.

| Image | SHA-256 |
| --- | --- |
| Cap ELF | `e641f9757c2a8d11230d7e4313f39ca7566b9ad651711e995a0ff5605b8e9534` |
| Cap binary | `727f9bde37729599d841140ef1c4015e09ec2373eea90e83e3b83ec3215aef59` |
| Conversion ELF | `187531bb6f0470983e2b2134ea55a4cb1b1a364515c37ae1e14a0f25d6486246` |
| Conversion binary | `362e3e8abe52c055a1253aed80da7e6736033aa303f4b7e9642c4d5c95d029fb` |

All five earlier affected campaigns requalified on this exact source: the
60-check baseline, 23-check routing, 662-check/308-boundary churn, native fatal
OOM and public fatal OOM callers. Their 22 actual source-mutation/profile cases
also passed independent audits. Original earlier failures remain documented in
the linked checkpoints; no successful original was rerun merely to read it.

## Next implementation and remaining scope

The finite public-size correction and its source mutants close A5 and this
slice of A6. Next implement allocated DMA/TXC/CS/callback lifetime under the
explicit borrowing contract, preserving these exact allocator regressions.
Full board/services/storage, generated native Arcade, representative RTx and
installed Lite adoption remain later work. These finite observations do not
qualify those outcomes, exhaustive integer execution or hardware accuracy.
