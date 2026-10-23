# Remaining SAMD runtime and storage work

Refresh `master`, open PRs and current ownership before starting. Keep the
qualified cases and their original failures unchanged. The finite selected
runtime uses engine `79071f69`; a new dependency selection requires explicit pin
review and affected clean guest qualification. The source census is an inventory,
not full-board execution. Historical compile receipts remain historical.

## 1. Freeze an actual generated full-board configuration

Start with [the current source census](PXT-FULLBOARD-SOURCE-CENSUS.md),
`profiles/pxt-framework-sources.json`, the retained `pxt.h`/platform projection,
`runtime-services/platform_includes.h`, and owning Codal headers/CMake. Freeze
public board/core/framework/generator identities and the actual generated
configuration plus preprocessor/compiler dependency closure in a new profile.
Core defaults USB0/WebUSB1 and service0/0 do not prove a full-board USB setting.

Done when every required header, type, implementation and selected feature has
an admitted source binding or a named replacement gap. Keep ASF4 and
hardware-use-restricted code excluded from the simulation selection. Preserve
notices and component-specific obligations. Establish a QSPI source use before
calling it required; its descriptor stub alone is insufficient. Source-only
controls must refuse missing/mismatched inputs and reproduce the new profile.
Do not overwrite the historical storage census or infer licence clearance.

## 2. Implement the explicit no-ASF board consumer profile

Use the admitted configuration from item 1. Resolve the real NVM/serial API
compatibility and device/HAL header gaps; select permitted implementations with
real callers, rather than invented object layouts or successful return stubs.
Compile every affected real consumer with exact source/dependency/notice
bindings and named compiler refusals. Keep required features enabled. A compile
or declaration result cannot qualify runtime behavior or a complete image.

## 3. Qualify required models, IRQ/services and application startup

Derive this scope from item 1's actual feature selection. For USB1, qualify clock
and reset admission, pin mux, calibration, descriptor/buffer, endpoint and
interrupt semantics against the real enabled source. Qualify any separately
required serial/NVM service, natural SERCOM IRQ path and the application vectors
at `0x4000`. The service fixture's vectors at `0` and feature-disabled profile do not
close these gaps. Native NVMCTRL/CMCC sources already exist; their absence from
the PyBadge descriptor is a selection gap, not evidence that the models do not
exist. Preserve the finite DMA/TXC/CS/callback and allocation-domain contracts.

Done when each affected source/model change has clean actual positive/refusal
and specific mutation guests, frozen independent original readers, one retained
official evidence intake and every enabled exact-head check. No physical,
complete-peripheral-reset or full-board claim before that exact evidence.

## 4. Engine restart/backing reopen and interrupted operations

The separate modeled AIRCR result covers CPU/core-system reset within one Bus
after completed writes; peripheral models remain live. A new reopen case must
freeze backing identity/serialization and the real new-machine startup contract,
retain completed-operation bytes without repair, and verify exact data,
deletion/listing, code/neighbor canaries and a specific lost-backing control in
both profiles. A host reset shortcut or copied expected output is insufficient.

Interrupted erase/program/commit and power-loss recovery require another precise
fault/commit-order contract. Do not infer them from object remount, successful
bank compaction, spacing panic `921` or the completed-write reset case.

## Later consumer and performance work

Keep active-deletion/borrow-lifetime limits, natural stale IRQ/timeouts and
complete peripheral reset separately scoped. Generated native Arcade needs a
real generated program and admitted board/image, frames/input and debugger
stop/resume. Representative native/WASM performance requires repeated paired
measurements of that real workload. Installed Lite adoption follows qualified
source/package/pin selection and actual installed acceptance. Existing source
or synthetic caller results do not qualify those boundaries.
