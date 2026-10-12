# Actual Timer caller qualification — 2026-10-12

[PR13](https://github.com/CrispStrobe/codal-samd/pull/13) landed reviewed source
`d55214588780d737640536b0a58707cd6d69cfce` as
`2ab8dcc380d12468e5464b9edd5c24878648683e`. Reviewed, build-tested, predicted
and landed trees equal `046b6ccb0a2e8f217258c332bb9a0eb45951eb4d`.
The merge parents are base `a489c9f679383455e6470ebc109b93af1dcc5084` and
that reviewed head. Its enabled ARM compiler/link/provenance gate passed in
[run38163094316](https://github.com/CrispStrobe/codal-samd/actions/runs/38163094316).
The earlier repeated-header provenance failure in
[run38159532657](https://github.com/CrispStrobe/codal-samd/actions/runs/38159532657)
is preserved; the reader now validates repeated occurrences before canonicalizing.

## Actual guest evidence

The complete unchanged upstream `Timer.cpp` and actual headers came from
immutable core candidate `2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`.
[Positive run38169003939](https://github.com/CrispStrobe/labwired-core/actions/runs/38169003939)
passed both scheduler profiles: 45,344 instructions each, six ordered caller
milestones, two actual IRQ entries/returns, handler mask0/exception16 with closed
paired state, fault0 and final masked-thread status `0x600d`.
The [unchanged positive receipt](https://github.com/CrispStrobe/labwired-core/blob/main/docs/receipts/2026-10-11-samd-real-timer/README.md)
retains original source/image/log/run identities and separate packet/source audit.

[Negative run38198468505](https://github.com/CrispStrobe/labwired-core/actions/runs/38198468505)
passed three authored adapter derivatives under both scheduler profiles:

| Adapter mutant | Exact status | Exception | Steps each profile | Terminal instructions |
| --- | --- | --- | --- | --- |
| Unconditional IRQ restore | `0xb02` | 0 | 24384 | 64 |
| Early IRQ restore | `0xa14` | 16 | 26301 | 64 |
| Unsupported Event entry | `0xa07` | 0 | 1331 | 64 |

All stayed masked with fault0 and exact case-specific witnesses. Each had 64
consecutive instructions inside its actual rejection function with changing PC,
within the 250,000-instruction bound. Wrong status, context, timeout, escape,
stopped CPU or engine failure did not qualify rejection.
The [unchanged negative receipt](https://github.com/CrispStrobe/labwired-core/blob/2ed408758487b7e76c52489dcedaebef1a87033b/docs/receipts/2026-10-12-samd-real-timer-refusals/README.md)
records separate audit of immutable original/helper sources, exact derivatives,
dependency closures, consumed objects/ELF/maps/logs and official run/merge identity.
Recorded compiler component snapshots are limited, not whole-toolchain closure.

## What this does not qualify

This opt-in fixture has an authored advancing counter, one bounded event-list
allocation, counted component construction/destruction and explicitly rejected
event dispatch. It is not peripheral timer MMIO, a real allocator/component
registry/event bus, fiber switching, USB/WebUSB, full application startup or a
complete board. The diagnostic profile does not replace required application
services. Immutable dependency attribution remains preserved; header/compiler
components and exact distributed-build obligations still require review.

Default runtime CMake and historical dependency pins did not change. This is
not an ASF-free runtime, full core/board ABI qualification, clean-room origin,
legal/store clearance, physical safety, native Arcade boot or new RTx result.
The original `timer-caller/README.md` retains its prospective source plan;
the dated observed results here supersede its unexecuted-status wording.

Next: final assembled [LabWired PR200](https://github.com/CrispStrobe/labwired-core/pull/200)
qualification and all enabled final-head checks, then actual clock/component/
allocator/event/fiber services, a named no-ASF build, required services, boot,
consumer/debugger/package adoption and active-workload performance separately.
