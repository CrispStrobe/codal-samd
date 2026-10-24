# Authored CF2 and UF2 information metadata — source contract

Owned scope: `no-asf-board/cf2/`, `profiles/pxt-cf2-authored-metadata.json`,
`.github/workflows/pxt-cf2-metadata.yml`, and this contract. This source-only
profile admits public configuration literals and an authored data layout.
It does not select a bootloader, compile an ARM application, execute runtime
configuration reads, move an engine pin or qualify native Arcade/HF2 startup.

Read the exact public PyBadge configuration in `adafruit/uf2-samdx1` commit
`d4dc92889759c0c551683420e24c5ef535ac303e`,
`boards/arcade_pybadge/board_config.h`. Admit its Git blob, raw SHA-256 and
literal-only CF2 array. Preserve all 62 used and 100 total entries, unique keys,
exact values and zero reserve/terminator. Retain the original repository licence
notice, including third-party distinctions. Only the CF2 numeric array is
selected; no ASF, boot USART setup, bootloader implementation or binary is used.

Author three non-overlapping data fragments inside the reserved region below
`0x4000`: the exact CF2 table at `0x3000`, an explicit authored informational
string at `0x3400`, and the 16-byte UF2 information record at `0x3ff0`. Its first
word points to CF2; both handover function pointers are zero; its final word
points to the authored string. No vendor firmware identity or handover function
is invented. Preserve the reset forwarding words at zero and application origin.

Strict controls must reject source drift, nonliteral expressions, bad magic or
counts, duplicate/zero used keys, missing zero reserve, overlap, bad pointer
alignment, an out-of-region or unterminated string, and executable handover
pointers. Generated fragments remain unqualified data until an explicit native
consumer includes them and actual runtime/PXT USB reads are observed in both
scheduler profiles with independent original-evidence audit. Later integration
must bind `getBootloaderConfigData()`, `UF2_BInfo` and `uf2_info()` to the selected
framework sources; a host decode cannot substitute for those actual calls.
