# Actual NVM consumer compilation — landed bounded checkpoint

This separately selected **compile-only** profile builds unchanged real CODAL
`source/drivers/KeyValueStorage.cpp` against current `inc/SAMDNVM.h` declarations.
It uses the already-qualified service core
`2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`, not a silent upgrade to the newer
board's core. Hosted compilation and the bounded receipt audit now pass.

## Exact passing result — 2026-10-15

[PR39](https://github.com/CrispStrobe/codal-samd/pull/39) landed at
`376bc0f177f57ef0070e2fa8fd68d1390631417a` after both enabled checks passed.
Reviewed source was `d99bad0b652c6391508d91b1856bf2dea851d219`; tested merge
`568a44476fc1c9bd11c7e24aba5bf8d62efc5f7a` and the landed merge share tree
`6dcb713a031e0b599cafffc8a119d9e15e695211`. Landed parents are
`cf71b16d9fe67a872811b6392a4a37386f020dd7` and that reviewed source.

The [actual hosted run38728020738](https://github.com/CrispStrobe/codal-samd/actions/runs/38728020738),
job116398576853, compiled the unchanged real caller and actual API assertions.
All 23 selected dependencies match pinned Git bytes; both specific compiler
refusals passed. Compiler: GNU Arm 13.2.1, package `15:13.2.rel1-2`.
The [unchanged compile report](receipts/2026-10-15-nvm-consumer-compile.json)
retains the exact dependency and object hashes and explicit false admission
flags. Objects were not distributed in the artifact and no firmware was linked.

The [official artifact11973637292](https://api.github.com/repos/CrispStrobe/codal-samd/actions/artifacts/11973637292/zip)
was acquired once after provider admission: 67124 bytes, SHA256
`f956a756099d37394364693297e6583738d308f2c6f082689cd6509e15687b45`.
A standalone receiver checked source/job/tested-tree binding, official artifact
digest, selected Git blobs, notices, strict qualification flags, unresolved libc
and specific refusal diagnostics. Its frozen predicates and metadata-only
integration binding passed synthetic refusal controls before acquisition.
This is a same-workstream receipt audit, not an independent compiler rebuild,
human review, storage implementation or whole-image licence clearance.

The compiler force-includes the actual SAMDNVM public header, checking its
virtual overrides while compiling the real caller. A separate small translation
unit checks exact read/write/erase/address member signatures and non-abstract
inheritance. It neither instantiates a fake controller nor implements storage.

Two actual compiler refusal controls remain mandatory: the historical core
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
