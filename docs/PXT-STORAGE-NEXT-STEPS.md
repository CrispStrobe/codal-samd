# Selected SAMD runtime and storage: status and next work

Status reconciled on 2026-10-29. Refresh `master`, open PRs and ownership before
starting. Preserve qualified cases and original failures. Existing finite
storage/runtime cases retain their recorded `79071f69` engine; fullboard and USB
suites have separate explicit selections. This documentation changes no source,
engine pin, request adaptation or storage format selection. New selections need
affected clean actual guest qualification.

## Implemented compiler and runtime boundaries

The selected compile/link closure is implemented in the
[no-ASF fullboard base](PXT-NO-ASF-FULLBOARD-BASE.md), with an explicit
[core API migration review](PXT-NO-ASF-CORE-MIGRATION.md). The
[generated configuration](PXT-GENERATED-BOARD-PROFILE.md) retains USB1/WebUSB1,
120 MHz, 192 KiB RAM and a 10 KiB stack. The 64-file framework projection and
historical census remain source inventories; they are not blanket admission of
other compiler, header or library selections.

[Lanes A–G](PXT-NO-ASF-NEXT-LANES.md) are implemented for the selected software
board: adapters, generated native PXT programs, authored CF2 reads, natural
startup, enabled USB/HF2 and endpoint lifetime/repeated enumeration have named
[native cases](../no-asf-board/native/README.md). The
[USB-only leaf](../no-asf-board/usb/README.md) and
[authored CF2 component](PXT-CF2-AUTHORED-METADATA.md) retain their narrower
historical boundaries; neither alone proves the later integrated runtime.
Required features stay enabled. Preserve source/notice bindings and refusals of
ASF4, restricted vendor headers, guessed layouts and successful return stubs.

## Completed reopening and modeled interruption

[Completed-operation reopening](PXT-STORAGE-REOPEN.md) qualifies backing bytes
across a fresh engine process and startup in both scheduler profiles, including
data, deletion/listing, code/neighbor canaries and a specific lost-backing control.
The [older AIRCR case](PXT-STORAGE-RESET.md) still means CPU/core-system reset in
one Bus with peripheral models live; its scope has not expanded.

[Interrupted settings overwrite](PXT-STORAGE-INTERRUPTION.md), merged in
[PR94](https://github.com/CrispStrobe/codal-samd/pull/94), qualifies three actual
NVM boundaries with strict old/old/new recovery in fresh processes. That receipt
is not retroactive evidence for deletion, bank compaction, erase or torn writes.
Those families now have separate authored cases in the
[native suite](../no-asf-board/native/README.md): `store-delete`, `store-gc`,
`store-gc2`, `store-clear`, `store-scope`, `store-gc-torn` and `store-gc-garbage`.
They observe committed-operation cuts and explicitly bounded torn-operation or
invalid-bank models, with retained original failure controls. Do not describe
all interrupted-operation work as unimplemented or treat these modeled faults
as physical power-loss observations.

## RAFFS payload integrity and migration

[RAFFS v2](RAFFS-V2-CHECKSUM.md) is implemented through the exact-source
`REQUEST_FIXES` adaptation. Payload/CRC corruption and lost-data cases recover
the previous valid value; the unadapted failure remains a negative control.
The native migration case covers all 13 committed-operation cuts in both profiles.
Independent CRC vectors and the byte-bound host filesystem controls supplement
that native evidence. Keep every case's stated fault envelope.

[PR103](https://github.com/CrispStrobe/codal-samd/pull/103) merged as
`adbf5a84a555e150fc5ff727b76ea392f1f3ef73`; all four default/event-scheduler
fullboard shards passed in
[run40734675974](https://github.com/CrispStrobe/codal-samd/actions/runs/40734675974).
The same selection includes the faster display-clock work described in
[H1/H3](PXT-NO-ASF-NEXT-LANES.md). Preserve the initial UART calculation failure
and [recorded image/log identities](receipts/2026-10-28-clock-raffs-native.json).
Nominal frame cycles are not measured host real-time performance.

CRC32 detects accidental corruption, not authenticated data. Legacy banks remain
readable and migration preflights space, but original firmware cannot read a v2
bank. A later consumer adoption must explicitly review format/downgrade
compatibility and qualify its actual selected guest. This status update does
not move the older finite cases or an installed application to that selection.

## Remaining work and entry conditions

- **Hardware observations:** [H2](PXT-NO-ASF-NEXT-LANES.md#h2-hardware-observations-needs-a-pybadge-and-instruments)
  requires a board and instruments for flash power loss, glass colour, analogue
  levels and oscillator drift. Freeze fault points, backing and permitted
  outcomes before a new case. Record revision, supply, temperature and tolerance;
  never repair backing or copy expected output. Modeled tears, completed writes
  and reset within one Bus do not establish physical power loss or complete
  peripheral reset.
- **New board selections:** SPI slave support needs an actual peer consumer;
  PDM, HF2 flashing/bootloader handover, JACDAC and GhostFAT have the explicit
  selection conditions in [H4/H5](PXT-NO-ASF-NEXT-LANES.md). They are not missing
  prerequisites of the currently selected PyBadge software board.
- **Application and performance acceptance:** the existing
  [source-built native/Node game results](SAMD-NATIVE-GAME-RESULTS.md) already
  establish their finite functional frames/input boundary. Normal project
  compile/import, debugger stop/resume and GUI-route acceptance remain separate.
  Check the active [Lite PR1533](https://github.com/CrispStrobe/brickwright-lite/pull/1533)
  before changing its consumer lane. Representative active native/WASM speed
  needs paired measurements of the actual workload; nominal cycles and idle
  spin are insufficient. Preserve active-deletion/borrow-lifetime limits and
  natural stale IRQ/timeouts.

Engine drift acknowledgements remain the operator-owned Core task in
[H6](PXT-NO-ASF-NEXT-LANES.md#h6-engine-drift-acknowledgements-labwired-core-operator-decision),
not a storage implementation gap or permission to renew dates. Acquire official
originals once and audit them without guest replay. Every enabled final-head
check and reviewed/tested/landed tree/ordered-parent join remains required for
later source changes. Historical receipts, captures, baselines and limits stay
unchanged.
