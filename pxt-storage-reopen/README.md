# Completed-write backing across separate engine processes

Hosted-only actual guest case. The producer, successful reopen and lost-backing
negative run in three separate exact Cargo test invocations. Each constructs a
new CPU, Bus, controllers and RAM. They share only the admitted ELF and a closed,
read-back 512KiB flash file with a bounded canonical identity manifest.

The caller retains the actual settings/RAFFS/GC/services implementation and
startup. The producer completes 270 checks, then quiesces with every operation
returned and no pending flash operation. A fresh process initializes data/BSS,
runs constructors, mounts the exact persisted bytes and completes 287 checks.
A third process erases only its owned storage memory and must observe the real
failed mount before any format or write. It never edits the retained backing.

The file transport is an authored qualification fixture. File sync and closed
readback are checked; this does not establish physical durability, interrupted
writes, power loss, snapshot restoration, full-board or enabled USB operation.
Both scheduler profiles and independent original readers are required before
qualified merge. See [the case contract](../docs/PXT-STORAGE-REOPEN.md).
