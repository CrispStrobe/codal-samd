# Actual CODAL DMA-error result — 2026-10-14

Runtime [PR28](https://github.com/CrispStrobe/codal-samd/pull/28) merged as
`374348d256299f3cd46fc33a3e7287863f12aa6c` after both enabled checks passed.
Engine [PR233](https://github.com/CrispStrobe/labwired-core/pull/233) merged as
`1ec61766775897fa9eeb9f8edc4a0a796def4527` after all 23 enabled checks passed
and only seven declared optional jobs skipped. Reviewed/tested/landed trees,
exact source/base parents and default-branch refs were independently verified.
These finite results do not qualify a complete native Arcade image or app.

## Exact source and observations

Runtime caller [e81fa3b95a3f3e0a32975273f7da0d176ffd61b8](https://github.com/CrispStrobe/codal-samd/commit/e81fa3b95a3f3e0a32975273f7da0d176ffd61b8)
build [38600413289/job115977902980](https://github.com/CrispStrobe/codal-samd/actions/runs/38600413289/job/115977902980)
passed. Reviewed/build-tested tree `e654b0041caa5d94af89bd9c04a755c4cee4680f`
matched; tested merge `2bd50436b76f926d42f364ec9401a9e31292a0a1` had exact
base `c8960063d729ba38e616a67a2bf7507e7fc33157` and source parents.
All thirteen selected modes passed their complete mode-specific dependency
gates: twelve rules for modes 0–7 and fourteen for SPI modes 8–12. Images 0–11
stayed unchanged. The selected upstream core remains
`2ed19d64ec3359143409a4b0a9758b4aeef3ee7d` with retained patches/notices.

Final engine [e5726080426b58b916f6c2f24294db767fefc3f7](https://github.com/CrispStrobe/labwired-core/commit/e5726080426b58b916f6c2f24294db767fefc3f7)
actual [38609722076/job116009634346](https://github.com/CrispStrobe/labwired-core/actions/runs/38609722076/job/116009634346)
passed all four separately selected error/positive cases. Reviewed/tested tree
`bf0732fa4d9fb96558824b3e8f66c855884bb32e` matched; tested merge
`75a4ea34dc18ca8f9791b913c3b28328397495d4` had exact base
`194dd59b17efa6506e1a0e936621d5f7eeb2ac02` and source parents.

| Case, each of two scheduler profiles | Actual observation |
| --- | --- |
| Mode12 writeback AHB error and recovery | 148174 steps; 18 error-prefix and 28 recovery rows; nine actual fiber swaps |
| Partial failed transfer | DATA bytes `91 82 73`, wire byte `91`; frozen pre-fault remaining count 30 |
| Error delivery | One actual TERR at exception47, typed `dma_error`, PRIMASK0, released CS, disabled SPI MASTER `0x2000c`, INTEN0, DMA TX `0x200d00` |
| Error full IRQ census | DMA3/SPI2 entries and matching returns; recovery-only DMA2/SPI2 is explicitly derived after full verification |
| Mode8 positive control | 126844 steps, unchanged 28-row asynchronous/blocking ordering, six actual swaps, DMA2/SPI2 entries and returns |

The real guest worker globally disables the exclusively owned controller before
snapshotting writeback count, retargets its enable-protected writeback address to
aligned unmapped `0x60000000`, then reenables global DMA without restarting the
live channel. One further DATA acceptance precedes the failed writeback and
actual TERR. Every accepted/wire byte and raw disable/address/re-enable write is
retained. The original RAM snapshot is not a post-fault writeback result.

The observed callback entry and return, busy reentry refusal, real Event/main
resume, subsequent asynchronous and distinct blocking recovery preserve the
causal ordering. No host IRQ/Event/flag/callback injection or producer-helper
import was used in the independent audit. The driver/model were unchanged.
This is an explicit controller-corruption fixture, not evidence of spontaneous
valid-buffer failure or physical fault behavior. The four-million-step bound
was unchanged.

## Original bytes and independent admission

| Record | Binding |
| --- | --- |
| Mode12 ELF SHA256 | `8a388ace0da2faa10e782b67422f92822b595f367f58e05a13788c373ae39793` |
| Mode8 ELF SHA256 | `a7f4d21f32f4c0f4e38fbb79b02bf4a31f5b3552f0c3cae353297128fac5e68e` |
| Separate original runtime build log | 160944 bytes; SHA256 `b56c400845bf6bc4730ed83f3ae9901891281c056be03f2a063096878cdd2e71` |
| Final original guest log | 170574 bytes; SHA256 `b9e2e06b5efc60b929a407b26dac28f2dc72ac10edf4c14331407a24f171f236` |
| Frozen final source-binding reader | SHA256 `17e4a54f9bebf9886415e25badcde15ed90637e6b0c125e9eb46af0a4109d82e` |
| Unchanged error-predicate reader | SHA256 `22852f9c102e4897531caf8da24f758b3a12fab9f815c4e584e895a91dd9f706` |

Official provider metadata, tested merge, exact parents/tree and source were
admitted before acquiring the sole final original. The final binding-only
reader hash-checks its frozen predicate parent and distinct source-build images;
before acquisition it passed one fault positive/96 refusal controls, three
complete-envelope positives/165 refusals and one provider positive/21 refusals.
All four actual original cases passed. Reading evidence did not replay guests,
rewrite originals or download a second original.

## Failures preserved, not relabelled

The first runtime [build38598675332](https://github.com/CrispStrobe/codal-samd/actions/runs/38598675332)
failed the strict dependency census because newly selected mode12 was absent
from the fourteen-rule selection. Source e81 adds that mode to the complete
requirement; it does not weaken the gate. Original:159947 bytes, SHA256
`abff6bd62697a3fe5898b78c267e021088b9d2598d8d6e730d15ff7dd956e5a6`.

The prior engine implementation passed its actual
[38604962935](https://github.com/CrispStrobe/labwired-core/actions/runs/38604962935)
but its broad [default-member job116001385107](https://github.com/CrispStrobe/labwired-core/actions/runs/38604963070/job/116001385107)
failed generated ignored-test inventory freshness: materialization had dropped
the renderer-required trailing blank line. Final e572 restores that single
generated Markdown line only, with counts156 ignores/92 files/0 undocumented;
executable source, tests and workflows are unchanged. Failed original:50013
bytes, SHA256 `5fb73fc58017e42059c974893defd7c78fb22f5c3f65ea17eb19ab61b64de649`.
The earlier passing actual original also remains retained, separately from the
final-head original and qualification. No failed original becomes a pass.

## Remaining slices

Stale software-pended IRQ dispatch and blocking terminal-result ownership need
separate actual callers and complete raw IRQ/Event/fiber observations. Object
lifetime remains a separately named acceptance boundary. Complete board/API,
no-ASF build profile, selected component notices, CF2/admitted image, actual
frames/input/debugger, repeated native/WASM RTx and installed consumer adoption
are not established here. No consumer pin, threshold, acknowledgement or physical
capture changed; no full native Arcade, release/licence or store claim follows.
