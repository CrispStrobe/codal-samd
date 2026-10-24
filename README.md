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

## Current state — 2026-10-15

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
| Real cancellation and recovery | [PR22](https://github.com/CrispStrobe/codal-samd/pull/22) and engine [PR229](https://github.com/CrispStrobe/labwired-core/pull/229) landed after correcting controller-reset initialization. Fresh success and cancellation/recovery callers passed four actual guest cases and an independent original-log audit; all 23 enabled engine checks passed. [Original checkpoint](docs/REAL-SPI-SERVICES-RESULTS.md#corrected-cancellation-and-recovery-checkpoint); the later eight-case checkpoint below includes gated timeout. |
| Clock-gated timeout cleanup | [PR26](https://github.com/CrispStrobe/codal-samd/pull/26) and engine [PR231](https://github.com/CrispStrobe/labwired-core/pull/231) landed after preserving the original cleanup failure. Fresh success, cancellation and MCLK/GCLK timeout callers passed all eight actual cases under both scheduler profiles and an independent original-log audit; 23 enabled engine checks passed. [Exact source/images, results and limits](docs/REAL-SPI-TIMEOUT-QUALIFICATION.md#landed-corrected-source-checkpoint--2026-10-14). Later IRQ and ownership slices are separate checkpoints below. |
| Writeback AHB DMA-error recovery | [PR28](https://github.com/CrispStrobe/codal-samd/pull/28) and engine [PR233](https://github.com/CrispStrobe/labwired-core/pull/233) landed after all final checks passed. Four actual error/positive cases passed both scheduler profiles and a frozen independent reader; engine23 enabled checks passed. [Exact results, original hashes and limits](docs/REAL-SPI-DMA-ERROR-RESULTS.md). Explicit disabled-controller corruption, not spontaneous valid-buffer or physical failure; no native Arcade/app/RTx claim. |
| Software-pended idle IRQ dispatch | [PR30](https://github.com/CrispStrobe/codal-samd/pull/30) and engine [PR235](https://github.com/CrispStrobe/labwired-core/pull/235) landed after both runtime and all 23 enabled engine checks passed. Four actual stale/positive cases passed both profiles and a frozen source/image-bound original reader. [Exact results and boundaries](docs/REAL-SPI-STALE-RESULTS.md). No extra callback/DATA/wire/CS during the software epoch; natural stale IRQs, stale timeouts and blocking-result ownership remain separate. Guest steps are not RTx. |
| Blocking terminal-result ownership | [PR32](https://github.com/CrispStrobe/codal-samd/pull/32) and engine [PR237](https://github.com/CrispStrobe/labwired-core/pull/237) landed after both runtime and all 23 enabled engine checks passed. All four actual cases and the frozen independent reader pass both profiles: real NotifyOne, competing BUSY before owner error consumption, live fiber-bound returns and positive recovery. [Exact sources, original failures and limits](docs/REAL-SPI-BLOCKING-LEASE-RESULTS.md). Lifetime, whole native Arcade, app adoption and RTx remain separate. |
| SPI destruction policy and idle reconstruction | [PR34](https://github.com/CrispStrobe/codal-samd/pull/34) and engine [PR239](https://github.com/CrispStrobe/labwired-core/pull/239) landed after both Runtime and all 23 enabled engine checks passed. All ten actual cases and the frozen independent audit pass both profiles: active-token, delivering-callback and unconsumed blocking-result deletion deliberately fail-stop; settled idle deletion/free/reconstruction completes fresh transfers. [Exact results, preserved failure and next board/API blockers](docs/REAL-SPI-LIFETIME-RESULTS.md). No safe active deletion, native Arcade, app or RTx claim. |
| Core-local reset | Runtime [PR36](https://github.com/CrispStrobe/codal-samd/pull/36) and engine [PR242](https://github.com/CrispStrobe/labwired-core/pull/242) landed. Four actual reset cases and 38 earlier caller cases passed the frozen original audit; all 27 final engine checks passed. [Exact results and scope](docs/REAL-CORE-RESET-RESULTS.md). No whole-board/peripheral reset, app adoption or RTx claim. |
| Actual NVM consumer compilation | [PR39](https://github.com/CrispStrobe/codal-samd/pull/39) landed after both checks passed. Real KeyValueStorage and SAMDNVM declarations compile without ASF fallback; 23 dependency identities and two compiler refusals pass. [Source-bound result and receipt](docs/NVM-CONSUMER-COMPILE.md). Not a SAMD51 NVM implementation, PXT storage build, linked firmware or image admission. |
| Real PXT flash driver compilation | [PR42](https://github.com/CrispStrobe/codal-samd/pull/42) landed after both checks passed. The explicitly include-adapted real SAMDFlash driver compiles without ASF; 16 dependency identities and two specific compiler refusals pass. [Actual result and unchanged receipt](docs/PXT-FLASH-COMPILE.md). Not full PXT headers, RAFFS/settings, original board configuration, flash execution or RTx. |
| Real RAFFS compilation | [PR44](https://github.com/CrispStrobe/codal-samd/pull/44) landed after both checks passed. Real include-adapted RAFFS and unchanged Flash/RAFFS interfaces compile; 24 dependency identities and two specific refusals pass. [Actual result and unchanged receipt](docs/PXT-RAFFS-COMPILE.md). Not settings/GC implementation, linked storage, filesystem execution, persistence or RTx. |
| Real settings and PXT base types | [PR45](https://github.com/CrispStrobe/codal-samd/pull/45) landed after both final-head checks passed. Real include-adapted settings and complete actual PXT base definitions compile, with selected 32-bit ABI assertions, 25 dependency identities and three specific compiler refusals. [Exact results and unchanged original receipt](docs/PXT-SETTINGS-COMPILE.md). Not GC implementation/linkage, native Arcade, original board configuration, persistence or RTx. |

The allocator and finite GC/runtime caller landed through
[PR47](https://github.com/CrispStrobe/codal-samd/pull/47) and
[PR48](https://github.com/CrispStrobe/codal-samd/pull/48), with independently
audited passes in both scheduler profiles:
60 checks in 5,853,557 steps, with the public `realloc` header over-read corrected
and the selected clearing routine optimized without self-recursive codegen.
[Results and remaining qualification](docs/PXT-GC-RUNTIME.md) preserve the
earlier failures. All six actual source-mutation cases now fail at their exact
intended checks. [PR51](https://github.com/CrispStrobe/codal-samd/pull/51) additionally
landed the separate [real IRQ/in-GC allocation-routing caller](docs/PXT-GC-ALLOCATION-ROUTING.md):
both profiles passed 23 checks in 946,262 guest steps, and all four routing-removal
mutants failed at the observed wrong allocator. The unchanged 60-check caller
and six original mutants were requalified on the same candidate. The later finite
[failure/churn and allocated-DMA lifetime](docs/PXT-GC-FAILURE-LIFETIME-LANES.md)
checkpoints below supersede this dated queue; none of these guest-step counts
is generated native Arcade,
installed-app adoption or a representative RTx measurement.

[PR53](https://github.com/CrispStrobe/codal-samd/pull/53) also landed the separate
[finite allocation-churn/native-refusal caller](docs/PXT-GC-ALLOCATION-CHURN.md):
both profiles passed 662 checks and 308 API/collection boundaries across eight
rounds, in 10,079,873 guest steps. All four native source-mutant cases failed at
their intended observed wrong-result assertions. The earlier 60-check and
23-check callers and their ten negative cases were requalified on the same
source. This covers finite payload preservation and native zero/arithmetic
refusal, not real OOM or allocated-DMA lifetime.

[PR55](https://github.com/CrispStrobe/codal-samd/pull/55) additionally landed
[separate native fatal exhaustion](docs/PXT-NATIVE-REAL-OOM.md): both scheduler
profiles observed the real raw allocation failure and panic code 20, with all
32 old payload bytes and 131072 guard bytes intact, in 396399 guest steps.
All four corruption/fatal-policy mutation cases failed at their intended actual
wrong results; the unchanged baseline, routing and churn campaigns requalified.
The separate [public-GC fatal-exhaustion checkpoint](docs/PXT-PUBLIC-GC-REAL-OOM.md)
also landed in [PR57](https://github.com/CrispStrobe/codal-samd/pull/57): both
profiles retain all 18 live arrays and reach real panic 21 on attempt 19 in
1099271 guest steps, with four actual ownership/panic-policy mutant cases
detected and all earlier campaigns requalified.
[PR59](https://github.com/CrispStrobe/codal-samd/pull/59) adds the separate
[public request-size diagnostics](docs/PXT-PUBLIC-SIZE-BOUNDARIES.md): both
profiles observe cap refusal with panic 22 in 399306 steps, while `SIZE_MAX`
returns a pointer backed by only a four-byte array block in 867133 steps.
All four new mutation/profile cases and five earlier campaigns passed independent
audits. [PR61](https://github.com/CrispStrobe/codal-samd/pull/61) corrects the
GC public allocation before signed conversion and alignment/header arithmetic.
Both profiles qualified seven oversized refusals before `app_alloc`, the valid
131068-byte maximum and a separate fresh normal recovery case; all 16 new
mutation/profile cases and five earlier campaigns passed independent audits.
Finite allocated DMA/TXC/CS/callback lifetime, blocking, terminal and premature-release
cases subsequently qualified; see [their results and retained borrowing limits](docs/PXT-GC-FAILURE-LIFETIME-LANES.md).
The full runtime lane stays open.
These correctness results are not new RTx/performance measurements.

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

## Current selected-runtime checkpoint — 2026-10-24

Fixed-engine [adoption72](https://github.com/CrispStrobe/codal-samd/pull/72),
actual [RAFFS/settings74](https://github.com/CrispStrobe/codal-samd/pull/74),
[compaction](docs/PXT-STORAGE-COMPACTION.md),
[source census](docs/PXT-FULLBOARD-SOURCE-CENSUS.md) and
[modeled retained-backing reset](docs/PXT-STORAGE-RESET.md) have separate
qualified checkpoints. Completed-operation [engine-process backing reopen](docs/PXT-STORAGE-REOPEN.md)
now also qualified through PR81, including the specific lost-backing control.
Interrupted writes, power loss and complete peripheral reset remain open.

[PR80's generated profile](docs/PXT-GENERATED-BOARD-PROFILE.md) reproduces the
actual USB1/WebUSB1 configuration; it is source/configuration admission, not a
compiled full-board image. [PR83's authored CF2 metadata](docs/PXT-CF2-AUTHORED-METADATA.md)
is source/data admission, not actual runtime CF2 reads or a bootloader.
The separate [USB leaf](no-asf-board/usb/README.md) qualifies a finite
actual core/driver boundary through PR82 and engine PR256. It does not run native
Arcade or HF2. Existing storage/service profiles retain their qualified engine
selections; the leaf's explicit selection is not a default package adoption.

The dated compile table above remains historical. [Remaining implementation-ready
lanes](docs/PXT-NO-ASF-NEXT-LANES.md) specify the full permitted source closure,
peripheral replacements, genuine PXT program/template, actual CF2/HF2/startup,
and USB lifetime work. Native Arcade, representative active-workload performance
and installed adoption each need their own qualification.

## Next acceptance steps

The [full historical board consumer inventory and storage continuation](docs/BOARD-CONSUMER-CLOSURE.md)
now cover all172 C/C++ units and542 dependency files, including allocated
ZFlash/RAFFS/settings code. This is migration evidence, not a new no-ASF build.

The separately selected [actual NVM consumer compile-only lane](docs/NVM-CONSUMER-COMPILE.md)
checks a real current-core caller and SAMDNVM declarations without ASF fallback.
It has passed its bounded hosted checks but does not supply a SAMD51 storage
implementation or image.

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
   read evidence. Keep remaining natural/split-vector IRQ work separate from the
   qualified finite DMA cases;
   diagnostic software-pending IRQ0 does not establish peripheral delivery.
2. Preserve the qualified actual service/heap/SPI caller images and originals.
   Preserve the bounded cancellation/recovery, clock-gated timeout, DMA-error and
   software-pended idle IRQ and blocking terminal-result ownership checkpoints.
   Preserve the separately qualified [destruction-policy and idle-reconstruction
   checkpoint](docs/REAL-SPI-LIFETIME-RESULTS.md); active destruction deliberately
   fail-stops rather than cancelling safely. Natural stale IRQs/timeouts remain
   separate. Keep the Timer, MessageBus and buffers alive.
   The separately selected [core-local reset caller](docs/REAL-CORE-RESET-RESULTS.md)
   now passes its four reset cases and all38 earlier caller cases; Runtime PR36
   and engine PR242 landed. Consumer package adoption remains separate.
3. Select compatible immutable core/board dependencies for the upstream NVM and
   serial APIs, and introduce an explicitly named full-board no-ASF build profile.
   The old pointer-address NVM API and current integer-address/read API are not
   interchangeable. Compile every affected real consumer. Preserve the bounded
   reset-reentry evidence and qualify required NVM/serial services; core reset alone is not complete
   peripheral reset. Unsupported paths must fail visibly, not simulate success.
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

Build preparation: [Public PXT storage source bindings](docs/PXT-STORAGE-SOURCE-BINDINGS.md)
binds and stages 64 exact public framework files; the complete generated-board
source/compiler closure remains unqualified; the generated configuration itself
is separately admitted by [PR80](docs/PXT-GENERATED-BOARD-PROFILE.md).

Latest storage results: [compaction](docs/PXT-STORAGE-COMPACTION.md),
[modeled retained-backing reset](docs/PXT-STORAGE-RESET.md), and
[engine-process backing reopen](docs/PXT-STORAGE-REOPEN.md).
Earlier: [Actual blocking SPI terminal-result ownership](docs/REAL-SPI-BLOCKING-LEASE-RESULTS.md).
Earlier: [Actual software-pended idle SPI IRQ qualification](docs/REAL-SPI-STALE-RESULTS.md).
Earlier: [Actual SPI DMA-error and recovery qualification](docs/REAL-SPI-DMA-ERROR-RESULTS.md).
Earlier: [Actual SPI callback/fiber qualification](docs/REAL-SPI-SERVICES-RESULTS.md).
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
