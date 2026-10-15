# PXT-owned public allocation, distinct native buffers

The linked native Arcade runtime will retain PXT's `malloc/free/realloc` from
the actual pinned `base/gc.cpp`. Its `GC_GET_HEAP_SIZE` configuration remains
enabled. Do not link the service-only `runtime-services/aligned-heap.cpp` into
that image: both implementations export those same public names. Do not hide
the conflict using duplicate-symbol linker flags or by disabling PXT's GC policy.

Existing service builds are unchanged. The separately selected
[native helper](../pxt-native-allocation/native-aligned-heap.cpp) exposes only
`bw_native_alloc`, `bw_native_free`, and `bw_native_realloc`. These retain the
existing service allocator's metadata, eight-byte alignment, arithmetic checks,
copy/reallocation semantics and actual raw CODAL `device_malloc/device_free`
backing, but do not replace the public allocation API. Both the existing
Cortex-M service selector and `BW_PXT_NATIVE_ALLOCATION` are required to compile.
This is not yet an enabled native Arcade build profile.

## Ownership rules

- PXT managed objects and public allocations follow the real PXT GC API.
- Raw CODAL pointers use their matching `device_*` APIs.
- Native aligned buffers use only the three `bw_native_*` APIs. They are not
  traced GC roots; retain any referenced managed object through real PXT roots.
- Never pass pointers between these ownership domains. Metadata sanity checks
  do not authenticate arbitrary or already-freed pointers.
- Native reallocation preserves the old buffer on allocation failure. Actual
  raw-heap exhaustion can panic under the retained CODAL configuration; this
  change does not promise recoverable out-of-memory behavior.

Pure source controls (not allocation execution):

```sh
python3 -B -m unittest discover -s pxt-native-allocation -p 'test_*.py' -v
```

## Required linked guest checkpoint

The next builder must select real PXT GC as the sole public allocation owner,
explicitly include this native helper only if needed, and reject duplicate
public definitions. Preserve actual PXT object/collection implementations and
the actual CODAL fiber root scanner; empty roots or substitute types are not
acceptable. A real admitted program/configuration and a finite RAM budget must
be bound to the linked image. A source/object check alone does not admit it.

Fresh affected guest tests must exercise both domains together: alignment and
contents across churn; zero/overflow cases; grow/shrink/reallocation-failure
preservation; free/reuse; ISR and in-GC public allocation routing; live managed
references across collections on current and suspended fibers; native buffer
lifetime across asynchronous DMA and final TX completion. Keep original failure
results. Existing service-only allocator captures do not qualify this profile.
Full board/model execution, native frames/input/debugger, representative RTx
measurements and installed adoption remain separate required milestones.
