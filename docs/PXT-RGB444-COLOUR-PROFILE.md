# Explicit native RGB444 simulation colour profile

The admitted MakeCode request selects the known-Adafruit direct RGB444 path.
That path writes pixel nibbles without sending RGBSET. The existing ST7735R
controller-memory model correctly keeps those colours unknown after power-on.
The original path and its wire-code qualification remain unchanged.

This candidate adds an **opt-in authored simulation runtime profile**, selected
by `fullboard/build.py --colour-profile rgb444-lut`. Only the owned extracted
compile copy of the byte-bound screen source changes. It sends command `0x2d`
and 128 actual SPI data bytes after COLMOD/INVOFF initialization. The original
request, historical source profile, CF2 board ID and default build are unchanged.
The build report records the original and compiled source hashes and selection.

The R32/G64/B32 table expands indices 0–15 with `(n << 2) | (n >> 2)` and
saturates the unused higher indices at 63. This is a deliberately specified
RGB444-only software contract. It is neither an inferred power-on table nor a
measurement of an installed module. RGB565, analogue output, gamma, glass
aperture and physical colour matching are outside this profile.

Hosted acceptance uses the original native frame program, actual Reset/runtime,
SPI/DMA path and sampled PB5/PB7 signals. A tap feeds each actual received byte
into the existing engine's unregistered 132×162 controller-memory prototype;
its descriptor is retained unchanged. The checker accounts for MADCTL=0x80
and checks every one of 19,200 main-area pixels, including the 16×16 sprite.
The expanded controller channels are background `[117,223,85]` and sprite
`[255,32,32]`. These are stored model channels, not glass RGB values.

Two actual native controls retain identical artwork: the original no-upload
base and a separately compiled 127-byte RGBSET upload. Both must leave the
entire observed frame unknown. The positive must contain exactly one complete
128-byte upload and yield the exact frame digest. No host LUT is installed and
no pixel, completion, DMA interrupt or guest response is injected.

The fullboard workflow retains all earlier cases and adds these three in both
engine profiles. Merge requires their official successful originals, an
independent source/configuration/image/result audit, and every enabled final-head
check. Local source controls alone do not qualify this candidate. Downstream
adoption needs its own package and native/WASM acceptance; it does not follow
from this fixture. Physical panel binding remains an independently scoped task.
