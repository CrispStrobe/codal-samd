# codal-samd
codal port for ATSAMD21 and ATSAMD51 MCUs

## What this fork is doing

This is CrispStrobe's fork of Lancaster University's CODAL SAMD port. We are
developing and qualifying simulation-only building blocks for Brickwright's
native SAMD emulation, initially targeting SAMD51/PyBadge MakeCode Arcade through
LabWired. The aim is a source-reviewed runtime suitable for our distribution,
followed by application/debugger qualification and active native/WASM performance
measurements. The existing translated/PXT route is a separate execution route.

This work is incremental: we do **not** yet claim a complete permissively
distributable runtime, native Arcade boot, shipped application adoption, or a new
real-time performance result. Our simulation prototypes are not qualified for
flashing onto physical hardware.

## Current state — 2026-10-14

| Work | Evidence and boundary |
| --- | --- |
| Upstream synchronization | At this checkpoint, fork `master` contains upstream through `0320025836c8994a3126f0d92cf79e8a91f50e61`. The five previously missing commits update NVM APIs, serial/JACDAC interfaces, and empty-buffer DAC behavior. |
| Current-base integration | [PR10](https://github.com/CrispStrobe/codal-samd/pull/10) merged as `c2274c7ed397787820c92a91438c4c0a902612b8`. Its tree matches reviewed source `3c66f0d27c865192a2b22217e71c1f88f2b3fe91`; the five upstream-modified legacy files and historical qualification pins are preserved. This is not default runtime adoption. |
| Transfer-state controls | [LabWired PR189](https://github.com/CrispStrobe/labwired-core/pull/189) qualifies authored abstract software transfer state, not SPI, DMA, interrupt delivery, or fibers. |
| Interrupt-mask controls | [LabWired PR190](https://github.com/CrispStrobe/labwired-core/pull/190) and [PR191](https://github.com/CrispStrobe/labwired-core/pull/191) exercise saved-mask restoration and software-pended IRQ0 under both scheduler profiles. They do not qualify real CODAL callers or a complete board. |
| Selected-source preprocessing | [LabWired PR196](https://github.com/CrispStrobe/labwired-core/pull/196), with [run 38122098497](https://github.com/CrispStrobe/labwired-core/actions/runs/38122098497), records 21 selected translation units and 278 dependencies. This is preprocessing evidence, not a historical firmware rebuild or execution; token observations are not established calls or retained code. |
| Strict paired-IRQ controls | [LabWired PR198](https://github.com/CrispStrobe/labwired-core/pull/198) landed after all 20 enabled checks passed, including final-head guest run38158310794. One positive and four refusal cases passed under both scheduler profiles; originals and separate audits are preserved. These ten authored cases do not qualify real fibers or scheduler integration. |
| Actual Timer caller fixture | [PR13](https://github.com/CrispStrobe/codal-samd/pull/13) merged as `2ab8dcc380d12468e5464b9edd5c24878648683e`, with reviewed/build-tested/landed tree equality. Final [LabWired PR200](https://github.com/CrispStrobe/labwired-core/pull/200) now landed as `f45d0069c042456333ee472a90bf1d1e9e94c1e6` after all 28 enabled checks passed. Complete unchanged upstream Timer with diagnostic adapters passed two positive profiles and six actual adapter-refusal cases on the final base, with separate original audits. See [exact qualification and remaining limits](docs/TIMER-CALLER-QUALIFICATION.md). No default runtime or app pin was adopted. |
| Default build | The existing runtime CMake selection is unchanged. Adding MIT-authored files does not make the default build ASF-free or qualify its distribution. |
| Real service/heap integration | Actual selected Timer/MessageBus/Fiber/Heap callers and the aligned public heap adapter landed through [PR20](https://github.com/CrispStrobe/codal-samd/pull/20). [Heap qualification and limits](docs/ALIGNED-HEAP-QUALIFICATION.md) distinguish these bounded callers from full libc, board or application admission. |
| Real SPI positive caller | [PR21](https://github.com/CrispStrobe/codal-samd/pull/21) and engine [PR228](https://github.com/CrispStrobe/labwired-core/pull/228) landed; actual asynchronous and blocking transfers passed both engine scheduler profiles at 126775 guest steps, including DMA/wire/TXC/CS/callback/fiber ordering. [Exact evidence, preserved failures and next slices](docs/REAL-SPI-SERVICES-RESULTS.md). No consumer pin or new RTx/app result. |
| Real cancellation and recovery | [PR22](https://github.com/CrispStrobe/codal-samd/pull/22) and engine [PR229](https://github.com/CrispStrobe/labwired-core/pull/229) landed after correcting controller-reset initialization. Fresh success and cancellation/recovery callers passed four actual guest cases and an independent original-log audit; all 23 enabled engine checks passed. Error, gated timeout and blocking-result ownership remain unqualified. [Current result and limits](docs/REAL-SPI-SERVICES-RESULTS.md#corrected-cancellation-and-recovery-checkpoint). |

## Why interrupt integration is explicit

The historical board ABI exposes a no-argument `target_disable_irq()` /
`target_enable_irq()` pair whose counter enables interrupts at zero. Restoring
the caller's previous PRIMASK is a different contract. We must not silently alias
those policies or treat unmatched startup enables as balanced critical sections.

The new prototype requires `BW_SIMULATION_STRICT_PAIRED_IRQ=1`, bounds nesting at
64, and requires a stable nonzero scheduler identity for thread mode. It reads
the actual IPSR exception context; NMI and HardFault are excluded. Detected
underflow, overflow, ownership changes, and lost masking fail closed. Its
switch-permission query is advisory, not an installed scheduler guard. Diagnostic
identity changes in a fixture are not evidence of real fiber switching.

The separately selected service callers now connect actual clock/component/event/
allocator and Fiber/Heap services, with bounded guest qualification. Full board
integration still needs GPIO/EIC, complete DMA ownership/error paths, diagnostics,
melody and NeoPixel caller review. The selected-source census observes an HF2
pair and no matching spellings in the selected RP2040Flash/jddisplay units; that
does not establish absence across all interrupt callers. USB/WebUSB are enabled
in the observed selected configuration and must not be quietly omitted or
replaced by success stubs.

## Next acceptance steps

The next real-service implementation is scoped in
[Real CODAL service integration](docs/RUNTIME-SERVICES-INTEGRATION.md).
LabWired's peripheral prerequisites through authored final-TXC/CS/callback
ordering have now landed in [PR220](https://github.com/CrispStrobe/labwired-core/pull/220).
Bounded actual service/fiber and positive SPI callers now have separate
[qualified observations](docs/REAL-SPI-SERVICES-RESULTS.md). A complete no-ASF4
admitted Arcade image remains unqualified; the legacy default build is unchanged.

1. Preserve the landed [LabWired PR200](https://github.com/CrispStrobe/labwired-core/pull/200)
   checkpoint and unchanged final positive/refusal originals; its final-head
   checks and separate audits are complete. Do not replay unchanged guests to
   read evidence. Qualify actual SERCOM split-vector IRQ behavior before DMA;
   diagnostic software-pending IRQ0 does not establish peripheral delivery.
2. Preserve the qualified actual service/heap/SPI caller images and originals.
   Preserve the bounded typed cancellation/recovery checkpoint; next qualify actual DMA errors, gated timeout,
   stale IRQs and blocking-result ownership, with real callback/Event/fiber evidence.
3. Select compatible immutable core/board dependencies for the upstream NVM and
   serial APIs, and introduce an explicitly named no-ASF build profile.
   Unsupported paths must fail visibly rather than simulate success.
4. Complete component-specific source, notice, and retained-inline review, then
   qualify peripheral IRQ/DMA behavior, boot, display/input, debugger integration,
   package adoption, and active-workload performance. Publish measured results
   separately from control-fixture results.

## Licensing, attribution, and scope

Preserve upstream attribution and assess the exact components selected by each
build. Historical build inputs include vendor-restricted ASF sources:
reproduction or preprocessing does not admit those sources into a distributable
runtime. Newly authored MIT components do not relabel their dependencies or
establish whole-firmware clean-room origin, legal clearance, or store eligibility.
MPL/LGPL components can be considered with their particular obligations retained.
Restricted/reference firmware is not bundled or offered for automatic download.

## Detailed guides

Latest result: [Actual SPI callback/fiber qualification](docs/REAL-SPI-SERVICES-RESULTS.md).
Earlier: [Actual Timer caller qualification](docs/TIMER-CALLER-QUALIFICATION.md).
Earlier: [Authored paired-IRQ guest qualification](docs/PAIRED-IRQ-QUALIFICATION.md).
These dated results supersede prospective steps in the retained fixture guides,
not their source or historical fixture pins.

The immutable links below describe the integration candidate without implying
that its files are already selected by default `master`:

- [Current-master integration and preserved pins](https://github.com/CrispStrobe/codal-samd/blob/d0533c597326bccdff867f8b2f3290e590a30225/docs/CURRENT-MASTER-INTEGRATION.md)
- [Strict paired-IRQ prototype and test boundaries](https://github.com/CrispStrobe/codal-samd/blob/d0533c597326bccdff867f8b2f3290e590a30225/paired-irq/README.md)
- [Foundation source inventory/profile](https://github.com/CrispStrobe/codal-samd/blob/d0533c597326bccdff867f8b2f3290e590a30225/docs/FOUNDATION-SOURCE-PROFILE.md)
- [Critical-section source census](https://github.com/CrispStrobe/codal-samd/blob/d0533c597326bccdff867f8b2f3290e590a30225/docs/CRITICAL-SECTION-SOURCE-CENSUS.md)
- [LabWired permissive-runtime lanes](https://github.com/CrispStrobe/labwired-core/blob/main/docs/engineering/samd-permissive-runtime-lanes.md)
- [LabWired critical-section interface](https://github.com/CrispStrobe/labwired-core/blob/main/docs/engineering/samd-critical-section-interface.md)
- [Selected-source preprocessing qualification](https://github.com/CrispStrobe/labwired-core/blob/fe003506c6ace2b6e10465a18a67bf147b1fa071/docs/engineering/samd-selected-irq-preprocessing.md)

The CODAL guides describe source/runtime contracts; the LabWired guides describe
engine qualification. Neither substitutes for consumer package qualification.
