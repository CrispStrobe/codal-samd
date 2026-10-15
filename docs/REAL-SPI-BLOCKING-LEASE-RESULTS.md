# Actual blocking SPI terminal-result ownership — 2026-10-15

Runtime [PR32](https://github.com/CrispStrobe/codal-samd/pull/32) merged as
`f9a6525e104e28e42c529e1da8718dc900744b4d` after both enabled checks passed.
Engine [PR237](https://github.com/CrispStrobe/labwired-core/pull/237) merged as
`068c2c1a370f266ac97399aff8592303ea423d6e` after all 23 enabled checks passed,
including all three workspace shards and final aggregates; only seven declared
optional jobs skipped. Reviewed/tested/landed trees, exact parents and default
refs were verified. Actual execution and the source/image-bound independent
original-log audit pass all four cases below. This is bounded caller evidence,
not a whole native Arcade image.

## Exact source and original evidence

- Runtime caller source: `0f952e5399c0723f8312b9f9a7d767272856d099`,
  [PR32](https://github.com/CrispStrobe/codal-samd/pull/32), reviewed tree
  `cd6385b653695ed55622dc739d97e045878650cf`.
- Engine/test source: `a6352784c056eb4b6f37caa795263357054acdba`,
  [PR237](https://github.com/CrispStrobe/labwired-core/pull/237), tested merge
  `1888e4c4dc1aa4b12218321b356215e0974dafa7`, reviewed/tested tree
  `12a2297840c8e80bc38d1dea333bd0b0a1f28fd9`.
- [Separate fifteen-mode source build38675007480](https://github.com/CrispStrobe/codal-samd/actions/runs/38675007480),
  job116226502218, passes. Previously qualified modes0–13 keep their ELF hashes;
  selected input counts are43 for modes0–7 and49 for modes8–14, with each mode's
  full dependency census. Driver/model and earlier callers are unchanged.
- [Final actual four-case run38678769414](https://github.com/CrispStrobe/labwired-core/actions/runs/38678769414),
  job116238283467, passes. The retained original job-log response is192635 bytes,
  SHA256 `0350cb9f7570ae066cd18c954697a756e0489c17a7b03f7a1245af97ce38615a`.
  Provider/head/base/tested-tree/parent/step identity was verified before its
  sole acquisition; the guest was not replayed to inspect evidence.
- Mode14 ELF: `e2cd69126a4ebaa9df092b02c7b934460668cd38187a2f73e3c2a4788ba8a24c`.
  Positive mode8 ELF: `a7f4d21f32f4c0f4e38fbb79b02bf4a31f5b3552f0c3cae353297128fac5e68e`.
  Each caller uses a separate clean immutable dependency tree.
- The independent standard-library reader was frozen before the original:
  final exact-head binding SHA256
  `dd10ed24e2691a96ee7f4970fd5fafaca972db05bd092e314c11faeacf61dd98`;
  immutable predicate SHA256
  `920cdcfb3fa3d1b47a7b717bdb31f5f1368ba8cbe3dfdb2fa62d21a9dfe17f54`.
  Synthetic controls reject154 altered cases,224 envelopes and24 provider
  packets. Actual parsing passes all four; no producer predicate import.

## Observed boundary

| Caller | Both default and event-scheduler profiles |
| --- | --- |
| Blocking cancellation/result ownership |158067 guest steps;31 prefix observations,28 positive recovery observations;10 actual context swaps. Two bytes accepted by DMA, no slave wire byte before CS release, frozen writeback remaining30 of32. |
| Unchanged positive control |126844 guest steps;28 observations;6 actual context swaps. Asynchronous and blocking transfers succeed. |

The actual owner enrolls notification value1024 through `fiber_wake_on_event`.
A distinct worker cancels with interrupts masked. The actual NotifyOne event
uses source1022 and that same value, returning through the real event path.
The worker's valid competing transfer returns DEVICE_BUSY before the original
owner resumes and consumes DEVICE_SPI_ERROR (`-1014`). No asynchronous failure
callback is invoked. The original owner returns on its original live fiber;
both worker returns and the owner's actual Event58/1 path precede main resume.
The constants come from immutable CODAL
[ErrorNo.h](https://github.com/lancaster-university/codal-core/blob/2ed19d64ec3359143409a4b0a9758b4aeef3ee7d/inc/core/ErrorNo.h)
and [CodalComponent.h](https://github.com/lancaster-university/codal-core/blob/2ed19d64ec3359143409a4b0a9758b4aeef3ee7d/inc/core/CodalComponent.h),
not guessed error or notification values.
Shared trampoline return PC5846 is disambiguated by live `currentFiber`, not PC
alone. Fiber-address reuse during later recovery does not imply simultaneous
identity or whole-lifetime qualification.

The direct settled diagnostic at step91831 matches the final lease witnesses,
main-resume observation, notification value, thread context and unmasked state.
It records four actual swaps and zero DMA/SPI entries and returns before
recovery. Strict cleanup is SPI control `0x2000c`, interrupt enable0, channel
`0x200d00` and CS released. Every accepted/wire byte, channel-disable byte,
actual helper return and enrollment/notification role remains in the raw prefix.
Wire progress is deliberately allowed from zero through accepted count; the
observed zero does not establish a universal zero-wire cancellation promise.

Recovery then reaches two DMA entries/returns and two SPI entries/returns, exactly one
successful asynchronous callback, real Event/main resume and a successful
distinct blocking transfer. Both profiles produce the same observed result.
Guest step counts are not execution time or RTx.

## Preserved development failures

The [first compile failure38675342285](https://github.com/CrispStrobe/labwired-core/actions/runs/38675342285)
was E0716 in formatted ELF symbol-name borrowing; an explicit `&str` annotation
fixed the test lookup. [Run38676746833](https://github.com/CrispStrobe/labwired-core/actions/runs/38676746833)
compiled, then exposed a mutation-test mistake: removing an optional wire row
was wrongly expected to refuse although zero wire progress is valid. Controls
now include that valid positive case and retain every required-role/accepted-byte
deletion, wrong-value/order and extra-effect refusal; the actual predicate was
not relaxed. Neither failed run executed a guest.

[Run38677084124](https://github.com/CrispStrobe/labwired-core/actions/runs/38677084124)
first passed all actual cases and the independent reader. Its broader default
check failed only Clippy's two alignment-expression style warnings; equivalent
`is_multiple_of(2/8)` assertions fixed them without suppression. The final
source-bound actual run above repeats all four successfully. Earlier originals
and readers remain unchanged; no failed record was normalized or erased.

## What remains separate

This finite slice establishes blocking terminal-result ownership under the
named cancellation scenario, not arbitrary concurrent access or destruction.
Keep driver, Timer, MessageBus and buffers alive. Next qualify the explicit
active/delivering destruction refusal and idle reconstruction policy, then
remaining board/API/build/configuration/notices/image/CF2 admission and actual
frames/input/debugger. Natural stale IRQs/timeouts, complete native Arcade,
active native/WASM RTx, installed consumer adoption, physical behavior and
distribution/licence clearance remain unqualified. No consumer pin, threshold,
acknowledgement or physical capture changes.
