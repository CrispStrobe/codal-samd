# Prospective actual software-pended idle SPI IRQ dispatch

Runtime PR30 and engine PR235 now landed after exact-head checks and actual
four-case qualification. See [the immutable result record](REAL-SPI-STALE-RESULTS.md).
The retained prospective contract below does not establish natural stale IRQs,
stale timeouts, blocking-result ownership or a complete native Arcade image.

Mode13 completes a separate priming asynchronous RAM transfer through the real
callback, MessageBus Event and main-fiber resume. Its baseline recovery callback
and stage witnesses remain untouched during that epoch. The guest then masks
interrupts and writes the actual NVIC ISPR registers for IRQ31 and IRQ63;
these are software-pended dispatches, not naturally stale peripheral IRQs.

The separately observable dispatch function records full pending words before,
while armed and after unmasking, plus DMA/SPI flags, SPI interrupt enable,
control, masks and current-fiber identity. Architectural DSB/ISB barriers follow
unmasking. Pending-bit disappearance alone does not prove delivery: qualification
must observe both actual exception47/79 entries and returns before dispatch
returns, distinguish these from priming and recovery hardware completions, and
retain the full IRQ census. The no-token handlers must not create DATA/wire/CS
activity, another callback, failure or Event wake. A retained sticky TXC is
allowed; it must not be silently cleared or interpreted as another completion.

Only after this idle epoch settles does the caller execute the original positive
asynchronous and distinct blocking recovery. All priming, pending writes,
dispatch, callback-return, main-resume and recovery boundaries remain in raw
evidence. No host IRQ, callback or Event injection is part of the fixture.

This is source-only preparation. A separate clean source-bound mode13 build,
unchanged modes0–12 image comparison, actual bounded execution in both engine
profiles, positive controls, a frozen independent original reader and every
enabled exact-head check are required before qualification/merge. No driver or
model changes are proposed. Timeout staleness, blocking terminal-result leases,
object lifetime, native Arcade, physical behavior, installed consumers, RTx and
distribution/licence clearance remain separate boundaries.
