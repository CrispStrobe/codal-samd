# Source-bound active SAMD game campaign

Pending qualification. The authored `samd-game/game.ts` is a small paddle game:
eight targets, a bouncing ball, overlap callbacks, score, a moving player, and an
A-button colour/audio response. This active workload is separate from old idle
spin measurements and from the complete native peripheral correctness suite.

The hosted campaign compiles the actual program onto the owned no-ASF runtime's
explicit RGBSET profile. The exact engine's standalone simulation descriptor,
system, display and authored calibration blobs are retained with the native
image and its flash ELF wrapper. No cloud-precompiled base or bootloader is used.

Three fresh native and three fresh Node-WASM instances run the same sequence.
Each observer stops at the actual `pxt::updateScreen` entry and its original
return address, checking the same stack pointer. The observer records actual
controller-memory bytes after that call, with complete colour validity. It
requires a 16×16 player, eight 8×8 targets, an 8×8 moving ball, and actual right/
A press and release effects. It does not replace a guest, inject pixels, complete
DMA or synthesize a button callback. Native and WASM source/profile/image/input
bindings and independent original-frame audits remain required before adoption.

Measurements include cold start to the first valid game picture, five render
return intervals, observed input latency, total wall time and process memory.
The render interval includes observation cost and is a bounded active-game
measurement, not a universal frame-rate or real-time guarantee. Process memory
is not isolated emulator memory. Samples and firmware snapshots are retained,
including failures; there is no new relaxed correctness threshold.

The WASM sample is Node execution of the actual engine, not browser or installed
Lite acceptance. A reproducible producer release, all enabled exact-head gates,
then actual browser and installed package qualification remain necessary before
the separate Lite adoption claim can close. Physical panel, gamma, module
aperture, electrical timing and hardware accuracy remain outside this campaign.

The first native campaign failed its unchanged game-picture budget; rejected
frames were not retained by that observer, so the cause remains unresolved.
The diagnostic candidate adds original rejected RGB frames and colour
histograms. A later pre-guest workflow failure was an observer admission error:
the canonical conformance writer passed and reproduced both tracked outputs
exactly, but the workflow required them to change. The corrected guard requires
no tracked drift. Neither failure is converted into guest qualification.

The diagnostic original identifies the first picture failure: the real game
remained in its PAUSED system menu. The authored default CF2 assigns virtual
button pins1000–1007 (active high), while the attached existing PyBadge shift
register models active-low inputs. Its released high levels were therefore
interpreted as pressed, including Menu. The corrected game declares all eight
standard active-low virtual inputs1050–1057 in its native `userconfig`; the
original CF2, runtime, physical input model, drawing requirements and budgets
remain unchanged. Qualification must independently read these exact key/value
pairs through the actual compiled bytecode configuration pointer, then prove
normal movement and press/release. A picture of the menu is not game acceptance.
