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

Completed: all eight enabled source/profile/census/policy and ARM build checks
passed on `3c66f0d27c865192a2b22217e71c1f88f2b3fe91`.
[LabWired run38153786907](https://github.com/CrispStrobe/labwired-core/actions/runs/38153786907)
then passed the positive/four-refusal authored controls in both scheduler profiles,
with source-bound startup/vector/BSS/entry/IRQ/fault and bounded refusal witnesses.
The original packet passed a separate source/log audit without replay.
[PR10](https://github.com/CrispStrobe/codal-samd/pull/10) merged as
`c2274c7ed397787820c92a91438c4c0a902612b8`; reviewed and landed tree
`8ed57db57606e4ea644609e33c6eaf385beb623b` matched. Earlier failed mutation
controls remain preserved in their original runs. Broader LabWired PR198 CI and
additional guest adversaries remain separate, unfinished gates.

After that, select compatible immutable core/board dependencies for the changed
NVM/serial API; rebuild representative **actual** consumers in the named no-ASF
simulation profile. Unsupported required services must fail, not link success
stubs. Integrate actual Timer/Fiber/Heap/early-return/startup/panic callers and
the scheduler's critical-section ownership guard. Finish origins/notices and
required USB/WebUSB before runtime admission, engine/application adoption or
active native/WASM RTx claims. No hardware or store qualification follows from
this source merge.
