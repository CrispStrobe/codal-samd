# Interrupted settings overwrite: explicit atomic-boundary campaign

The [qualified contract](../pxt-storage-interruption/README.md) specifies three
actual NVM boundaries and strict old/old/new recovery in separate fresh engine
processes. Both profiles passed the
[official campaign](https://github.com/CrispStrobe/codal-samd/actions/runs/40326059546)
and independent original source/library/ELF, atomic-program-prefix, complete
backing and fresh-process audits. [PR94](https://github.com/CrispStrobe/codal-samd/pull/94)
is merged at `a2a8f909` after all three enabled checks passed.

This extends [completed-write reopening](PXT-STORAGE-REOPEN.md). It does not
replace that evidence or qualify physical power loss, torn programming,
interrupted deletion, bank compaction or erase. Those remaining families have
separate proposed acceptance contracts in the qualified README.
