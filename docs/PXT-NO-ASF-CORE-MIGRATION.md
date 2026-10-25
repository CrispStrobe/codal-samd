# Core migration review: 312ae57e to 2ed19d64

The historical board lock (`codal-itsybitsy-m4` v0.2.7 `target-locked.json`)
pins codal-core `312ae57e`. Every qualified no-ASF consumer, and now the
[full-board base](PXT-NO-ASF-FULLBOARD-BASE.md), selects `2ed19d64`. This review
is the explicit migration check required by lane A of
[the remaining lanes](PXT-NO-ASF-NEXT-LANES.md). `312ae57e` is an ancestor of
`2ed19d64`: 72 commits, 68 changed files under `inc/` and `source/`.

## Source compatibility of the selected callers

The complete selected closure was compiled against both cores with the same
flags, request, support layer and generated device headers.

- Against `2ed19d64` every translation unit compiles and the image links.
- Against `312ae57e` 42 translation units fail, each with exactly the same five
  errors and no other: `inc/SAMDNVM.h` (included by every PXT unit through
  `platform.h`) overrides the `NVMController` interface as reworked in core
  `2ed19d6` ("Update non-volatile memory APIs to no longer assume on chip
  address space"): `erase`, `read`, `write`, `getFlashStart` and `getFlashEnd`.
  This repository's NVM driver targets the newer interface; the historical
  upstream `codal-samd` `5bd6b93c` matched the old one.

No other API difference reaches a selected caller at compile time. All 64
framework projections compile unchanged against `2ed19d64`.

## Changed functions present in the linked image

Functions changed between the two cores were intersected with the symbols of
the linked base. Each is reviewed for its effect on this board:

| Core source | Linked changed functions | Effect here |
| --- | --- | --- |
| `driver-models/Timer.cpp` | `recomputeNextTimerEvent`, `trigger` | Past-due events now use `max(timestamp - now, MINIMUM_PERIOD)`. `max` is `CodalCompat`'s `max(int, int)`, so a small negative `uint64_t` difference narrows to a negative `int` and the minimum period wins: equivalent to the old explicit branch for differences within ±2^31 µs. Larger gaps fire early and re-arm. |
| `driver-models/CodalUSB.cpp` | `add`, `initEndpoints` | Endpoint sharing behind `DEVICE_USB_ENDPOINT_SHARING`, which the generated configuration does not define: identical endpoint numbering. The `sendConfig` deallocation is separately adapted (lane G). |
| `drivers/MessageBus.cpp` | `remove` | Optional listener deletion callback; null unless installed, which no selected caller does. |
| `core/CodalHeapAllocator.cpp` | (debug output) | DMESG format strings only. |
| `driver-models/Accelerometer.cpp`, `drivers/LIS3DH.cpp`, `MMA8653.cpp`, `MPU6050.cpp` | constructors, `update`, `requestUpdate`, gesture and pitch/roll | Upstreamed sensor changes (coordinate translation fix, `Pin::isActive` IRQ polarity). The generated board selects no accelerometer pins, so these paths are linked but unqualified. |

Header-only changes reaching selected callers are the base `Pin::disconnect`,
polarity accessors, `FiberLock::getWaitCount`, `LowLevelTimer` default
implementations and the typed `DataStream` API used by `SAMDDAC`/`SAMDPDM`. They
are compile-compatible here; their runtime behaviour belongs to the peripheral
lanes B–D and is not claimed by this review.

## Conclusion

`2ed19d64` is required, not merely preferred: the selected NVM driver does not
build against the historical core. The reviewed semantic differences are
neutral for the generated configuration or confined to unselected sensors.
This review is a source audit; it does not qualify any runtime behaviour.
