# Actual clock-gated SPI timeout qualification

Modes 10 and 11 separately gate SERCOM4 MCLK and GCLK during an active transfer;
a real worker then snapshots partial DMA progress in a short masked transaction.
DWT, SysTick, DMAC and GPIO remain clocked.
The real Timer's one-shot deadline must deliver one typed timeout through its
immediate MessageBus listener in exception 15, followed by an actual Event wake
and main-fiber resume. No host callback, Event or invented time is injected.

The caller preserves writeback count and clock boundaries across the gate.
Its failure handler restores the clock before inspecting SPI ENABLE/INTEN,
DMA enable and CS, before recovery initialization can overwrite that state.
Gated reads returning zero are not cleanup proof. The original driver's gated
SPI cleanup stores were dropped in the preserved actual failure below.
Do not weaken the model,
cleanup assertion, four-million-step bound or existing positive ordering.

After successful cleanup and actual callback return/main resume, the caller
waits beyond its original one-shot deadline, requires no extra terminal delivery,
preserves actual wait-begin/end DWT and clock-counter witnesses,
then performs the unchanged positive asynchronous and blocking recovery calls.
The already-fired Timer slot is released before its listener; do not infer
failure from cancellation of a slot that has already been consumed.

Compiler checks and source controls are not guest evidence. Both separately
built modes require bounded actual execution in both engine profiles, preserved
raw gate/progress/IRQ/callback/fiber observations, immutable image/source/map
bindings, a frozen independent reader and every enabled exact-head check before
merge. Any shared-driver repair also needs affected positive and cancellation
qualification. This is not full TX service, native Arcade, physical safety,
installed-consumer, performance or distribution/licence admission.

## Preserved first actual failure and repair

The first actual [run38554696298](https://github.com/CrispStrobe/labwired-core/actions/runs/38554696298)
failed all four cases at the strict cleanup assertion. The frozen independent
reader classified each as qualification failed: one typed timeout, two DATA
acceptances, zero modeled slave wire bytes and writeback remainder 30, but SPI
still ENABLED after restoring the clock. Callback return, main resume, wait
and recovery did not complete. The failed original is retained, not relabelled.

Candidate cleanup now briefly masks interrupts, disables DMA and releases GPIO
CS before temporarily restoring both SPI clock bits. It clears SPI interrupts,
disables to the named MASTER configuration, retires TXC/error flags, then
restores the exact previous gate words and entry mask before failure delivery.
This is an explicit exclusive-SERCOM4 clock policy, not shared-clock arbitration
or cleanup with a gated DMA controller. Source review alone is insufficient;
the fresh eight-case qualification below covers all affected callers.

## Landed corrected-source checkpoint — 2026-10-14

[Runtime PR26](https://github.com/CrispStrobe/codal-samd/pull/26) merged as
`66d85eaa095aa2651276be1bcbaacf4de0e5666b`. Reviewed source
`d7a5f5a4e2607d1b5c19a3a6168bb0d286efc92a`, separately build-tested merge
`ba75d453737f2d74757e79a7e457a617fcf96a5e` and landed merge have the same
tree `a2cebb379fa0ef5f115b74b8025e4aca2f67f6fb`. Both enabled runtime checks passed.
[Separate source build38561665356](https://github.com/CrispStrobe/codal-samd/actions/runs/38561665356)
produced twelve selected images; modes 0–7 stayed unchanged. Each SPI caller
used a separate clean pinned core dependency tree with 49 selected inputs.

| Caller mode | Separately built ELF SHA256 |
| --- | --- |
| Success 8 | `a7f4d21f32f4c0f4e38fbb79b02bf4a31f5b3552f0c3cae353297128fac5e68e` |
| Cancellation 9 | `47e36edabbcd2f88b8a2bb53a906cb5cc805e1f45dfcacc7b92f743294955b56` |
| MCLK timeout 10 | `7ecaf38c756a482af4060fab9f85456bb601743033d331233b90c1d531949162` |
| GCLK timeout 11 | `0737c04e92ff96fc1f587a74e386cae1682dcae639da016782cad37bb7b750ba` |

[Engine PR231](https://github.com/CrispStrobe/labwired-core/pull/231) merged as
`f9d8d670f023382efaf0ddb4ec87ceae6d28d2a1`, after **23 enabled checks passed**
and seven declared optional checks skipped. Reviewed source
`33036f775cff6a65085b104bbd9113de7a756d73`, actual tested merge
`9f32b8e7a29b89402db6a75c4df55b097cf19d7f` and landed merge share tree
`860f4fda4e8bd34b4215bbbc81cead848b8ac934`; exact parents and remote refs
were verified. No consumer pin moved.

[Actual run38562339148](https://github.com/CrispStrobe/labwired-core/actions/runs/38562339148)
passed all eight cases under default and event-scheduler profiles. Success
completed at 126844 guest steps with 28 causal rows and six actual context swaps;
cancellation/recovery at 145483 steps with nine prefix rows, 28 recovery rows
and eight swaps. Both profiles matched exactly.

MCLK and GCLK timeouts completed at 448932 and 448936 steps respectively in
each profile. Each preserved two DATA acceptances, zero modeled wire bytes,
writeback remainder 30 and sixteen raw prefix rows. Cleanup's temporary clock
restore and subsequent re-gate remain visible: two off writes and three on
writes. Restored-clock witnesses show disabled MASTER `0x2000c`, INTEN zero
and disabled DMA TX `0x200d00`, before recovery initialization. Exact previous
gate words were retained. One typed timeout entered IRQ15 with PRIMASK zero;
actual callback return, Event/main resume and a 1504 modeled-microsecond wait
preceded the unchanged 28-row asynchronous/blocking recovery. Nine actual
context swaps and IRQ entry/return counts `[1,2,2]` matched. No duplicate terminal
delivery occurred within this finite observation.

The independent reader was frozen against the separate build hashes before
official guest-log acquisition. Its SHA256 is
`409d39aa933eb51801607f1e62034eb7807305ce941c4d221917fb7a893e835f`.
Complete synthetic envelopes, missing/malformed/mixed-failure refusals, provider
bindings and temporary-clock controls passed before the original was acquired
once and independently audited. The unchanged 298094-byte original log has
SHA256 `cc21472c71124c1a4f633b2011c0ab2b8c636f0c220516cb6d2625bd3cfd614c`.
No guest replay or producer-helper import was used for the audit.

Remaining slices: actual DMA error, stale IRQ and blocking-result ownership;
then required board APIs/reset, CF2/build/notices/image admission, native Arcade
frames/input/debugger, representative RTx and installed-consumer adoption.
This finite exclusive-SERCOM/clocked-DMAC fixture is not physical calibration,
general clock arbitration, complete native Arcade or distribution clearance.
