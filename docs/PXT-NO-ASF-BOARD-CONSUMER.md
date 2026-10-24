# No-ASF generated SAMD board consumer

Own isolated implementation paths: `no-asf-board/`,
`.github/workflows/pxt-no-asf-board.yml`, and this contract. Configuration
admission remains in the separate generated-board profile. Required engine
changes land in the owned core repository before an explicit consumer pin move.
No historical fixture, engine pin or vendor source selection changes silently.

The first executable boundary is an authored empty Arcade program using the
actual generated main, shims and runtime. Preserve the generated USB1/WebUSB1,
120MHz, 192KiB RAM, 10KiB stack and vectors at 0x4000. The pinned framework already
defines `PXT_MAIN` to call `pxt::start()`; the generated main naturally selects
it. Do not replace it with a diagnostic main or assume its unused `uBit.init()`
branch proves the historical Itsy constructor was executed.

Implementation and admission sequence:

1. Reproduce the generated configuration. Bind all 64 projected framework files,
   actual generated `pointers.cpp`, `main.cpp`, tiny `pxtconfig.h` and function
   metadata. Run the selected compiler with dependency output for every unit;
   admit each resulting header/source, include order and library member. An old
   542-file source inventory is not this compiler closure. Reject ASF4,
   vendor-restricted headers and the historical samd-peripherals selection.
2. Independently author leaf MMIO/API definitions and genuine selected drivers.
   Preserve the existing MIT SAMD USB driver through a reviewed exact include
   adaptation to the permitted register surface. Resolve pin/EIC/PWM, serial,
   SPI/I2C/DMA, timer/audio and platform dependencies explicitly. Unsupported
   paths must refuse; unused symbols are not evidence of functional peripherals.
   Review the historical core312ae57e to core2ed19d64 migration separately.
3. Implement an explicit SAMD51 USB device model with real guest SRAM descriptor
   banks and a typed external host transaction interface. Bind reset/sync,
   calibration, clocks, pinmux, address, endpoint flags/readiness/W1C/W1S and
   exact IRQ routing. The default USB stub and other chips' USB models cannot
   qualify this driver. Test bounds, descriptor alignment, RAM-only DMA, disabled
   endpoints, absent host completion, missing IRQ and malformed buffers.
4. Build an owned no-ASF base/template. Use the admitted Arcade4.2.1 target and
   worker to compile public `main.ts` containing an empty program for
   `samd51adafruit`, supplying that new base through `extinfo.hexinfo`. Bind the
   worker, target, request, template hash and final patched native image. A base
   `PXT_SHIMS_BEGIN` sentinel alone is not an executable program.
5. Author boot metadata from reviewed public PyBadge CF2 configuration only;
   preserve applicable notices and provenance. Admit the actual 16-byte UF2
   information layout, pointer at 0x3ff0 and CF2 magics, count and bounds used by
   both runtime configuration and USB. Do not execute, bundle or patch a vendor
   bootloader image. Public candidate configuration source is
   `adafruit/uf2-samdx1` at `d4dc92889759c0c551683420e24c5ef535ac303e`,
   `boards/arcade_pybadge/board_config.h`; review its Microsoft MIT terms and
   applicable third-party notices before extraction. This admits no ASF code.
6. Freeze independent readers, then qualify actual reset/data/BSS/constructors,
   relocated vectors, native program entry, CF2 reads, scheduler/IRQ/services,
   real USB enumeration/descriptors, WebUSB BOS/MS OS and HF2 transactions in
   both engine scheduler profiles. Drive host requests through the model and
   compare bytes produced by actual core/PXT code. Do not synthesize responses.
   Preserve the existing unimplemented WebUSB landing-page response.
7. Require nonvacuous configuration, transfer-completion, IRQ, descriptor and
   interface-discovery mutations. Acquire each official original once, audit
   source/image/model/event joins without replay, then merge only after every
   enabled final-head check succeeds and exact tree/parents match review.

Completed-operation storage backing reopen has its own case and cannot prove
power-loss recovery, hardware accuracy or this full consumer. A smaller USB-only
leaf guest can qualify its declared driver/model boundary; it cannot substitute
for generated native-program startup or close the board contract above.
