# Actual NVM consumer compilation — prospective bounded checkpoint

This separately selected **compile-only** profile builds unchanged real CODAL
`source/drivers/KeyValueStorage.cpp` against current `inc/SAMDNVM.h` declarations.
It uses the already-qualified service core
`2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`, not a silent upgrade to the newer
board's core. No positive hosted result is claimed yet.

The compiler force-includes the actual SAMDNVM public header, checking its
virtual overrides while compiling the real caller. A separate small translation
unit checks exact read/write/erase/address member signatures and non-abstract
inheritance. It neither instantiates a fake controller nor implements storage.

Two actual compiler refusal controls are mandatory: the historical core
`312ae57e0b31f5b9df07a81e9d846945828e3c5a` must refuse the conflicting
SAMDNVM address return type, and omitting the explicit compile selector must
refuse with its named diagnostic. An arbitrary compiler failure is not enough.

## No ASF fallback

The entry is hosted-only. It requires clean immutable core checkouts and clean
tracked Runtime source. `-nostdinc -nostdinc++` disable implicit toolchain include
search; only explicitly listed core/runtime directories and the unchanged
previously reviewed GCC `stdbool.h` are available. Actual compiler dependency
rules must resolve to admitted own files or immutable core `inc`/`source`
files. External, unknown and vendor headers refuse. Every selected dependency
is checked byte-for-byte against its Git source and recorded with SHA256.

The small compile-only platform header reuses the unchanged service-profile
declarations and adds C ABI declarations for `strlen` and `strcmp`, which the
actual caller uses. These are declarations, not linked success stubs. The
result must retain unresolved references to those functions. This is not the
full board configuration; its disabled service-profile USB selection must not
be substituted for the original application's required USB/WebUSB paths.

Run the pure controls without an ARM compiler:

```sh
python3 -B -m unittest discover -s consumer-build -p 'test_*.py' -v
```

The hosted workflow preserves compiler identity, dependency/object hashes,
undefined symbols, exact selected source/header bytes and MIT/GCC-exception
notices. It does not upload objects or a runnable image. Preserved notices do
not establish whole-image licence clearance. Failed compiler diagnostics remain
available; do not weaken source admission or replace actual callers to get green.

## Boundary and continuation

This is one actual current-core NVM API consumer, not the historical PXT
`ZFlash`/RAFFS/settings route identified by the
[full historical migration inventory](https://github.com/CrispStrobe/codal-samd/pull/38).
It does not implement SAMD51 NVMCTRL/CMCC, compile the complete board or PXT
application, resolve all serial/JACDAC consumers, link/execute firmware, qualify
flash persistence or establish frames/input/debugger/RTx. `linked`, `executed`,
`storage_model_qualified`, `image_admitted` and `licence_clearance` remain false.

Next close actual PXT storage and remaining board consumers against a declared
compatible profile, then implement required engine peripherals with affected
guest qualification. Existing service fixtures, default CMake and app pins
are unchanged by this compilation tool.
