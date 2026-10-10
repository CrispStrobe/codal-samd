# Critical-section source census, not runtime admission

This tool makes the [pinned interface/caller review](https://github.com/CrispStrobe/labwired-core/blob/548fc9b1c83486fe35885598aaee17d5ecec1675/docs/engineering/samd-critical-section-interface.md)
reproducible against the already retained original dependency evidence from
[run 37794963356](https://github.com/CrispStrobe/labwired-core/actions/runs/37794963356).
It binds 15 reviewed dependency source/header hashes, three immutable public
repository pins, the exact diagnostic report and generated configuration.
Output includes each selected file's diagnostic consumer paths, not host paths
or original compiler commands. It performs no acquisition, extraction,
compilation, relinking, disassembly or guest execution.

Run `python3 -B tools/critical-section-census/census.py ORIGINAL_CAPTURE.zip`
against your retained original ZIP. Input member names are the original
`diagnostic-dependencies.json` and
`generated/build/codal_extra_definitions.h`; changed bytes, missing/duplicate
members, duplicate JSON keys, mismatched source/unit/configuration bindings
and unexpected source paths must refuse with exit status 2. No automatic
download or recovery substitutes a different capture. Review output before
publication; original input receipts remain unchanged.

The result is only `SOURCE_CENSUS_ONLY`, with runtime admission, runtime build
selection and active-call proof all false. The report contains separate `-M`
probes, not recovered original compiler execution or retained-inline proof.
The tool does not scan current repositories, expand macros, inspect firmware,
verify every report field or independently establish its source origins. It
joins the exact original report to reviewed public source hashes; the pinned
public source review remains a separate prerequisite. No source spelling count
is treated as an executed-call count or legal allowlist.

Unresolved work is emitted explicitly: effective preprocessor paths;
request/generated/assembly callers; selected component notices; retained and
executed callers; simulation build selection; and actual integrated caller
guests. This bounded dependency census excludes the seven known request caller
inputs and other unidentified dependencies. It is not complete R1a.

Hosted CI runs synthetic positive/refusal controls only, also under Python
optimization. It does not reacquire or replay the actual capture, run an ARM
compiler or change the pinned foundation fixtures. Existing inherited runtime
CMake/ASF selection, stacked prototype status and image rejection policy are
unchanged. Follow the [runtime replacement plan](https://github.com/CrispStrobe/labwired-core/blob/main/docs/engineering/samd-runtime-replacement-plan.md)
before choosing the paired ABI adapter and real simulation build.
