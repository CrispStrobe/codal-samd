# Real PXT GC and CODAL services: prospective linked caller

This [hosted builder](../pxt-gc-runtime/build_gc.py) progresses beyond separate
translation-unit compilation. It selects the immutable public PXT GC and object
implementation, actual CODAL services and the approved
[allocator ownership split](PXT-NATIVE-ALLOCATION.md). No successful hosted link
or affected guest result is claimed yet. This is not a native Arcade application.

## Exact source selection

The existing closed public source profile and generator verification remain
mandatory. The complete real `gc.cpp` body is retained with one include change.
The complete real `pxt.cpp` body is retained with one include change and removal
of its unused `using namespace std` directive. The complete actual PXT base
header retains all type/field/method/GC definitions; only seven include lines
select explicitly named freestanding declarations and the real IPSR observation.
No substitute PXT classes or empty GC implementation is supplied.

The actual SAMD adapter's `list_fibers`, thread-context access, copied-stack
address mapping and `gcProcessStacks` functions are extracted byte-for-byte
between unique source boundaries. Actual configuration lookup and its original
weak bootloader fallback are extracted likewise from `base/core.cpp`. Excluding
unrelated board/application functions is explicit; this does not qualify the
complete SAMD adapter, board initialization or generated program inputs.

The authored probe supplies a declared small GC metadata/configuration fragment,
not a generated `pointers.cpp` or an Arcade executable. Actual configuration
lookup reads its 32 KiB system-heap reserve. The existing three exact CODAL
service patches remain selected at the unchanged core pin. Native C++ new/delete
and TCB allocation calls in a separately adapted target source use only the
named aligned domain; real PXT placement-new objects retain GC ownership.
Existing service source files and builds are not modified.

The new startup retains and invokes actual `.init_array` constructors before
entering the probe. Silently discarding GC static initialization is not allowed.
The linker requires no unresolved symbols; each object is checked so that only
actual `gc.cpp` defines the public `malloc/free/realloc` names. This does not
assert complete C library compatibility or general-purpose allocation semantics.

## Authored caller and qualification owed

The probe checks native alignment/reallocation preservation, simultaneous public
and native allocation contents, real managed objects across collections, global
roots and current/suspended fiber stack roots. Its C++ stack spans are published
explicitly to the actual thread context; they are not generated PXT ABI evidence.
All these checks still require clean hosted guest execution, independently
audited original artifacts and nonvacuous mutation controls. A successful link
alone does not qualify them. ISR/in-GC routing, exhaustive failure/churn cases
and DMA buffer lifetime remain additional affected tests before runtime adoption.

Pure controls:

```sh
python3 -B -m unittest discover -s pxt-gc-runtime -p 'test_*.py' -v
```

Original/adapted selected public sources, dependency hashes, compiler/link logs,
symbol lists, image digests and component notices are retained. The selected
profile disables USB/WebUSB and is not the original board configuration. Full
board services, flash/controller models, actual native Arcade frames/input/debug,
representative native/WASM RTx and installed-app adoption remain required.
No engine/package pin, physical capture or hardware acknowledgement changes.
