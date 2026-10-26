# Source-built SAMD game: native and Node WASM results

[PR93](https://github.com/CrispStrobe/codal-samd/pull/93) qualified the actual
[game source](../samd-game/game.ts) in three fresh native processes and three
fresh Node WASM instances. Both execute the compiled ARM program, with JIT
disabled in WASM. The workload has a moving ball, eight targets, a score display,
a player paddle, and actual press/release input. It uses the opt-in authored
RGB444 upload and the software display view; it is not a silicon capture.

## Exact source and observations

Reviewed source: `8e4781e3a455d616f48780bd70fc7b587e917a3c`.
The [official run](https://github.com/CrispStrobe/codal-samd/actions/runs/40334385377)
built checkout `3ae3c46acfb0347f1001d782b82be886c06ac622` with Core
`c122b91f2c9f67cea6b9ddde14a6c40fc4026ce8`. The resulting flash image SHA-256
is `9d4bf919115e8c63eaede25b21ba655d779227b517facb70c1c82a6cc3509c72`.
The `samd-active-game` artifact archive SHA-256 is
`3fab3e5b45476c75893bfe9b3ff6f508ec9d56dbe3a374000ac6f0e45ce786ea`.

Independent source, package, image and raw-frame checks passed for all six
samples. Frames are collected after actual `updateScreen` returns to its
original caller with the matching stack pointer. Each fully known 128×160
controller view contains the expected player, ball and targets. Ball movement,
player movement, the pressed colour and the restored released colour agree
between native and WASM observations. Host code does not paint expected pixels
into the guest display or replace guest callbacks.

The original failed picture campaigns are retained. Their diagnostic image
showed the system menu paused: the authored input configuration had selected
active-high virtual pins for an active-low shift-register model. The final game
uses PXT's existing active-low virtual pin IDs 1050–1057. Its compiled configuration
was independently decoded and checked; the original runtime, physical input
model, frame budgets and picture assertions were preserved.

## Measured observer-inclusive wall time

Each value is the median of three fresh samples, in seconds. The first-six-frame
window spans five intervals after the first frame. These are process-level
observations including the frame observer, not isolated emulator timings.

| Observed phase | Native | Node WASM |
| --- | ---: | ---: |
| Startup to first observed frame | 40.037 | 78.012 |
| First six observed frames | 15.785 | 30.835 |
| Press response campaign | 30.464 | 57.412 |
| Release response campaign | 17.658 | 33.175 |
| Whole campaign | 103.964 | 199.762 |

This establishes functional execution and a reproducible measurement boundary.
It does not establish real-time playability, browser performance, installed-app
acceptance, isolated emulator memory use or physical panel accuracy.

## Remaining work and acceptance

1. Complete the explicit Core assembly's exact-head source tests and actual CLI
   game coverage in [Core PR262](https://github.com/CrispStrobe/labwired-core/pull/262).
   Obtain real throughput and deterministic measurements before admitting any
   initial baseline. Preserve original failures and existing thresholds. A
   reset-inactive optimization does not by itself qualify this active game.
2. Build the production WASM engine and portable configuration asset reproducibly
   from the qualified Core commit. Require matching independent build outputs,
   the retained engine suite and fresh actual game execution before publication.
   Bind freshly compiled firmware to its own source and toolchain manifest; do
   not relabel it as the original image above. Retain component notices.
3. Adopt the immutable qualified engine/configuration in Lite. Bind exact asset
   hashes and source to fresh browser and actual installed-app game executions,
   including moving frames and press/release responses. Require enabled final-head
   checks before merge. Keep the existing Arcade JavaScript route's qualification
   separate from a future native compilation/import route.

Physical display aperture/colour, analog audio, hardware flashing and universal
peripheral timing remain separate work.
