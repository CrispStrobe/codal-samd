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
must pass hosted CI before merge. Qualification is pending.
