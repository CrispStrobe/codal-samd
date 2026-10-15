# Public PXT storage input bindings

The [source profile](../profiles/pxt-framework-sources.json) binds 64 framework
C/C++ source/header files from the historical request to exact public Git blobs,
including the real `SAMDFlash.cpp`, `RAFFS.cpp`, `settings.cpp`, `Flash.h`,
`RAFFS.h`, `pxt.h`, `pxtbase.h` and SAMD `platform.h`.

This is source-origin/staging work, **not a new compilation or image admission**.
The prior [actual NVM consumer compilation](NVM-CONSUMER-COMPILE.md) qualifies
the separate CODAL KeyValueStorage caller, not this PXT path.

## Immutable source and scope

The original Arcade4.2.1 package declares common-packages14.1.4 and core13.2.1
in its [versioned package metadata](https://github.com/microsoft/pxt-arcade/blob/v4.2.1/package.json).
Common-packages tag14.1.4 resolves to commit
`31abf23d118f35010fb75122e60a1eb2b6dffe9f`, tree
`ca0b059d1142cdc91ae96181268d8e2d6de65e3c`. The bindings were checked against
that exact public tree and the retained original request, not inferred from
matching filenames or an unpinned latest version.

All 64 matched files have one public source-path match each. Their Git blob,
SHA256 and byte count are recorded. The request has 69 replacement entries:
64 public framework matches, three unmatched C/C++ inputs (`pointers.cpp`,
`pxtconfig.h`, `main.cpp`), plus `codal.json` and `functions.json`. All five
unmatched entries remain explicitly unresolved metadata; no generated bodies
or application program are published or staged. This explains the earlier
67 C/C++ dependency-origin census without treating non-C/C++ metadata as code.

The [component MIT notice](https://github.com/microsoft/pxt-common-packages/blob/31abf23d118f35010fb75122e60a1eb2b6dffe9f/LICENSE)
is bound to Git blob `4b1ad51b2f0efc36f38aa3349a9f30fbd9217547` and preserved
unchanged when staging. A component notice and exact bytes do not close
per-file/transitive/generated/toolchain obligations or whole-image licensing.
`licence_clearance` remains false. No vendor header is relabelled or copied.

## Reproducible check and staging

With an already checked-out public source at the exact commit:

```sh
python3 -B -m unittest discover -s tools/pxt-storage -p 'test_*.py' -v
python3 -B tools/pxt-storage/sources.py --verify-checkout owned-pxt-framework --stage staged-pxt-framework
```

The destination must not exist. Every source is checked against the profile,
its actual pinned Git bytes and both hashes before any output is written.
Paths must remain inside the selected checkout; mutations, unknown/missing
sources, duplicate JSON keys, altered notices or broadened admission flags
refuse. Only the 64 public framework files and their component notice stage;
no unresolved request input, private artifact, binary or vendor tree is fetched.
Hosted CI checks this against public Git and does not repeat an original build
or download the historical artifact.

`--require-consumer-build` must still refuse. A passing source-binding job is
not a successful PXT consumer build, header ABI check, storage implementation,
guest, native Arcade frame or RTx measurement.

## Next actual build work

### Two fixed generated inputs

The supplemental [reconstruction tool](../tools/pxt-storage/generated.py) uses
the public PXT13.2.1 [generator](https://github.com/microsoft/pxt/blob/c40f412dc5bebb915a5891cd62fe956e198fb194/pxtlib/cpp.ts)
at commit `c40f412dc5bebb915a5891cd62fe956e198fb194`, tree
`3e5a442c4265eaf2e5b2495eb68539b86841716e`. It verifies the unchanged generator
Git blob `e2d5c77c6f67c8f93c4efee4cdf098a2fdde1ff9` and its component MIT
notice before reconstructing two files. This is a narrow Python reconstruction,
**not execution of the TypeScript generator or its package/shim scanner**.

For `pxtconfig.h`, the selected values are UTF8 enabled, UF2 family
`0x55114460`, and no other emitted options. The public generator supplies the
preamble and emission rules. These values reproduce the retained original
78 bytes; they are not proof of a complete original options object, the
generated CODAL definitions, USB configuration or a compatible new board profile.
For `main.cpp`, the tool extracts the public generator's literal template;
the resulting 187 bytes also match the original Git blob, SHA256 and size.
Neither file contains a copied unknown application body. The generator's
component notice is preserved unchanged in the separate output directory.

```sh
python3 -B tools/pxt-storage/generated.py --verify-checkout owned-pxt-generator --stage staged-pxt-generated
```

The destination must be new; altered generator/notice/output bytes, extra
outputs and existing destinations refuse before any write. Hosted CI performs
the positive reconstruction against public Git. The original 64/5 census stays
unchanged as an historical record; this supplemental two-file reconstruction
does not admit `pointers.cpp`, `codal.json` or `functions.json`. The full generated
input set, consumer build, image and licensing clearance remain unqualified.

### Remaining closure

The [qualified include-adapted real flash compile profile](PXT-FLASH-COMPILE.md)
isolates the actual hardware-driver translation unit without inventing a PXT
header. It is a declared one-line source adaptation, not closure of the full
shared-header or board configuration requirements below.

The separate [real RAFFS compilation profile](PXT-RAFFS-COMPILE.md) continues
above that driver with a declared include adaptation and extracted public hash
declaration. The hosted RAFFS compile and separate receipt audit now pass; see
that guide's source-bound results. Settings/GC compilation and filesystem
execution remain separate requirements.

The [settings/base-type compilation profile](PXT-SETTINGS-COMPILE.md) preserves
the complete real PXT base class and GC declaration bodies with explicit
include-only adaptations. Its hosted compile, selected ABI assertions, 25-input
receipt audit and three specific compiler refusals now pass. This is not GC
implementation, original SAMD board configuration or linked storage execution.

1. Qualify the generated configuration and entry/table inputs separately; do
   not copy unmatched bodies or invent storage/application state.
2. Preserve the actual PXT shared headers. Its SAMD platform includes GPIO,
   SPI/I2C, DAC/PDM, serial/single-wire and NVM interfaces even for the selected
   storage translation units. Port the required no-ASF header/API closure
   explicitly, compiling real callers rather than substituting a PXT stub.
3. The historical generated board configuration enables USB/WebUSB and SAMD51.
   Do not silently use the service fixture's disabled-USB profile or claim
   compatibility after excluding a required service. Declare a new compatible
   core/board/profile and actual changes; keep existing qualified pins intact.
4. Admit actual toolchain dependencies/notices, then compile/link the real
   ZFlash/RAFFS/settings route. Only after the required NVMCTRL/CMCC model and
   affected guest checks pass can flash behaviour or persistence be claimed.

No existing source selection, guest fixture, compiler flags, engine, package
pin, physical capture or performance threshold changes in this staging lane.
