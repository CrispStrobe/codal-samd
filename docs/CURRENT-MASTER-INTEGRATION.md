# Current-master simulation integration — 2026-10-11

This separate integration branch reconciles the authored simulation work with
current upstream/default `master`. It is source preparation, not qualified
default adoption, whole-runtime admission or a completed native PyBadge build.

## Exact base and preserved changes

Both upstream and our fork's default `master` were
`0320025836c8994a3126f0d92cf79e8a91f50e61` at refresh. The historical simulation
stack starts from `5bd6b93c219c7e784e885ba2d6812809fb6289a8`; the five commits
between these bases were already present on our default branch, not missing
from the fork as a whole.

Integration merge `199307ce2df51e56064deb891673cd4b9a7744b5` has parents
`3f618f87c31e06e9b1e402ec56d09ef671b9289e` (paired-IRQ prototype) and
`0320025836c8994a3126f0d92cf79e8a91f50e61` (current master).
The normal merge had no conflicts. Its five changed legacy files are exactly
the upstream changes; their integrated bytes match current master:

| Preserved current-master files | Change |
| --- | --- |
| `inc/SAMDNVM.h`, `src/SAMDNVM.cpp` | Integer-address NVM API and `read` compatibility changes |
| `inc/SAMDSerial.h`, `inc/ZSingleWireSerial.h` | Replace outdated serial/JACDAC includes |
| `src/SAMDDAC.cpp` | Set output to zero when the playback buffer is empty |

Relative to current master, the simulation integration adds new paths only:
no existing master file is modified or deleted. Runtime CMake selection remains
unchanged and still requires its separate component/build review. Bringing the
five upstream changes into this branch does not select an ASF-free runtime.

## Immutable evidence is not rewritten

All existing simulation, profile and census implementation bytes are unchanged
from the paired-IRQ prototype. Historical source pins, including
`2a991760339fe89f5272f923baf89dc18fd434d0`, remain in their original fixtures and
receipts. The old branches are retained; no rebase or forced push is required.
Their earlier guest results are not automatically current-master qualification.

The [strict paired profile](../paired-irq/README.md) remains opt-in, unconnected
to actual CODAL consumers/scheduler and incompatible with historical unmatched
enable-at-zero semantics unless callers/startup are explicitly migrated. Its
advisory switch guard does not prove real fiber ownership enforcement.

## Acceptance and continuation

First pass every enabled exact-head hosted source/profile/census/policy control
and ARM build on this current-master integration branch. Preserve failures and
distinguish pure controls and linked authored fixtures from guest execution.
Then pin the actual qualified integration source in a separately named LabWired
guest harness and qualify the positive/four-refusal paired-ABI controls in both
scheduler profiles with vector/BSS/entry/IRQ/fault/liveness witnesses.

After that, select compatible immutable core/board dependencies for the changed
NVM/serial API; rebuild representative **actual** consumers in the named no-ASF
simulation profile. Unsupported required services must fail, not link success
stubs. Integrate actual Timer/Fiber/Heap/early-return/startup/panic callers and
the scheduler's critical-section ownership guard. Finish origins/notices and
required USB/WebUSB before runtime admission, engine/application adoption or
active native/WASM RTx claims. No hardware or store qualification follows from
this source merge.
