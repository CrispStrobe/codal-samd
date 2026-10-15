# Native board consumer closure

## Reached: complete historical C/C++ dependency projection

The [machine-readable inventory](../profiles/historical-board-consumers.json)
projects all172 historical C/C++ units and542 dependency files, rather than
only selected public headers. Each unit records its source hash, object and
dependency edges. `dependency_file_ids` are zero-based indices into `files`;
duplicate GCC prerequisites are counted separately, not mistaken for extra
consumers. There are51 repeated occurrences in the original rules.

Origin categories are438 pinned Git blobs,67 original-request files, one
captured generated header and36 unreviewed toolchain files. These categories
are provenance observations, **not licence classifications or admission**.
The historical ASF and samd-peripherals inputs remain rejected for the new
simulation build. No source bodies, raw compiler invocations or host paths
are copied into this projection.

This report comes from the retained original
[source-origin artifact](https://api.github.com/repos/CrispStrobe/labwired-core/actions/artifacts/11559576651/zip)
of [run37794963356](https://github.com/CrispStrobe/labwired-core/actions/runs/37794963356).
Archive SHA256 is
`85361c5387b4eddf7bb0622b64cc89c09f4f367e465e9b12a8a36723620b7f27`.
The reader requires its exact948805 bytes, unique members/JSON keys,
dependency/origin/request/builder bindings and the exact original map hash.
It neither downloads nor extracts the artifact and invokes no compiler.

To regenerate from an already retained original:

```sh
python3 tools/board-consumers/inventory.py source-origin.zip
python3 -B -m unittest discover -s tools/board-consumers -p 'test_*.py' -v
```

The optional `--output` destination must not already exist. Hosted CI runs
the pure controls and checks the committed report boundary; it does not
reacquire the original, compile consumers or qualify firmware.

## Storage decision: port the selected internal-flash route first

The original allocated link map retains28 settings text sections:
16 from `RAFFS.cpp`, four from `SAMDFlash.cpp` and eight from `settings.cpp`.
The four retained `codal::ZFlash` methods are `totalSize`, `pageSize`,
`erasePage` and `writeBytes`. Their exact section/address/size/object records
are in `allocated_settings_text`. Discarded input sections, object `LOAD`
records, vtables and debug sections are excluded from this code census.

This establishes an allocated internal-flash implementation in that historical
image, not that every retained method executed or a complete runtime call graph.
It does not establish absence of another storage route. Merely porting
`SAMDNVM.h`, or compiling `SAMDNVM.cpp` whose implementation is guarded by
`SAMD21`, cannot qualify this SAMD51 settings path.

Read-only inspection of the retained request's `SAMDFlash.cpp` identifies its
SAMD51 branch's requirements: NVMCTRL parameter/READY/cache/write-mode fields,
command key and page-buffer-clear/quad-word-write/block-erase commands, plus
CMCC cache disable/invalidate/enable. It writes actual flash addresses and
uses8-byte alignment checks with16-byte write windows. This was ordinary source
review, not isolated implementation or a runtime observation. Do not copy
vendor register headers into the replacement to satisfy these spellings.

The current PyBadge engine descriptor has no NVMCTRL entry; USB and QSPI are
stubs. A plain flash memory region does not implement command admission,
busy/error handling, buffered program or erase semantics. The necessary
storage model and actual caller qualification remain open.

## Executable next lane: real no-ASF consumer build

1. Preserve the historical pair: core
   `312ae57e0b31f5b9df07a81e9d846945828e3c5a`, board `codal-itsybitsy-m4`
   `6ecd80ccf1abcc126d06653f178ca03a4d1c6421`, Runtime
   `5bd6b93c219c7e784e885ba2d6812809fb6289a8`. The separately qualified
   services use core `2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`.
   Do not silently substitute these incompatible NVM APIs. Select and record
   a new immutable board/core pair and each explicit migration patch.
2. Use the full unit/dependency inventory to enumerate affected source,
   headers, inlines and generated inputs. Add assembly/startup/linker coverage;
   the historical dependency projection explicitly lacks assembly coverage.
3. Add a separately named simulation-only selector with an allow-listed include
   graph and source manifest. Required unsupported services and unknown inputs
   must fail. Never fall back to legacy CMake's ASF search directories or return
   success from replacement service stubs. Compile/link actual consumers hosted.
4. For storage, qualify the actual selected ZFlash/RAFFS/settings callers,
   not just a toy NVM API. Implement required NVMCTRL/CMCC semantics separately
   in the engine and test real read/write/erase/busy/error/bounds/reset cases.
   State the persistence and cache-model boundary explicitly.
5. Migrate actual serial/JACDAC callers and close required USB/WebUSB/audio/
   GPIO/display paths according to the selected configuration. Removing an
   enabled required service is a profile change, not compatibility evidence.
6. Preserve exact selected sources/notices/toolchain inputs and inspect allocated
   map/undefined symbols/CF2/image admission before original frames, input,
   debugger, installed app and active RTx qualification.

No current consumer has been compiled by this inventory lane. Image admission,
runtime qualification, licence clearance and current-profile compilation remain
explicitly false. Existing service fixtures, source pins, default CMake build,
engine models, physical captures and app packages are unchanged.
