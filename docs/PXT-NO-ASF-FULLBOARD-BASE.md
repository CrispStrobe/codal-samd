# No-ASF SAMD51 full-board base

Owned paths: `no-asf-board/fullboard/`, `no-asf-board/support/`,
`tools/samd51-device-header/`, `tools/pxt-native-request/`,
`.github/workflows/pxt-no-asf-fullboard.yml` and this document. This closes
lane A of [the remaining lanes](PXT-NO-ASF-NEXT-LANES.md) and the compile and
link half of lanes B–D: the complete generated Arcade board compiles and links
with no ASF4, no samd-peripherals and no vendor-restricted header. It qualifies
no peripheral, runtime or USB behaviour; those lanes keep their own actual
guest qualification.

## Inputs, all byte-bound

| Input | Identity |
| --- | --- |
| MakeCode native request, `samd51adafruit` | SHA-256 `19efcdc7…f42b1`, reproduced offline from the admitted Arcade 4.2.1 `target.json` and PXT 13.2.1 `pxtworker.js` (npm payloads, verified by SHA-256) by [request.mjs](../tools/pxt-native-request/request.mjs) |
| 69 request files | 64 framework projections and 5 generated files, each equal to [pxt-framework-sources.json](../profiles/pxt-framework-sources.json) |
| Forced configuration | the reproduced 1,634-byte generated header (USB1/WebUSB1, 120 MHz, 192 KiB RAM, 10 KiB stack, vectors at 0x4000) |
| codal-core | `2ed19d64`, one reviewed adaptation (`sendConfig` frees with `delete[]`, lane G); see [the migration review](PXT-NO-ASF-CORE-MIGRATION.md) |
| Board | `codal-itsybitsy-m4` `6ecd80cc`: model, HAL and Apache-2.0 linker script |
| Device headers | generated from Microchip SAMD51 DFP 3.9.276 `ATSAMD51G19A.atdf` (pack SHA-256 `6f4e530b…`, ATDF `ab17dad8…`) |
| CMSIS core | `inc/CMSIS` v4.30, five BSD-3-Clause core headers only |

The 3.9.276 pack is Apache-2.0 in full ([notice](../tools/samd51-device-header/notices/microchip-samd51-dfp-3.9.276-LICENSE.txt)).
Earlier legacy-format packs and ASF4 device headers state "exclusively with
Microchip products" and are never read. [generate.py](../tools/samd51-device-header/generate.py)
re-expresses the ATDF facts in the legacy struct/union/macro spelling the
selected CODAL sources use: register layouts and sub-mode views, field masks
and value constants, instance addresses and parameters, the IRQ table and
vector struct, pin multiplexing macros and the pin-object table. A one-hot
value group (for example `GCLK SYNCBUSY.GENCTRL`) becomes per-bit members plus a
`vec` view. [test_generate.py](../tools/samd51-device-header/test_generate.py)
compiles static assertions for all 611 generated struct members against their
ATDF offsets and widths, detects one inserted reserved byte, refuses a changed
ATDF, and cross-checks the independently authored
[USB leaf header](../no-asf-board/usb/samd51_usb_mmio.h) against the ATDF.

## Support layer

`no-asf-board/support/` is independently authored. It
supplies exactly the call surface the selected drivers use, over the generated
registers: GPIO and pin objects, the clock tree, TC/TCC helpers and IRQ
dispatch, EIC, SERCOM clocking and SPI pad/baud helpers, a polled I2C master, an
interrupt-driven USART device, SPI master setup, a single-conversion ADC, the
named `hri` accessors, freestanding `<new>`/`<cstddef>`, the C++ allocation
entry points (no libstdc++ or libsupc++ is linked), runtime hooks and the
reset entry. Driver sources in `src/` stay byte-unchanged.

Clock plan: GCLK0 120 MHz (DPLL0), GCLK1 48 MHz DFLL (USB), GCLK2 100 MHz (DPLL1,
SERCOM, matching `SERCOM_100MHZ_CLOCK`), GCLK3 32 kHz, GCLK4 8 MHz (TC/TCC),
GCLK5 1 MHz (DPLL reference). An I2C master selects the 48 MHz generator for
its own core clock: standard-mode rates are unreachable from 100 MHz with 8-bit
`BAUD`/`BAUDLOW` fields. This differs from the historical build, which clocked
I2C from 100 MHz while its HAL computed baud values for another frequency.

## Closure and image

[build.py](../no-asf-board/fullboard/build.py) compiles 163 units in parallel
with full `-MD` output and admits every dependency into one category: request,
config, core, samd (own `src`/`inc`), board, support, device, cmsis, asm or
toolchain. ASF4 and samd-peripherals paths are refused by name. Three units
present in selected directories are recorded as inactive with their reasons:
the two SAMD21 PTC touch sources (the ATDF has no PTC module) and the board's
`startup_samd51.c`, which names TAL vectors absent from the admitted ATDF and is
replaced by the generated vector table and
[startup.c](../no-asf-board/support/startup.c). The link uses the board's
linker script through the `gcc` driver, `nano.specs` and no start files,
libnosys or libstdc++. Archive members are admitted from the linker map: only
`libc_nano`, `libm` and `libgcc` may contribute. The link must leave no
undefined symbol, no ASF symbol, the vector table at 0x4000 and exactly one PXT
function table at a 16-byte Intel-hex record start.

Lane E's first step then runs in the same job: [compile.mjs](../tools/pxt-native-request/compile.mjs)
compiles an empty public `main.ts` with the admitted worker onto this base
through `extinfo.hexinfo`, offline, and records the request, base and output
hashes.

## Development observations (not qualification)

A local development build (Arm GNU 13.3 with Ubuntu noble newlib 4.4 headers)
reproduced the closure: 163 units, 411 admitted dependencies, text 129 KiB. In a
local engine branch with additional opt-in SAMD51 models, the base booted
through reset, data/BSS, VTOR, FPU, the clock tree (both DPLLs locked from an
enabled reference channel), cache enable and static constructors to
`pxt::start`, which halts at the runtime-version check because a bare base has
no program. With the patched empty program it passed the runtime-version and
template-hash checks and entered runtime initialisation, stopping at the first
unmodelled peripheral (TC0). These observations motivate the engine models of
lanes B–D and F; they are not hosted evidence.

## Limits

No guest, peripheral, scheduler, USB/HF2, CF2 runtime, display, audio,
timing, power or licence-clearance claim. The board base is not a bootloader and
carries no boot region; the authored boot metadata is lane E/F work.
