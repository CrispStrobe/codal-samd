# Real settings and complete PXT base types: prospective compilation

This hosted compile-only profile uses the [immutable public framework bindings](PXT-STORAGE-SOURCE-BINDINGS.md).
**No hosted settings result is claimed yet.** It does not assume the pending
RAFFS compilation is already qualified or repeat the qualified flash build.

## Actual types, not invented layouts

The [builder](../pxt-settings-compile/compile_settings.py) preserves the complete
actual `pxtbase.h` body: classes, fields, inheritance, aliases, inline methods,
GC helpers, namespaces and non-include macros remain unchanged. Seven include
lines select an explicitly named freestanding header instead of standard headers
and the board platform. This counts the integer include in the unselected VM
branch; the actual profile is SAMD51, not PXT_VM. The adapted header has the
distinct name `pxtbase-storage.h`.

Settings changes two includes: `pxt.h` selects that actual adapted base header;
the unrelated `GhostFAT.h` include selects actual `CodalDevice.h`. Its algorithm
body stays unchanged. Real `Flash.h`, `RAFFS.h`, `pxtcore.h` and `configkeys.h`
compile unchanged. Original/adapted bytes and hashes are retained. This is never
reported as unchanged PXT or compilation of the full SAMD platform/USB stack.

The freestanding header contains compiler-native primitive types/macros and
C/C++ ABI declarations only. It implements no PXT type, GC, allocation, libc,
math or device behaviour. Actual UTF8 string methods and GC root wrappers compile
from the complete public base header. A separate [ABI assertion unit](../pxt-settings-compile/settings-abi.cpp)
checks selected real 32-bit object/segment/collection/buffer sizes, aliases,
inheritance and UTF8 selection; this is not runtime GC interoperability.

`pxtconfig.h` uses the already-reviewed pinned public-generator reconstruction:
UTF8 enabled and UF2 family `0x55114460`, matching the original 78 bytes. No full
TypeScript generator/shim scan runs; unknown application/table bodies are not
staged. Both PXT component notices, CODAL and reviewed compiler notices remain.

## Required checks and limits

Implicit compiler include paths are disabled. Exact clean pins and closed
dependency-origin/byte checks are required; unknown/compiler/vendor headers
refuse. Omitting the settings selector must refuse with its named diagnostic;
the original caller must specifically fail for missing `pxt.h`, not silently
resolve through a substituted PXT header.

Positive compilation must emit real set/get/remove/exists/list/clean/large-store
methods and retain unresolved real string/buffer creation, GC root registration,
collection growth, configuration and RAFFS dependencies. Source/ABI probe,
compiler identity/flags, symbols, diagnostics and notices are preserved, not
objects or a firmware binary. No GC or filesystem success stub is linked.

```sh
python3 -B -m unittest discover -s pxt-settings-compile -p 'test_*.py' -v
```

This explicit storage-only compile profile uses service declarations with
disabled USB/WebUSB, not the original enabled USB/WebUSB board configuration.
Full SAMD interfaces, entry/shim tables, GC implementation/linkage, linked image,
scheduler/flash behaviour, NVMCTRL/CMCC model, persistence, native Arcade, RTx and
whole-image licensing remain unqualified. Existing engine/app pins, guest
fixtures, captures and hardware acknowledgements do not change.
