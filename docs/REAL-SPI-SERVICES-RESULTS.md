# Actual CODAL SPI positive checkpoint — 2026-10-14

The separately selected mode-8 caller passed actual guest qualification under
both LabWired scheduler profiles. This is bounded positive SPI/IRQ/callback/
Event/fiber evidence, not complete TX-service, native Arcade, physical timing,
CF2/image admission, installed consumer adoption, licence clearance or RTx.

## Immutable sources and original evidence

| Record | Exact binding |
| --- | --- |
| Runtime source | [`490b6bd8286609109eb03ad8022862ff581108bd`](https://github.com/CrispStrobe/codal-samd/commit/490b6bd8286609109eb03ad8022862ff581108bd) |
| Runtime source/build tree | `1f338b49ffe5af9a293f7d67e7a6d7a8e7b87078` |
| Runtime merge | [PR21](https://github.com/CrispStrobe/codal-samd/pull/21), `4e083de4f20967f80f4a13223c63b3f4469f8b71`; reviewed/build-tested/landed trees match |
| Selected core | `lancaster-university/codal-core` at `2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`, with explicit retained patches/notices |
| Runtime build | [run38526626225/job115728396890](https://github.com/CrispStrobe/codal-samd/actions/runs/38526626225/job/115728396890), sole enabled exact-head check SUCCESS |
| Engine tested source | [`d7ea4ef393347e3d732372462f02d3250a044d53`](https://github.com/CrispStrobe/labwired-core/commit/d7ea4ef393347e3d732372462f02d3250a044d53), [PR228](https://github.com/CrispStrobe/labwired-core/pull/228), merged as `b4f1f01175bcb0f902a6a1adfa86f82f7d260aaf` after 24 enabled checks passed and seven declared jobs skipped; tested/landed trees match |
| Engine tested base/tree | base `6e03dd12e22ceb534ec1bc494c360b72209ca94f`, tree `55f3a02107fb6bc7ef379ccbdba5feb48764a631` |
| Engine tested merge | `47c1e97e6553fba9e29b0a210a580e7553ab63dc`, parents tested base and engine source, same tree |
| Actual guest | [run38528013679/job115732881796](https://github.com/CrispStrobe/labwired-core/actions/runs/38528013679/job/115732881796), SUCCESS |
| Mode-8 ELF SHA256 | `89a9c8b071a8c2a79eab43a03f98c9a4b504ee6657db5f67f18e9702860b160f` |
| Runtime original build log | 118340 bytes, SHA256 `beaefb5b73084431fce976804bc7be9c9003374504cf7d9e5bead36900b8e076` |
| Actual original guest log | 164502 bytes, SHA256 `8bf25e0b73ce1b2167433e44165b64e0e94ddcfe465f20a540fb46f57cb63d5b` |

The image has 49 selected inputs and fourteen C++ dependency rules. Unknown
headers still refuse; selected source and compiler notices remain preserved.
The eight prior service/clock/heap images remain byte-identical in this build.
Complete allocated-member/header/inline and applicable-exception review remains
owed before a distribution claim. The legacy default build is not changed.

## What actually passed

Each engine profile completed at **126775 guest steps**, below the unchanged
four-million-step bound. Both produced identical qualified observations:

- Real startup, complete vector binding, data copy and BSS clearing.
- Two four-byte RAM transfers with eight actual DATA acceptances and eight
  wire bytes; two separate DMA completion/writeback-zero IRQs and two SPI TXC
  IRQs, with matching exception entries and returns.
- Exactly 28 chronological rows: CS select before acceptance; final DATA before
  DMA IRQ; DMA IRQ before final wire byte; final wire before TXC/IRQ; CS release
  before callback or blocking return.
- One real success callback in IRQ79, correct argument and released CS; busy
  refusal on callback reentry; observed actual callback entry and LR return.
- Actual MessageBus Event delivery and resumed main/distinct blocking fibers,
  with six observed assembly context swaps. No host callback/Event injection.

The separately frozen independent reader retained the original predicates and
passed one synthetic positive and fifty refusal controls before acquisition.
Original reader SHA256:
`20a25710b9acb405c6fdd449bf23a234bde350e8bda63c99c97043f333ed8f80`.
RAM source/image-binding reader SHA256:
`f8db89742e6856df9ceb977a6e53fb90a79347739d0d65205f11620db1b11327`.
Both actual original profiles passed that reader. Original logs were acquired
once; reader inspection did not replay the guest or import producer helpers.

## Preserved failures and fixes

The first flash-buffer guest [run38517091806](https://github.com/CrispStrobe/labwired-core/actions/runs/38517091806)
hit the unchanged step bound. Its diagnostic follow-up
[run38519806789](https://github.com/CrispStrobe/labwired-core/actions/runs/38519806789)
reported the DMA model's primary-RAM-only refusal while reading terminal state.
The caller now uses startup-copied RAM bytes; the driver explicitly refuses
flash buffers before state reservation or hardware changes. Flash DMA is a
separate unimplemented extension, not silently supported.

The RAM guest [run38526985190](https://github.com/CrispStrobe/labwired-core/actions/runs/38526985190)
completed the callbacks and fibers but failed the unchanged ordering predicate:
the slave's CS notification followed first DATA acceptance by one step. The
engine fix refreshes bus-sourced CS before an admitted DMA beat, sharing the
tick resolver without advancing time or draining wire bytes. Hosted controls
check selection before the DATA observer, no premature wire byte, no duplicate
selection, and no gated/blocked effects. Ordinary CPU stores are unchanged.
None of these failed originals is relabelled as a pass.

## Corrected cancellation and recovery checkpoint

The first actual cancellation [run38535325325/job115756701784](https://github.com/CrispStrobe/labwired-core/actions/runs/38535325325/job/115756701784)
delivered its typed failure and resumed main, but recovery failed: software
reset required a disabled DMAC controller. That original failure remains a
failure. The driver now disables the exclusively owned controller before reset;
the model guard, step bound and ordering predicates were not weakened.

| Record | Exact binding |
| --- | --- |
| Corrected runtime | `846601f69005d2cbd1c82f33e350301882ac4f80`, [PR22](https://github.com/CrispStrobe/codal-samd/pull/22), merged as `bdd4bede956634ca271c2d9670da98a9622a1d27` |
| Runtime reviewed/build-tested/landed tree | `44fa38159f5bb0c06ac04ea4395dbf0ae4f08498`; source build and documentation-policy checks SUCCESS |
| Separate runtime build | [run38537687070/job115764476185](https://github.com/CrispStrobe/codal-samd/actions/runs/38537687070/job/115764476185) |
| Engine source/adoption | `00525a5ad3a0ece03018004489c3bf5f063bb28e`, [PR229](https://github.com/CrispStrobe/labwired-core/pull/229), merged as `49d7a2af8a7bf462c70424e1376beb100780287b` after all 23 enabled checks passed and seven declared jobs skipped; reviewed/tested/landed tree and exact parents verified |
| Engine actual tested base/tree/merge | base `b4f1f01175bcb0f902a6a1adfa86f82f7d260aaf`, tree `64a00a307db14a042725f651f210886251f6bf92`, merge `34cefece18ae486f72ec0dd85fdcec690a588a7a` |
| Four actual guest cases | [run38537952558/job115765353858](https://github.com/CrispStrobe/labwired-core/actions/runs/38537952558/job/115765353858), SUCCESS; original independently audited |
| Corrected positive mode-8 ELF SHA256 | `9db4a75a31effe631325ef152d251d3a92488f6575bfdcf9b767123a9a80a290` |
| Cancellation mode-9 ELF SHA256 | `377c4be4b57a5b36d32784e074724b391a3cbb058599e27a886303f2b738955b` |
| Actual original guest log | 216579 bytes, SHA256 `950533e7740a960ba793cf40fffef804e1a432974c4d12fef8e2d089a4610027` |
| Four-case independent reader SHA256 | `f3fb068dcf75c9ca1cc7a2bbfb74f6b4fcf70888c381800b22a3955ea80ba5d5` |

Both modes use separate clean dependency checkouts of the same immutable core
and 49 selected inputs. The eight older probe images remain unchanged. Before
guest-original acquisition, the independent reader retained the previous
positive/cancellation predicates with explicit new image bindings and passed
two synthetic positives, 110 refusal controls and four combined-envelope
refusals. Inspection acquired the official original once; no guest replay or
producer-helper import was used to audit it.

The affected positive caller passed both profiles at 126790 steps, with the
same 28-row causal ordering and six actual context swaps. Cancellation/recovery
passed both profiles at 145294 steps, with eight actual swaps and matched
DMA47/SPI79 entries and returns. Each cancelled prefix had two DATA acceptances,
zero modeled wire bytes, writeback remainder 30 and nine preserved causal rows.
One typed cancellation callback ran in the cancelling fiber with interrupts
masked and CS released; competing start and callback reentry refused busy.
Actual callback return, cancellation return, Event wake and main resume preceded
the two successful recovery transfers and their unchanged 28-row ordering.

This is finite cancellation/recovery evidence, not a general promise that
cancellation transmits no bytes. DMA error, gated timeout, stale IRQs,
blocking-result ownership, complete board/native Arcade, installed consumers,
performance, physical behavior and distribution/licence closure remain open.

## Next independent acceptance slices

1. Preserve the landed PR228/PR229 source and exact-head qualification records.
   Neither merge implicitly moves a consumer pin.
2. Preserve the four qualified corrected-source originals and the failed
   cancellation original. Do not replay unchanged guests to read evidence.
3. Preserve the later [eight-case clock-gated timeout checkpoint](REAL-SPI-TIMEOUT-QUALIFICATION.md#landed-corrected-source-checkpoint--2026-10-14).
   Preserve the later [four-case DMA-error checkpoint](REAL-SPI-DMA-ERROR-RESULTS.md).
   Next add actual stale IRQ and blocking-result ownership cases. Source state
   controls or callback counters alone are insufficient.
4. Close required board APIs/reset, CF2/build/notices/image admission and actual
   frames/input/debugger; then representative repeated RTx, installed consumer
   adoption and measured-bottleneck optimization. No physical flashing.

See [the selected service contract](REAL-SPI-SERVICES-QUALIFICATION.md) and
[the aligned-heap boundary](ALIGNED-HEAP-QUALIFICATION.md).
