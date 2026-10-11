# Authored strict paired-IRQ guest qualification — 2026-10-11

This result supersedes the prospective qualification steps in the retained
[prototype guide](../paired-irq/README.md), without rewriting historical pins.
It qualifies bounded authored controls, not real CODAL callers or a whole runtime.

Source `3c66f0d27c865192a2b22217e71c1f88f2b3fe91` passed all eight enabled
source checks and merged through [PR10](https://github.com/CrispStrobe/codal-samd/pull/10)
as `c2274c7ed397787820c92a91438c4c0a902612b8`, with identical reviewed/landed trees.
[LabWired run38153786907](https://github.com/CrispStrobe/labwired-core/actions/runs/38153786907)
at source `e7589656998d3a08cd59670ff09a4f3214310d2e` passed all five authored
ARM cases under ordinary and event scheduling.

| Authored case | Status | Fault | Guest steps, each profile | Terminal refusal instructions |
| --- | --- | --- | --- | --- |
| Positive nested/restoration/IRQ | `0x600d` | 0 | 3791 | 0 |
| Underflow | `0x201` | 3 | 369 | 64 |
| Changed diagnostic identity | `0x202` | 5 | 624 | 64 |
| Depth limit | `0x203` | 4 | 16644 | 64 |
| Lost masking | `0x204` | 6 | 638 | 64 |

The guest validates vectors, startup BSS/data initialization, entry-mask
restoration and exactly one positive stage2/mask0/exception16 IRQ, followed by
thread return. Refusals remain masked in thread mode with stable exact diagnostics
and zero IRQ witnesses. These are bounded observations, not indefinite liveness.
Diagnostic identity changes are not real fiber switching.

The actual compiled mask-restore helper is permitted for at most 32 transition
instructions; the 64-instruction terminal count resets across it, with a
192-observation transition bound. Caller escape, unrelated code, stuck helper
and stopped PC are rejected by the harness and its pure progress controls.
Additional guest-level policy mutants and full recording-entry-point adversaries
remain owed; host policy mutants are not those guest adversaries.

## Original evidence and audit

The sole [official artifact11707374717](https://api.github.com/repos/CrispStrobe/labwired-core/actions/artifacts/11707374717)
is `authored-paired-irq-guest-results`, 16,643 bytes, ZIP SHA256
`374e6bd22a8c7eec1de729a9e14e63f837e79bb5d4309f723b360aac2570a83f`.
Its original `paired-qualification.json` is 4,291 bytes, SHA256
`ff7a67aabfcf7410da46020fc2b067f914d8808a6d05e9a915b663b62b444d4e`.

A separate standard-library audit without producer imports checked official
run/head/attempt/success, artifact identity/size, qualification merge tree and
parents, before/after source snapshots, six actual source-file hashes, both log
hashes and all ten result rows. It passed. Official Git commit metadata also
confirmed qualification merge `f074b54d48cf4381b6832aab7ce5abc4aaea45d3`, tree
`3c2c12a69ed4abdab301b5528eeb5cd8d462e6a6`, and its recorded parents.
This was not replay, independent-agent testimony, full toolchain closure or
independent rehashing of absent ELF binaries. The packet deliberately excludes ELFs.

Broader [LabWired PR198](https://github.com/CrispStrobe/labwired-core/pull/198)
CI remains a separate merge gate at this checkpoint. Real Timer/Fiber/Heap,
startup/panic and scheduler integration, compatible immutable core/board inputs,
a named no-ASF build, component notices, required USB/WebUSB, peripheral IRQ/DMA,
boot/display/debugger/package adoption and active RTx remain unqualified.
