# Pinned storage source gaps and first execution slice

The finite [allocation/DMA lane](PXT-GC-FAILURE-LIFETIME-LANES.md) is landed.
The next runtime lane starts with the reproducible
[source-gap census](../profiles/pxt-storage-link-census.json) and its
[reader](../tools/storage-link-census/census.py). This census validates all 64
retained public framework files against their immutable Git identities and
records source-line witnesses. It reads the PyBadge descriptor from the exact
engine Git object, rather than assuming the checkout's current branch is the
qualified engine. It invokes no compiler, linker, guest or hardware operation.

## Exact source boundary

| Input | Binding or observed requirement |
| --- | --- |
| Public framework | `31abf23d118f35010fb75122e60a1eb2b6dffe9f`, complete 64-file admitted source projection and original notice. |
| Held engine | `73f8475f8e406b1ff4475bc9886e22479881d67a`, `configs/chips/atsamd51-pybadge.yaml`. |
| ZFlash | Real `SAMDFlash.cpp`, with READY polling, geometry, erase, page-buffer clear, quad-word program and actual flash-address stores. |
| Settings | Real `settings.cpp`, configurable size and mount/read/write calls into RAFFS. The SAMD51 expression uses `512 * 1024 - size`. |
| Platform | The real shared platform still includes GPIO, SPI/I2C, NVM, DAC/PDM and serial headers. |
| Earlier compilation | The unchanged flash, RAFFS and settings reports retain their separate source bindings and compile-only flags. |

The held descriptor declares neither required address `0x41004000` (NVMCTRL)
nor `0x41006000` (CMCC). Its USB and QSPI entries have type `stub`. These are
facts about that descriptor, not a complete capability audit of every engine
subsystem. A flash memory range alone does not establish command or cache
semantics. No engine pin or model is changed by this inventory.

The settings default is 32 KiB on SAMD51, giving the source expression's default
range `0x78000..0x80000`. Runtime configuration can override the size; this is
not an observed mounted filesystem or an admitted arbitrary storage range.
The historical full board enables USB/WebUSB; the qualified service fixture
disables them. Shared-header closure and that configuration difference remain
explicit work, even after a smaller selected storage guest succeeds.

## First implementation slice: the actual ZFlash route

Implement and qualify the low-level route before adding RAFFS/settings execution.
Keep the existing core/framework/generator pins and all GC/DMA regressions.
Any new engine source and eventual pin update require their own owned lane and
qualification; this source census does not authorize an unreviewed substitution.

1. Select one new, explicit guest linking the real four-method ZFlash driver,
   actual IRQ/libc services and the declared no-ASF register views. Preserve the
   original public driver and identify its include-only adaptation. Do not
   modify its algorithm to accommodate an incomplete model.
2. Define the model's exact geometry, READY transitions, command-key admission,
   erase block, page buffer, sixteen-byte program window and cache-control
   behavior from reviewed primary register documentation. The current census
   records caller requirements, not a hardware specification or model design.
   Distinguish controller state from flash backing; program/erase must affect
   the same memory observed by real guest loads and instruction/data access.
3. Reserve a declared test block inside the selected storage region. The linker
   must prove no overlap with vectors, application sections or another selected
   persistent owner. Observe real driver return, all programmed/readback bytes,
   re-erase and neighboring unchanged bytes. Reset and persistence claims require
   their own defined observation boundary, not just another read in one run.
4. Preserve actual driver behavior: eight-byte alignment/length refusals and
   partial sixteen-byte windows; already-written non-erased data can return -3.
   The source disables its ECC check. Do not invent caller error propagation,
   erase bounds checks or automatic recovery absent from the real driver.
   Separate model command/busy/error/bounds tests from what ZFlash observes.
5. Add nonvacuous isolated source/model mutations for each admitted positive or
   refusal result. Freeze standalone original readers before hosted acquisition.
   Require both scheduler profiles, actual source/image/commands/dependencies/
   notices/stream bindings, all enabled exact-head checks and exact merge
   composition. A setup failure, unmapped access or exhausted bound cannot pass
   as successful storage behavior.

Only then add a separate real RAFFS/settings guest using the qualified real
GC/root/allocation services, source-bound hashing, configuration and scheduler
inputs. Qualify format/mount/write/read/remove/list, live GC payloads and the
stated persistence boundary. Neither the earlier object compilation nor the
new low-level flash case substitutes for that filesystem/settings execution.

## Reproduce the source census

Use the already admitted staged public source projection and a local engine Git
object store containing the held commit; the command performs no acquisition:

```sh
python3 -B -m unittest discover -s tools/storage-link-census -p 'test_*.py' -v
python3 -B tools/storage-link-census/census.py public-stage engine-checkout --check profiles/pxt-storage-link-census.json
```

`--output` instead writes a new destination exclusively and never overwrites an
existing receipt. Hosted CI independently stages the pinned public framework
and reproduces the committed report. Its source controls reject changed source
bytes, missing notices, symlinks, missing required witnesses and unsupported or
duplicate descriptor entries. All compilation/execution/image/storage/board/
licence-clearance flags remain false. This is source scoping, not a full board,
native Arcade, performance, installed Lite, hardware or release result.
