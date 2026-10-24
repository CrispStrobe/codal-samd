# Engine-process restart with exact completed-write backing

This separate case owns `pxt-storage-reopen/`,
`.github/workflows/pxt-storage-reopen.yml` and this document. Reuse the unchanged
actual settings/RAFFS/flash/GC/services link-only parent and existing selected
engine79071f69; replace only the new authored caller/observer. Existing callers,
profiles, pins, bounds and original evidence remain unchanged.

One actual guest completes format/write/deletion/read/list and GC/runtime checks,
then quiesces. Require all original calls returned and no pending flash operation.
Serialize exact base0/512KiB Bus flash bytes with source/engine/ELF/descriptor,
geometry and full/storage hashes in a bounded exclusive owned envelope. Close
and read back the actual file. Never reconstruct expected values as backing.
Snapshots are not a substitute: this engine's snapshot formats do not admit
flat SAMD backing or a complete CortexM runtime restoration.

A separate test-process invocation constructs new CPU, Bus, controllers and RAM,
loads the same admitted ELF, validates the closed original envelope and restores
its exact backing before the first guest instruction. Transfer no CPU, RAM,
peripheral state or live filesystem object. Observe real data/BSS initialization,
constructors/runtime and a fresh actual mount, exact retained/deleted/internal
values and listings plus initialized-data/code/neighbor canaries. Record separate
process and construction observations; a second Machine in one test alone cannot
qualify engine-process restart.

Another fresh process admits that same immutable backing, then applies one named
storage-loss fault to its owned memory only. Require the actual failed mount
before any format/write. Never edit the retained original to run a negative.
Missing identity, changed geometry/image/hash, truncated/noncanonical envelope,
quiescence failure, setup errors, unrelated panic and timeout must refuse.

Both scheduler profiles require source-bound hosted execution, exact producer/
consumer/file joins, specific negative controls and independent readers frozen
before execution. Acquire each official original once, never replay a successful
guest to read it, and preserve every failure. All enabled exact-head checks and
reviewed/predicted/landed trees/parents precede a normal matched-head merge.

This is explicit fixture backing transport after completed operations, not a new
production persistence API, snapshot resume, interrupted erase/program recovery,
power-loss durability, complete peripheral reset, hardware, native Arcade or
full-board qualification. Generated-board USB1 requirements remain separate.

The exact mount-check census includes internal settings/RAFFS calls. The producer
has23: two explicit checks, six for three sets, two for remove, three for a
successful get, two for exists, and eight for two listings. Each listing performs
one mountedStorage check and three dirRead/lock/mount checks for two live files
plus termination. The fresh reader has21: one explicit check, four for two exists,
six for two successful gets, two for the missing get and eight for the listings.
These follow the pinned mountedStorage, read, lock/mount and dirRead bodies; the
initial blank check must return0, every later positive check1. Counting only the
explicit calls incorrectly expects2/1; those original failed observers remain
retained. No caller, stored data, checks, bound or engine was changed to fix that
observer accounting.
