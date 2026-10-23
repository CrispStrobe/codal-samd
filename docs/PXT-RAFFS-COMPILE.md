# Real RAFFS: qualified include-adapted compilation

The [flash driver checkpoint](PXT-FLASH-COMPILE.md) is separate from compiling
the filesystem above it. This new hosted-only profile selects actual immutable
common-packages14.1.4 `RAFFS.cpp`, unchanged `RAFFS.h` and `Flash.h`, and the
existing declared CODAL core pin. Hosted compilation and the independent receipt
audit passed within the boundary below. This is not a filesystem execution,
persistence or full PXT application test.

## Reached checkpoint — 2026-10-15

[Actual run38761091292](https://github.com/CrispStrobe/codal-samd/actions/runs/38761091292),
job116512239541, passed at reviewed source
`62eb035ac760dd9f3c22c40043b46b0c5c2001bb`. Its tested merge was
`0bf419f0bd468ffc332c769d1bcd68e388324b28`, tree
`ff3e9970cca42fd148d7424f9547422f6d2b2d65`, with ordered parents
`2cd3bfe88c72c70e0206bda4f8d924b4a2a52073` and that reviewed source.

The compiler emitted the required real RAFFS methods and retained unresolved
hash, scheduler, timer, notification and libc dependencies. Both named compiler
refusals passed. All 24 selected dependency identities passed the separate
receipt/source audit. The unchanged [original compile report](receipts/2026-10-15-pxt-raffs-compile.json)
is 7665 bytes, SHA256
`26352b5975d78b69f2950ed55c728ff291c27b0f7981601fee4ce50626dbc6e2`.
Its producer-recorded object SHA256 is
`1afa53a037e50264008af7b9b6f5b0909466fbccb6353ca889d37696dd4a759c`;
objects were not uploaded, so the receiver did not rehash or rebuild a binary.

A standalone stdlib receiver, frozen before reading the actual artifact, passed
two synthetic positives and 51 refusal controls. It admitted the successful
run/job/source/tested-tree metadata before acquiring the sole
[official artifact11988204975](https://api.github.com/repos/CrispStrobe/codal-samd/actions/artifacts/11988204975/zip)
(87452 bytes, SHA256
`5ac5afed4f50b76750c3acf6b969d2c25410b627b8da31231ae3d0898616f84d`).
It checked archive identity, source/adaptation/extracted-declaration bytes,
selected dependencies, notices, symbols, specific diagnostics and narrow flags.
No producer module, compiler or guest was used by that receiver. This is not
independent source-to-binary, scheduler/flash execution or licence clearance.

[PR44](https://github.com/CrispStrobe/codal-samd/pull/44) merged as
`58768ed9910bafb276e0491b815360e7af6063fc` after both enabled exact-head checks
succeeded. Landed tree `2372d49da9bc3a040732020db5b84ef9d4ed034a` equals the
independently predicted integration tree; ordered parents are
`b8967da9a7cc84399a3aa3242ea892c968c714f6` and reviewed source
`62eb035ac760dd9f3c22c40043b46b0c5c2001bb`. Compared with the tested source,
only PR43's four prior flash-results documentation/receipt paths changed.
Executable inputs are unchanged; global tested/landed tree equality is not claimed.

## Narrow, recorded source adaptation

The [builder](../pxt-raffs-compile/compile_raffs.py) verifies the original public
bytes with the [existing source bindings](PXT-STORAGE-SOURCE-BINDINGS.md).
It changes exactly two include lines in separately named `RAFFS-storage.cpp`:
the `pxtbase.h` and standard `stddef.h` includes select the explicitly named
`pxt-raffs-platform.h`. The algorithm body and real RAFFS/Flash interfaces remain
unchanged. Original/adapted source and hashes are retained.

The adapter supplies compiler-native `intptr_t`/`offsetof`, a `strlen` C ABI
declaration, and the exact hash-function declaration extracted from the verified
public `pxtbase.h`, preserving its namespace exposure. It implements no hash,
filesystem, flash, scheduler, event, timer, allocation or libc function. It does
not create a substitute named `pxt.h`, `pxtbase.h` or `stddef.h`. Actual CODAL
event/timer/fiber declarations and RAFFS classes must compile; invented PXT
String/Buffer/GC layouts are not admitted.

All implicit compiler include paths are disabled. Selected dependency bytes
must match the closed own-header set or immutable public source; unknown,
compiler and vendor headers refuse. Original PXT input and extracted declaration
bytes retain their component MIT notice, alongside CODAL and reviewed compiler
notices. This is not full licensing clearance.

## Mandatory compiler checks and limits

The positive compile must emit real constructor/write/read/remove/format/GC/
directory/raw-read methods and retain unresolved real hash, scheduler, timer,
notification and libc dependencies. Actual defined and undefined symbol lists,
compiler identity/flags, dependency hashes, original/adapted files, extracted
declaration and notices are preserved. No object or firmware is published.

Two named compiler refusals are required: omitting the explicit RAFFS selector
and compiling the original source without `pxtbase.h`. An arbitrary failure is
not success, and an unadapted success through a replacement PXT header refuses.
This does not repeat the already-qualified flash-driver compilation.

Pure controls, without a compiler:

```sh
python3 -B -m unittest discover -s pxt-raffs-compile -p 'test_*.py' -v
```

The selected service-only profile disables USB/WebUSB and is **not the original
board configuration**. Full PXT shared headers, settings String/Buffer/collection
and GC ABI, actual hash implementation/linkage, scheduler/flash execution,
NVMCTRL/CMCC model, persistence, native Arcade and RTx remain unqualified here.
The next settings build must preserve real PXT object types and GC contracts,
not invent layouts just to make a translation unit compile. Existing engine,
package pins, guest fixtures, captures and hardware acknowledgements do not change.

## Next linked-runtime gates

This is the historical compile-stage plan. Selected
[actual RAFFS/settings execution](PXT-RAFFS-SETTINGS-RUNTIME.md),
[compaction](PXT-STORAGE-COMPACTION.md) and
[modeled reset](PXT-STORAGE-RESET.md) now have separate finite evidence.
Use [the remaining scoped queue](PXT-STORAGE-NEXT-STEPS.md) before starting;
preserve the original compile receipt and the limits below.
 — not satisfied by these object files

The real [PXT GC source](https://github.com/microsoft/pxt-common-packages/blob/31abf23d118f35010fb75122e60a1eb2b6dffe9f/libs/base/gc.cpp)
needs more than the settings type declarations. A bounded object-only diagnostic
first refused the missing exception-state dependency. A private adapter reading
the actual Cortex-M exception number allowed that real GC source to compile;
no GC implementation, firmware link or guest execution is qualified by this.
The following are required next work, not completed runtime capabilities:

1. Provide real exception-context observation without ASF. ARM's
   [IPSR contract](https://arm-software.github.io/CMSIS_6/main/Core/group__Core__Register__gr.html)
   identifies the current exception in bits8:0. Do not replace `PXT_IN_ISR()`
   with a constant. Fresh guests must cover thread, handler and restored thread
   context through actual GC refusal paths before accepting this integration.
2. Resolve allocator ownership explicitly. With actual `pxtcore.h` selecting
   `GC_GET_HEAP_SIZE`, `gc.cpp` defines C `malloc`, `free` and `realloc`; the
   service-only [aligned allocator](../runtime-services/aligned-heap.cpp) defines
   the same symbols. They cannot both be selected unchanged. Record the selected
   public/GC/device allocator domains and migration patch; do not use duplicate-
   symbol linker allowances or silent macro removal. Fresh actual tests must
   cover alignment, roots/liveness, reallocation failure and deallocation in the
   right domain. Earlier service-only allocator results do not qualify the GC.
3. Use the real thread-context and stack-root scanning implementations in
   [SAMD codal.cpp](https://github.com/microsoft/pxt-common-packages/blob/31abf23d118f35010fb75122e60a1eb2b6dffe9f/libs/core---samd/codal.cpp).
   This source already contains the `get_fiber_list` compatibility bridge for
   the pinned CODAL API. Do not invent an empty fiber list or GC stack scan, or
   upgrade the core merely because its old `list_fibers` name is absent. Qualify
   actual suspended/current fiber roots, context lifetime and resumed execution.
4. Close real object/segment methods, buffer/string construction, configuration,
   globals and bytecode symbols with admitted implementations and a declared
   program/build profile. A successful object compile is not a successful link.
   Preserve selected sources/notices and reject unresolved required services.
5. Then qualify the original required board services or explicit, documented
   profile changes, NVMCTRL/CMCC behaviour, storage and the native Arcade
   boot/frame/input/debugger path. Only that runtime can support representative
   native/WASM measurements and installed-app adoption. Keep the complete
   roadmap's performance/adoption/optimization requirements open until observed.
