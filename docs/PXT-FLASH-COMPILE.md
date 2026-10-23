# Real PXT flash driver: include-adapted compilation checkpoint

This qualified compile-only profile selects the real common-packages14.1.4
`libs/settings/SAMDFlash.cpp` and unchanged `Flash.h`, using the immutable
[source bindings](PXT-STORAGE-SOURCE-BINDINGS.md). The hosted compiler and
independent receipt checks passed within the boundary below. This is not the
full PXT shared-header or RAFFS/settings build.

## Reached hosted checkpoint — 2026-10-15

[Actual run38744261811](https://github.com/CrispStrobe/codal-samd/actions/runs/38744261811),
job116453635778, passed at reviewed source
`2e80170cce92fef13d69608b6d1b1d293474da3d`. Its tested merge was
`dd7fc5b77ab7a287c474fe2ac56975942ef69e47`, tree
`60c4aa64e85e8d849d9b9b97431e6ab3517eec97`, with ordered parents
`8935cc0ab29c44584c3417ab7837fb5caa325ba3` and that reviewed source.

GCC ARM13.2.1 compiled the include-adapted driver with all four real `ZFlash`
methods and unresolved runtime/libc dependencies. Both specific compiler
refusals passed. All 16 selected dependencies match their reviewed source
bytes. The original [compile report](receipts/2026-10-15-pxt-flash-compile.json)
is preserved unchanged: 4973 bytes, SHA256
`d0bd0b2fb3aeb391f9425cb9efbd2a3fa1d5263a0e2d3004acc8bf45fd33b7ab`.
The producer-recorded object SHA256 is
`33e0db9ab50bc424702f9629072f0f57f7a97d688402b8d0b0871d7ad59851a3`;
objects were deliberately not uploaded, so the receiver did not rehash a binary.

A separate stdlib receiver, frozen before opening the official artifact, passed
one synthetic positive and 41 refusal controls. It admitted the successful
run/job/source/tested-commit/artifact metadata, checked the sole acquired
[official artifact11980034457](https://api.github.com/repos/CrispStrobe/codal-samd/actions/artifacts/11980034457/zip)
(55026 bytes, SHA256
`6934df1e8e3ecb1fa8974c13272a7ebf0eefdbe2e2a51fba091654126d8e0900`),
then checked source/adaptation/dependency bytes, notices, refusal diagnostics
and narrow report flags. It imported no producer code and did not rerun the
compiler or a guest. This is receipt/source verification, not an independent
source-to-binary rebuild, ABI execution test or licence clearance.

[PR42](https://github.com/CrispStrobe/codal-samd/pull/42) landed as
`2cd3bfe88c72c70e0206bda4f8d924b4a2a52073`, tree
`4b69e2f14bcdc0c212c4bd1e650f2664e5def302`, after both enabled exact-head
checks succeeded. Ordered parents are
`afd5ba6954ac77f9c57ee2ee08fbd4bb6bbc5a6d` and reviewed source
`2e80170cce92fef13d69608b6d1b1d293474da3d`. The independently predicted
integration tree equals the landed tree. Compared with the tested source tree,
only the four previously qualified NVM results documentation/receipt paths
changed; executable inputs are unchanged. Global tested/landed tree equality
is **not** claimed.

## Explicit adaptation, not a PXT header substitute

The [hosted builder](../pxt-storage-compile/compile_flash.py) verifies the original
public Git bytes, then changes exactly one include line in a separately named
derived source: `pxt.h` becomes `pxt-flash-platform.h`. The rest of the driver
body is unchanged. Both original and adapted bytes and hashes are retained;
the report identifies this as an include-adapted driver, never unchanged PXT.

The new platform header supplies only the actual CODAL target declarations,
no-op disabled logging macro, ARM memory barrier and authored narrow NVMCTRL/CMCC
register views needed by this driver. It supplies no fake `pxt.h`, Flash class,
storage implementation, IRQ function, libc implementation or successful device
response. The real `Flash.h` includes actual immutable CODAL headers.

Register addresses, offsets, widths, fields and command values were reviewed
against Microchip's [SAM D5x/E5x family datasheet DS60001507F](https://ww1.microchip.com/downloads/en/DeviceDoc/SAM_D5xE5x_Family_Data_Sheet_DS60001507F.pdf),
chapters 7, 11 and 25. No ASF, vendor device header or hardware-restricted source
is selected or copied. Authored declarations are MIT; this is not a claim of
independent clean-room implementation or whole-image licence clearance.
Compile-time checks cover register structure offsets and widths. Bitfield ABI,
bus accesses, cache behaviour, flash commands and persistence still require
actual affected guest qualification; layout assertions alone do not prove them.

## Compile and admission boundary

The entry requires hosted CI and an explicit SAMD51 compile selector. It uses
the separate service-only platform declarations, including disabled USB/WebUSB;
**this is not the original application's board configuration** and cannot close
that build requirement. No default firmware selection or existing pin changes.

Implicit compiler include paths are disabled. Every selected dependency must
come from the closed own-header set, the verified framework inputs or immutable
CODAL core `2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`. Bytes must match their
Git source; unknown/compiler/vendor headers refuse. The original unadapted
driver must specifically fail for missing `pxt.h`, demonstrating that no PXT
stub slipped into this profile. Omitting the profile selector must also refuse
with its named diagnostic.

The positive compile must define all four real `ZFlash` methods and retain
unresolved panic, IRQ, memcpy and memset dependencies. It does not link or run.
CI preserves compiler identity, original/adapted driver, selected headers,
dependency hashes, diagnostics and component/GCC-exception notices, not objects
or a firmware binary. A failed compiler run is not accepted as positive evidence.

Pure controls, without any compiler:

```sh
python3 -B -m unittest discover -s pxt-storage-compile -p 'test_*.py' -v
```

The bounded hosted compilation and original receipt audit are complete.
Next close real PXT headers/RAFFS/settings and board configuration; implement
NVMCTRL/CMCC behaviour with fresh guest tests before claiming storage execution.
No native Arcade, display, debugger, physical capture or RTx claim follows here.


## Current continuation — 2026-10-23

This compile receipt stays unchanged. Subsequent actual selected
[ZFlash adoption](PXT-FIXED-ENGINE-ADOPTION.md),
[RAFFS/settings](PXT-RAFFS-SETTINGS-RUNTIME.md),
[compaction](PXT-STORAGE-COMPACTION.md) and
[modeled reset](PXT-STORAGE-RESET.md) have separate finite qualification.
The [remaining scopes](PXT-STORAGE-NEXT-STEPS.md) require actual generated-board
configuration/header replacement and affected guests; do not repeat the older
prospective selected-storage gates as if they were still open.
