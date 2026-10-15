# Actual core-local reset caller — 2026-10-15

[Runtime PR36](https://github.com/CrispStrobe/codal-samd/pull/36) landed at
`3a93f1e786bda73a56d92acdae2cd82cacdf5d8a`, with both enabled checks successful
and reviewed/tested/landed tree `08b208e1fa681a95457cab9f41029059599f4c2e`.
Its mode19 caller passed actual engine qualification in
[Core PR242](https://github.com/CrispStrobe/labwired-core/pull/242), now landed as
`73f8475f8e406b1ff4475bc9886e22479881d67a` after all 27 enabled final-head
checks passed, including the actual reset guest. Final reviewed source
`7da58a8d43347e490209710267f95aca3bdda087` changes only approved acknowledgement
bindings and documentation relative to the audited source below. The original
acknowledgement age, November 12 expiry and physical captures are preserved;
physical re-capture remains owed. No application adoption is claimed.

The landed tree `cf8251039ffe7fbaa07179df014e2cfc82447e23` differs from the
reviewed merge tree only by another worker's concurrent lane-documentation
addition, not executable/workflow inputs. See the
[engine's exact integration record](https://github.com/CrispStrobe/labwired-core/blob/main/docs/engineering/samd-real-core-reset-qualification.md).

## Exact source and retained-original evidence

- Runtime source: `12d4759af0bf2b29726cfd79c487474a282cc5b0`; immutable CODAL core
  `2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`; explicit
  `BW_SIMULATION_CORE_RESET=1`. Old modes0–18 retain their existing selection.
- [Separate twenty-mode build38706839486](https://github.com/CrispStrobe/codal-samd/actions/runs/38706839486),
  job116328226468, passes. Independent retained-original comparison establishes
  byte-identical old0–18 ELFs. New reset ELF SHA256:
  `935b8a22fe65faafeb50ae3bfac92dd046fc415f55c570727d46673284406ce9`.
- Engine/test source: `7478a36e118f3fad87e2d0b3f1415a96a7f4d7e3`, source tree
  `af5b7659b0e746770d678110d6574bab9f8f3acb`. Actual tested merge:
  `629e06e14108a277a956928a78f76b881a6ae2f1`, tree
  `686e059ec762ade5de81a637b8503f4c25a05f7c`, parents
  `3f54e1ee7ba9b963d0987437e8a8d447c6b48838` and that source head.
  The PR event base remains `5ca516501e6effb167eb7e25fad8e6736eaf0f68`.
  The [source-to-tested comparison](https://github.com/CrispStrobe/labwired-core/compare/7478a36e118f3fad87e2d0b3f1415a96a7f4d7e3...629e06e14108a277a956928a78f76b881a6ae2f1)
  contains only a concurrent added performance Markdown file, no executable or
  workflow difference. Do not conflate source tree and tested merge tree.
- [Actual run38709086550](https://github.com/CrispStrobe/labwired-core/actions/runs/38709086550),
  job116335717016, passes all42 cases: four reset cases and38 earlier callers.
  Sole retained original log:989472 bytes, SHA256
  `f41724ad23a002be7aa332f51144e07383124b4c47174a8cef58a0295778bbf6`.
  Official run/job/attempt/steps/source/tree/parents were admitted before its
  single acquisition; no replay or repeat download was used to inspect it.
- The separately implemented reader was frozen before revised-original
  acquisition, SHA256
  `8f74e38fd20da774a1c5b3c6b4d19594da5196e0be89b51f27bdf952a1bfac10`.
  It retains hash-bound semantic predicates and the strict BSS integer guard,
  plus an explicit metadata-only binding for the concurrent Markdown merge.
  Synthetic refusal controls preceded acquisition. This is a same-workstream
  audit without producer-helper imports, not an independent human review or a
  claim of clean-room origin.

## Observed boundary

Both scheduler profiles pass quantum1 and quantum64 with1720 real instructions
per case, terminal1656 plus64 further instructions, two startup epochs and314
raw event/RAM rows. Each quantum64 case uses28 CPU batches rather than1720.
This establishes actual batching, not a measured throughput or RTx speedup.

The audit reconstructs initialized data/BSS and retained NOLOAD across both
epochs; verifies invalid-key return versus valid AIRCR store/reset cut, startup
register captures, poisoned core-state readbacks, no old-epoch return and zero
CFSR/HFSR. Software `MRS xPSR` reads EPSR fields as zero; the separate full
debugger xPSR check retains the Thumb bit. Opt-in wide-RAM-store observation
reports real committed bytes and remains off by default.

The new reset-only fixture explicitly excludes external board devices. The
full board's button DATA-driven resident still requires per-cycle service;
earlier caller fixtures retain their unchanged board setup. This is not
fully wired PyBadge batching or peripheral-reset qualification. Preserve the
[first failed run38698483581](https://github.com/CrispStrobe/labwired-core/actions/runs/38698483581)
and its incorrect first-boot MRS expectation; it never reached a reset request.

Peripheral/POR/flash persistence, NVM/serial and actual consumer compatibility,
no-ASF whole-image admission, native Arcade frames/input/debugger behavior,
installed packages, active RTx and component-specific licence closure remain
separate. No application pin, physical capture or performance threshold changed.
