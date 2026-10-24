# Remaining SAMD runtime and storage work

Refresh `master`, open PRs and ownership before starting. Preserve qualified
cases and original failures. Existing finite storage/runtime cases retain engine
`79071f69`; the USB leaf has a separate explicit engine selection. New source or
pin selections require affected clean actual guest qualification.

## Generated configuration admitted; compiler closure remains open

[PR80](PXT-GENERATED-BOARD-PROFILE.md) reproduces the exact generated forced
header with USB1/WebUSB1, 120 MHz, 192 KiB RAM and a 10 KiB stack. The 64-file
framework projection and historical census remain source inventories. They do
not close the selected full compiler/header/library graph. Follow
[lane A](PXT-NO-ASF-NEXT-LANES.md#a-complete-the-selected-source-and-header-closure)
for complete dependencies, permitted source/notice bindings, generated inputs
and explicit core API migration. Preserve the original profile and receipts.

## No-ASF board implementation and actual startup

The [USB-only leaf](../no-asf-board/usb/README.md) and authored
[CF2 source/data component](PXT-CF2-AUTHORED-METADATA.md) have separate finite
boundaries; neither is a native Arcade program or full-board compiler closure.
[Implementation lanes B–F](PXT-NO-ASF-NEXT-LANES.md) scope required pin/EIC/PWM,
serial/SPI/I2C/DMA, timer/audio implementations, a genuine generated PXT image,
actual CF2 reads, natural IRQ/services/startup and HF2. Keep required features
enabled; guessed object layouts and successful return stubs cannot close gaps.
USB repeated enumeration and endpoint/deallocation findings have their own
[lane G](PXT-NO-ASF-NEXT-LANES.md#g-usb-endpoint-initialization-and-repeated-enumeration).

## Completed engine-process reopen; interrupted operations remain open

[PR81](PXT-STORAGE-REOPEN.md) qualified actual completed-operation backing bytes
across a fresh engine process and startup in both scheduler profiles. Data,
deletion/listing, code/neighbor canaries and the specific lost-backing control
passed independent original audits. This supersedes the earlier prospective
reopen task. The [older AIRCR case](PXT-STORAGE-RESET.md) still means CPU/core-system
reset within one Bus with peripheral models live; its original scope is unchanged.

Before implementing interrupted erase/program/commit or power-loss recovery,
freeze an explicit fault point in the actual controller/RAFFS commit sequence,
the retained backing bytes, fresh-process startup and the permitted old/new/error
outcomes. Observe the real in-flight operation and inject only the declared fault;
never repair backing or copy expected output. Use specific partial-write/order
negative controls and compare data, listing, deletion and code/neighbor canaries
in both profiles through frozen standalone original readers. Completed writes,
object remount, compaction and spacing panic `921` do not establish this boundary.
Physical power loss and complete peripheral reset require separate evidence.

## Later consumer and performance work

Preserve active-deletion/borrow-lifetime limits and natural stale IRQ/timeouts.
Native Arcade needs an admitted generated program/board image, real frames/input
and debugger stop/resume. Representative native/WASM performance needs repeated
paired measurements of that real workload. Installed adoption follows qualified
source/package/pin selection and actual installed acceptance. Acquire official
originals once and audit them without replay; every enabled final-head check and
reviewed/tested/landed tree and ordered-parent join must pass before merge.
