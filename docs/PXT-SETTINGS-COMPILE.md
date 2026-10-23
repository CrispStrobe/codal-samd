# Real settings and complete PXT base types: qualified compilation

This hosted compile-only profile uses the [immutable public framework bindings](PXT-STORAGE-SOURCE-BINDINGS.md).
The source-bound hosted compilation and independent original-receipt audit now
pass. This is a separate checkpoint from qualified RAFFS/flash compilation,
not GC implementation, linked storage or an executable native Arcade runtime.

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

The real `CodalCompat.h` is included before unchanged `pxtcore.h` defines its
`itoa` compatibility macro. A bounded local object-only diagnostic first failed
on that declaration ordering; including the actual CODAL header fixes it without
substituting a declaration, removing the macro or weakening an ABI assertion.
That diagnostic is not hosted source admission, linking or execution evidence.
Hosted CI also requires the original ordering failure as a third compiler
refusal: removing only that adapter include must reproduce the actual `itoa`
declaration diagnostic. Positive caller and ABI assertions remain unchanged.

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

## Landed source-bound checkpoint — 2026-10-15

The original source `4eaaac7611bbec5f234222bf525aae4f0a33e71a` passed
[hosted compilation](https://github.com/CrispStrobe/codal-samd/actions/runs/38765901497)
and its separate documentation check. The frozen standalone receiver audited
the sole official artifact after admitting its provider/source metadata:
25 selected dependency identities, complete actual base bodies, selected ABI
assertions, real settings methods and unresolved dependencies, all three named
compiler refusals and component notices passed. No producer import, compiler or
guest replay was used to read that original.

The [unchanged original compile report](receipts/2026-10-15-pxt-settings-compile.json)
is 8,409 bytes, SHA-256
`ac93f96c3a0532cae5114ede21f1522ce87ea6d5a561e082fb0788b16adfcbba`.
Its original archive is 106,465 bytes, SHA-256
`cb352e0fffb09100cda3afab9d7dd880b44d434d8654bdd934b02eb5b21af43a`.
The report retains its actual tested checkout
`e31e6ea9301089a9dbc862401ac08038552f020a`; it is not relabelled as a later head.
Objects were not uploaded, so recorded object hashes are producer records, not
an independent binary rehash or execution result.

After integrating the landed RAFFS work and preserving both documentation
sections, final source `815a7cc9ad9d2c8b795336a59fe6acc7a7988816` passed both
enabled final-head checks, including
[the final actual compile job](https://github.com/CrispStrobe/codal-samd/actions/runs/38773036657).
Its original official job log binds the actual checkout
`8e4dcf6ea2ca2a365f073435560b2ae13e59a42f`, tree
`e3ab1df464692a33f2cb054534397bf99cd61067`, to the reviewed final head. All
selected producer inputs, original own dependency files and workflow bytes
remain unchanged from the independently audited source.

[PR45](https://github.com/CrispStrobe/codal-samd/pull/45) then landed at
`ffb76ca24637d5c3340f9af4b36dcc18a0935b55`. Predicted and landed tree
`874b7facbdf0a19677f0afacb8f5d358d2d6740d` and ordered parents
`72dbe58a541b5258fe8a887fa4273bc052a1ead3` /
`815a7cc9ad9d2c8b795336a59fe6acc7a7988816` match exactly. The tested-to-landed
difference is only the four already-qualified RAFFS documentation/receipt
paths; there is no claim of global tested/landed tree equality.

## Next runtime gate

This is the historical compile-stage plan. Selected
[actual RAFFS/settings execution](PXT-RAFFS-SETTINGS-RUNTIME.md),
[compaction](PXT-STORAGE-COMPACTION.md) and
[modeled reset](PXT-STORAGE-RESET.md) now have separate finite evidence.
Use [the remaining scoped queue](PXT-STORAGE-NEXT-STEPS.md) before starting;
preserve the original compile receipt and the limits below.


The [finite linked GC/allocation/DMA lane](PXT-GC-FAILURE-LIFETIME-LANES.md)
is now qualified and landed through PR69; its exact original results and finite
limits remain separate from these compile-only settings receipts. Continue with
the [pinned storage census and actual ZFlash execution slice](PXT-STORAGE-LINK-CENSUS.md),
then real RAFFS/settings execution and full required board configuration.
Native Arcade frames/input/debugger, representative RTx and installed adoption
remain later gates. No runtime settings or persistence result follows from this
object compilation.
