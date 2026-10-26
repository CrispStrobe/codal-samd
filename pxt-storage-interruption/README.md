# RAFFS overwrite interruption between modeled atomic commits

Qualified and merged in [PR94](https://github.com/CrispStrobe/codal-samd/pull/94),
reviewed `3bf85564`, merged `a2a8f909`. This explicit software fault contract
is separate from completed-write reopening.

Three source-compiled caller roles retain the actual settings/RAFFS/SAMDFlash,
GC, fiber, startup and newlib composition. A seed process formats an erased
32 KiB store and writes `atomic.value` (97 bytes) and `atomic.keep` (19 bytes).
Each writer starts a fresh machine from its own ELF plus the exact seed storage
bytes, then overwrites `atomic.value` with 257 deterministic non-FF bytes.

Three fault producers export the exact backing and end before the next guest
instruction at distinct observed boundaries:

| Boundary | Fresh recovery must return |
| --- | --- |
| First payload WQW accepted, loaded buffer, before commit | Original 97 bytes |
| Publication WQW accepted after all new payload bytes commit | Original 97 bytes |
| Publication WQW committed, before the settings call returns | New 257 bytes |

A fourth writer completes normally. Fresh readers of both seed and completed
control must also recover their exact values. Every reader starts a separate
engine process and new CPU/RAM/controller state. It must mount before any
format, execute actual getters/existence/listing, preserve the neighbor, and
leave storage byte-identical. A mount failure, automatic format, panic or mixed
payload fails; a generic error is not an accepted recovery outcome.

The observer records the actual MMIO `0xa504` command, pending/LOAD state,
address and backing transition. It does not use an operation ordinal or arbitrary
stop time. RAFFS publishes a new eight-byte metadata entry only after flushing
its payload and zero padding. The old metadata remains present; overwriting a
key does not append a deletion tombstone. The fixture derives the new payload
position from actual seed metadata and checks the metadata-bearing 16-byte
program, active/inactive headers, old records, code and neighbor canaries.

The admitted engine commits each WQW atomically after four instruction-boundary
services. Dropping its pending command/page buffer by ending the process is an
authored fault between these modeled atomic commits. No partial programmed bits,
physical power-loss timing, interruption inside erase/GC, arbitrary RAFFS crash
consistency, board bootloader or hardware durability is qualified.

The original completed-write/reset/reopen callers, readers and successful
campaigns stay unchanged. Both engine profiles and independent source/library/
ELF/raw-backing/result audits are required before this candidate can close.

Further interruption families need separate contracts: deletion publication
(old present/new absent with exact directory/neighbor invariants); bank-compaction
header transitions (old/new bank admission and spare-bank preservation); erase
interruption (an explicit byte-level controller fault model before any claim).
Do not generalize this overwrite campaign to those operations.

The actual interruption campaign pins current core main
`21ab8403c6b9d32d3e28152b4da7ad6b2e2908ce`. The unchanged link-only parent
report retains its historical engine marker; the three new role reports record
the actual campaign engine explicitly. No original parent guest is replayed.

The [official campaign](https://github.com/CrispStrobe/codal-samd/actions/runs/40326059546)
passed in both default and event-scheduler profiles. Independent readers joined
actual source/library notices and selected members, all three ELF compositions,
raw flash bytes, every ordered PBC/WQW pair and ten distinct producer/recovery
processes per profile. PBC retains the prior ADDR while clearing the page buffer;
only loaded WQW commits program flash. The readers independently derive the
whole backing from that exact program prefix, including pending final operations.
All three enabled final-head checks passed. The landed tree and ordered parents
matched the reviewed merge; original pre-execution linker failures are retained.
