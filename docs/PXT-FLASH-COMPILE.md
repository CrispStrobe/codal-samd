# Real PXT flash driver: include-adapted compilation checkpoint

This prospective compile-only profile selects the real common-packages14.1.4
`libs/settings/SAMDFlash.cpp` and unchanged `Flash.h`, using the immutable
[source bindings](PXT-STORAGE-SOURCE-BINDINGS.md). **No hosted compiler result
is claimed yet.** This is not the full PXT shared-header or RAFFS/settings build.

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

After actual hosted compilation, audit the original receipt independently.
Then close real PXT headers/RAFFS/settings and board configuration; implement
NVMCTRL/CMCC behaviour with fresh guest tests before claiming storage execution.
No native Arcade, display, debugger, physical capture or RTx claim follows here.
