# Actual guest reset and retained RAFFS backing

Contract: [modeled reset boundary](../docs/PXT-STORAGE-RESET.md).

The hosted producer links the unchanged parent without executing it, then
replaces one authored probe object. Original settings/RAFFS/driver/GC/services,
flags, libraries and pins remain selected. The explicit assembly store at
`bw_reset_store` is `str r1,[r0]` with AIRCR address and keyed SYSRESETREQ value.

The observer admits the retired actual store and resulting vector/MSP, core
reset state and unchanged flash bytes. Both boots run original startup and the
same baseline60-check GC/runtime caller; data/BSS initialization is observed
before static constructors. First boot uses270 storage checks, second287, with
real method entries/returns, exactly two filesystem constructors and one format.
The retained internal file selects the second-boot path; no RAM boot cookie or
positive backing injection is used.

A separate fresh run explicitly erases storage only after observing the real
reset boundary, then requires the actual second-boot `tryMount` return0 before
its next guest instruction and before another format. It cannot qualify a
timeout, unrelated panic or silently reformatted filesystem.

Each new run has its own160-million-step inclusive bound; old bounds and tests
are unchanged. Peripheral models are not reset. No physical full reset, engine
restart/reopen, interrupted-write, power-loss or full-board claim follows.
