# Current SAMD full-board source census

This source-only task owns `tools/fullboard-source-census/`,
`profiles/pxt-fullboard-source-census.json`,
`.github/workflows/pxt-fullboard-source-census.yml` and this document.
Preserve the historical storage census and every existing source/fixture/pin.

Bind the already selected framework/core/engine and current owning Codal
headers/configuration/build files by Git objects and SHA256. Distinguish the
qualified service profile from conditional full-board configurations. Report
header dependencies requiring replacement without selecting ASF4 or vendor
hardware-use-restricted inputs for a simulation build. Separate descriptor
stubs, native models, actual projected source uses and unknown generated
configuration. No compile, guest, board, boot or blanket licence claim follows.

A reproducible report and nonvacuous source/descriptor/configuration controls
must pass hosted CI before merge. Source-only qualification passed in [PR77](https://github.com/CrispStrobe/codal-samd/pull/77).

The reproducible [report](../profiles/pxt-fullboard-source-census.json) binds
Codal38257952, core2ed19d64, framework31abf23d and engine79071f69. The owning
platform's header graph still reaches device headers and SPI/USART/I2C HAL/HPL;
its CMake build still selects ASF4. These are exclusions and replacement tasks,
not inputs admitted to an ASF4-free simulation build.

Core defaults are `DEVICE_USB=0`, `DEVICE_WEBUSB=1`; the qualified service
profile explicitly selects0/0. USB1 is conditional until an actual generated
configuration and dependency closure are frozen. For that profile, USB requires
clock/reset, pin mux, calibration, descriptor-buffer, endpoint and interrupt
semantics. The PyBadge descriptor's USB stub does not qualify them.

No QSPI token occurs in the64 retained framework files. That bounded absence
neither proves absence in other sources nor turns the descriptor's QSPI stub
into a runtime requirement. NVMCTRL/CMCC native model sources exist at the
selected engine, although this PyBadge descriptor omits their declarations;
storage fixtures explicitly select them. Application vectors at0x4000 also
remain separate from the service fixture's vectors at0.

Next implementation must first freeze an actual generated board configuration,
then select a permitted replacement header/source closure and independently
review its obligations. Preserve existing caller behavior and notices. Qualify
every affected peripheral and the application startup/vector layout in a clean
guest before making full-board claims. The historical storage census is kept
byte-for-byte unchanged; this source report needs no ARM build or guest replay.


## Source-only checkpoint — 2026-10-23

Merged as `9d4c467f7dc6e194f23909746fd23fad3f563a97` after both enabled source/docs checks passed.
The committed report reproduces from the exact public source identities and
four source/configuration refusal controls pass. No compiler or guest was run
for this inventory. It leaves [the generated configuration and replacement
closure](PXT-STORAGE-NEXT-STEPS.md) open; historical reports remain unchanged.
