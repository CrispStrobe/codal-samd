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

## Current state — 2026-10-11

| Work | Evidence and boundary |
| --- | --- |
| Upstream synchronization | At this checkpoint, fork `master` contains upstream through `0320025836c8994a3126f0d92cf79e8a91f50e61`. The five previously missing commits update NVM APIs, serial/JACDAC interfaces, and empty-buffer DAC behavior. |
| Current-base integration | [PR10](https://github.com/CrispStrobe/codal-samd/pull/10) normally merges the simulation stack with current `master`. The five upstream-modified legacy files match that base exactly; authored prototypes and historical qualification pins are preserved. This candidate is not default runtime adoption. |
| Transfer-state controls | [LabWired PR189](https://github.com/CrispStrobe/labwired-core/pull/189) qualifies authored abstract software transfer state, not SPI, DMA, interrupt delivery, or fibers. |
| Interrupt-mask controls | [LabWired PR190](https://github.com/CrispStrobe/labwired-core/pull/190) and [PR191](https://github.com/CrispStrobe/labwired-core/pull/191) exercise saved-mask restoration and software-pended IRQ0 under both scheduler profiles. They do not qualify real CODAL callers or a complete board. |
| Selected-source preprocessing | [LabWired PR196](https://github.com/CrispStrobe/labwired-core/pull/196), with [run 38122098497](https://github.com/CrispStrobe/labwired-core/actions/runs/38122098497), records 21 selected translation units and 278 dependencies. This is preprocessing evidence, not a historical firmware rebuild or execution; token observations are not established calls or retained code. |
| Strict paired-IRQ prototype | PR10 adds an explicitly opted-in, bounded prior-mask/depth policy, fail-closed diagnostics, host mutation controls, and ARM build fixtures. Its exact-head checks are pending at this checkpoint; actual guest refusal, real callers, and scheduler integration remain unqualified. |
| Default build | The existing runtime CMake selection is unchanged. Adding MIT-authored files does not make the default build ASF-free or qualify its distribution. |

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

Real integration still needs Timer, Fiber/Heap, diagnostics, GPIO/EIC, DMA,
melody, and NeoPixel caller review. The selected-source census observes an HF2
pair and no matching spellings in the selected RP2040Flash/jddisplay units; that
does not establish absence across all interrupt callers. USB/WebUSB are enabled
in the observed selected configuration and must not be quietly omitted or
replaced by success stubs.

## Next acceptance steps

1. Finish every enabled exact-head PR10 host control and ARM build check,
   preserving failures. Successful compilation is not guest qualification.
2. Execute source-bound positive and refusal fixtures under both LabWired
   scheduler profiles: vector/startup/BSS checks, delayed interrupt delivery,
   entry-mask restoration, exact fault diagnostics, and stable masked refusal
   loops. Do not count a timeout as successful refusal.
3. Integrate actual Timer/Fiber/Heap callers, early returns, startup/panic paths,
   and scheduler-boundary guards with affected clean guest tests.
4. Select compatible immutable core/board dependencies for the upstream NVM and
   serial APIs, and introduce an explicitly named no-ASF build profile.
   Unsupported paths must fail visibly rather than simulate success.
5. Complete component-specific source, notice, and retained-inline review, then
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
