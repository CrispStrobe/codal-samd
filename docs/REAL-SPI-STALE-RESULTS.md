# Actual software-pended idle SPI IRQ checkpoint — 2026-10-14

Runtime [PR30](https://github.com/CrispStrobe/codal-samd/pull/30) merged as
`b991f75ea2be58e116b9f5a582a3d5fe168b10cc` after both enabled checks passed.
Engine [PR235](https://github.com/CrispStrobe/labwired-core/pull/235) merged as
`d64ab20c6d92bc2b821e634aa0c4ffa355ff964c` after all 23 enabled checks passed
and only seven declared optional jobs skipped. Reviewed/tested/landed trees,
exact source/base parents and default-branch refs were verified.
This is finite software-pended idle-dispatch evidence, not complete native Arcade.

## Immutable sources and original evidence

| Record | Exact binding |
| --- | --- |
| Runtime caller | `35386bee113d4dc1531aa3c8ab90e6f9078191fe`, [PR30](https://github.com/CrispStrobe/codal-samd/pull/30) |
| Runtime reviewed/build-tested tree | `5d6059ee8ad9c067fe5883776779b01580fcf054` |
| Separate runtime source build | [38627606289/job116070644515](https://github.com/CrispStrobe/codal-samd/actions/runs/38627606289/job/116070644515), SUCCESS |
| Runtime tested merge and base | `24026b3cb7a957bb243128cdd2f858d9d92bb508`; base `f2563489a19204f8f0b7bd94875d51954732d844`, exact source/base parents |
| Engine tested source | `28b93fea8307ded245baedffa6f318134997df26`, [PR235](https://github.com/CrispStrobe/labwired-core/pull/235) |
| Engine reviewed/tested tree | `295c8864a478ccdac500a4149134ccdb5ffb18fe` |
| Engine tested merge and base | `7a893d01de7ee68d0b5a157c6e6d2dc33bbaef1f`; base `e6b3a79c187d2d424f7253ca2d673fe638b90839`, exact source/base parents |
| Four actual cases | [38628064916/job116072238678](https://github.com/CrispStrobe/labwired-core/actions/runs/38628064916/job/116072238678), SUCCESS and frozen-reader PASS |
| Mode 13 ELF SHA256 | `b60579d48c3b4ffe8b8358fe53a070bd54251cbf00c9614f0fd24401c61d3107` |
| Positive mode 8 ELF SHA256 | `a7f4d21f32f4c0f4e38fbb79b02bf4a31f5b3552f0c3cae353297128fac5e68e` |
| Sole separate source-build original | 171561 bytes, SHA256 `a65a2a065e4f9cc84942876d94d1976d9eddacf837e0385c45f2a2a9b4311294` |
| Sole actual guest original | 227794 bytes, SHA256 `435fc53544e2c13f3534093148f8b7365c37c1d8ebc6f3fdce0d8a207d9cb9de` |
| Frozen source/image-bound reader | SHA256 `f4ee6e6b94611243f13673b2d2ea92e9212905dee22a74fc4b76a5952ab19b4f` |
| Hash-checked immutable predicate reader | SHA256 `5e42460b160593bba20421f7cd968b9dbbca777a3a8972226043db76a5c0e82b` |

All fourteen source-build modes passed their complete mode-specific gates;
old modes 0–12 ELF images remained byte-identical. Modes 0–7 have twelve compiler
dependency rules and 43 selected inputs; SPI modes 8–13 have fourteen rules and 49
selected inputs. Core dependency remains `2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`
with retained patches and notices. No driver/model or legacy default build changed.

## What actually passed

Both scheduler profiles produced identical accepted observations:

- A real priming transfer finished through DMA, final wire byte, SPI TXC,
  released CS, callback entry/return, actual Event 57/6 and main-fiber resume.
- The real guest masked interrupts and wrote bit 31 to NVIC ISPR0 and ISPR1,
  explicitly software-pending IRQ31 and IRQ63. All eight raw store bytes remain
  in the original, separately from peripheral hardware-completion events.
- Actual exceptions 47 and 79 entered and returned inside the observed dispatch
  helper's PC/LR epoch. DMA flags were 0; SPI flags retained sticky value 3 with
  INTEN0, control `0x2000e` and CS released. No additional callback, failure,
  DATA/wire byte or CS transition occurred during this software dispatch epoch.
- The complete closed prefix had 30 rows. Only after the helper returned did
  the unchanged 28-row asynchronous/blocking recovery start. Full actual DMA/SPI
  counts were 4/4 with matching entries and returns: prime 1, software 1, recovery 2
  for each. Software-only 1/1 is retained separately; recovery-only 2/2 is an
  explicitly derived projection after full raw-prefix and census verification.
- Each stale case finished at 156963 guest steps with eight actual context
  swaps. Each separate positive control finished at 126844 steps with 28 rows,
  six swaps and actual DMA/SPI 2/2. Guest-step counts are not RTx measurements.

Complete startup/vector/data/BSS/heap and source/compiler/linker/resource gates
and the unchanged four-million-step bound were retained. The workflow built
modes 13/8 from separate clean pinned dependencies and retained all four cases.
No host IRQ/Event/callback/flag injection or guest replay was used in the audit.

The independent reader was frozen against the separate source-build images
before acquiring the actual original. After distinct image binding, one case
positive/151 refusal controls, three complete-envelope positives/204 refusals
and one provider positive/24 refusals passed. Official provider/source/tree/
parent metadata was admitted before the sole original acquisition. A later
source-aware read-only crosscheck agreed; this is not an independent
implementation or clean-room-origin certification.

## Boundary and next slices

This fixture establishes **software-pended idle dispatch**, not naturally
arising stale hardware IRQs, stale timeout delivery or physical behavior.
Blocking terminal-result ownership is now [qualified separately](REAL-SPI-BLOCKING-LEASE-RESULTS.md).
Object lifetime still needs separate actual refusal and idle-reconstruction callers.
Complete board APIs, admitted no-ASF image/CF2/build/notices, native Arcade,
frames/input/debugger, repeated native/WASM RTx and installed consumer adoption
remain separate. No consumer pin, threshold, acknowledgement or physical capture
changed; this is not distribution/licence clearance or store eligibility.
