# Real RAFFS: prospective include-adapted compilation

The [flash driver checkpoint](PXT-FLASH-COMPILE.md) is separate from compiling
the filesystem above it. This new hosted-only profile selects actual immutable
common-packages14.1.4 `RAFFS.cpp`, unchanged `RAFFS.h` and `Flash.h`, and the
existing declared CODAL core pin. **No positive hosted RAFFS result is claimed
yet.** This is not a filesystem, persistence or full PXT application test.

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
