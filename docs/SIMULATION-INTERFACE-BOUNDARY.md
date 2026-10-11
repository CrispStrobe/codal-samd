# Simulation interface replacement boundary — 2026-10-09

Status: source review and implementation contract, not a completed API/ABI
inventory or runtime replacement. Follow the
[R1 plan](https://github.com/CrispStrobe/labwired-core/blob/main/docs/engineering/samd-runtime-replacement-plan.md).
The state-control prototype is unconnected to these drivers. Its passing
host/ARM tests do not remove inherited ASF dependencies or admit an image.

All observations below use immutable `codal-samd` source
`5bd6b93c219c7e784e885ba2d6812809fb6289a8`. Interfaces and implementation were
read for this integration plan. No isolated or whole-tree clean-room claim.
Preserve inherited attribution and review each selected component separately.

## Concrete header and implementation seams

| Source | Observed coupling | Required replacement decision |
| --- | --- | --- |
| [ZSPI.h](https://github.com/lancaster-university/codal-samd/blob/5bd6b93c219c7e784e885ba2d6812809fb6289a8/inc/ZSPI.h) | Includes `hal_spi_m_sync.h`, embeds `spi_m_sync_descriptor` and exposes `Sercom` internally. | Preserve selected CODAL SPI methods, replace internal descriptor/device state and rebuild consumers. A function-only shim does not remove this header dependency. |
| [ZI2C.h](https://github.com/lancaster-university/codal-samd/blob/5bd6b93c219c7e784e885ba2d6812809fb6289a8/inc/ZI2C.h) | Includes `sercom.h`/`hal_i2c_m_sync.h`, embeds `i2c_m_sync_desc` and stores `Sercom*`. | Define replacement instance state, error and repeated-start semantics before implementing the selected I2C service. |
| [SAMDSerial.h](https://github.com/lancaster-university/codal-samd/blob/5bd6b93c219c7e784e885ba2d6812809fb6289a8/inc/SAMDSerial.h) | Includes `hal_usart_async.h`/`sercom.h` and exposes public `::_usart_async_device USART_INSTANCE`. | Inventory direct users of this public field. Merely preserving virtual `putc/getc` does not establish source/ABI compatibility. Replacing the field requires coordinated consumer changes. |
| [SAMDDMAC.h](https://github.com/lancaster-university/codal-samd/blob/5bd6b93c219c7e784e885ba2d6812809fb6289a8/inc/SAMDDMAC.h) | Includes `sam.h` and exposes device-defined `DmacDescriptor` references. | Review descriptor origins/layout and all callers. Runtime software-slot count and engine hardware-channel extent remain separate. |
| [ZPin.h](https://github.com/lancaster-university/codal-samd/blob/5bd6b93c219c7e784e885ba2d6812809fb6289a8/inc/ZPin.h) | Pulls PWM/EIC interfaces; offers GPIO, analog, servo, touch and event APIs. | Do not advertise all methods from a GPIO-only replacement. Needed analog/events must work; unimplemented services must reject explicitly. |
| [platform_includes.h](https://github.com/lancaster-university/codal-samd/blob/5bd6b93c219c7e784e885ba2d6812809fb6289a8/inc/platform_includes.h) | Includes `sam.h` at the platform header boundary. | Review device/CMSIS definitions and every transitive consumer; individual HAL-header removal cannot by itself close the platform header graph. |

The pinned [ZI2C implementation](https://github.com/lancaster-university/codal-samd/blob/5bd6b93c219c7e784e885ba2d6812809fb6289a8/src/ZI2C.cpp)
uses HAL initialization/baud/enable state, `_i2c_m_sync_transfer`, pin/clock
helpers and direct BUSSTATE access. Repeated-start and bus-error behavior need
actual contracts and positive/negative guests; do not synthesize ACK or success.

The pinned [SAMDSerial implementation](https://github.com/lancaster-university/codal-samd/blob/5bd6b93c219c7e784e885ba2d6812809fb6289a8/src/SAMDSerial.cpp)
mixes direct DRE/RXC DATA polling with `_usart_async_*` initialization, interrupt
selection, baud calculation and callbacks. A polling `putc` test alone cannot
qualify the original asynchronous serial API or handler/state contributions.

The pinned [ZPin implementation](https://github.com/lancaster-university/codal-samd/blob/5bd6b93c219c7e784e885ba2d6812809fb6289a8/src/ZPin.cpp)
uses HAL GPIO and ADC interfaces. For SAMD51, analog selection chooses ADC0 or
ADC1 from pin metadata and calls `samd_peripherals_adc_setup`; subsequent
reference/resolution/channel/input/read/deinit calls still use the HAL.
The separate SAMD21 calibration branch is not a SAMD51 replacement recipe.
Source comments about discarding a conversion are not independent hardware
evidence. Pin metadata, clock setup, ADC units and conversion/error semantics
remain separately reviewable inputs.

## Next source change: explicit platform selection and consumer closure

1. Create a separately named simulation build selector, leaving historical
   configuration and images unchanged. Record exact own-fork source pins and
   enumerate selected compilation units, includes, generated headers and scripts.
2. Map all direct public-field/type users and the captured transitive consumers.
   Declare which CODAL-facing methods/types remain compatible and which require
   caller edits. No binary-layout compatibility with ASF descriptors is promised.
3. Introduce reviewed, narrow hardware-facing interfaces without ASF include
   fallback. Preserve attribution. Device headers, CMSIS, startup, assembly,
   clocks/cache and samd-peripherals require their own origin decisions; directory
   names or source-comment flags do not settle licence or retained-inline status.
4. Rebuild every affected consumer cleanly on hosted CI. Missing declarations,
   unknown sources and unsupported required services block the profile; do not
   link success stubs for omitted ADC/I2C/UART/USB/storage/audio features.
5. Qualify incremental startup/GPIO/polling guests, then independently IRQ/DMA
   and callback/fiber behavior. Preserve the original state/error distinction.
   No state-control marker is a peripheral completion witness.

Deliver an exact source/header/API/consumer manifest, changed-file list,
explicit exclusions and positive/negative guest plan. Until that closure and
R3 source/map/image admission pass, the historical images remain rejected and
the new profile must not be presented as native PyBadge or approved hardware
firmware. No app pin, benchmark, physical capture or store/legal claim changes.
