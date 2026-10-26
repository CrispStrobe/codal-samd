# Native TCC system-timer selection

The original Arcade 4.2.1 request is retained byte-for-byte. Its SAMD
`allocateTimer()` resolves a timer id to a table index, then incorrectly uses
the index to choose TC versus TCC. The owned compile-copy adaptation changes
only `if (idx < 0x20)` to `if (tcId < 0x20)` in the exact 4,401-byte source
(SHA-256 `b813dfdeacd5e528e38a5d7286bb60c31c1b373172b2449bfbe1d047431d78fc`).
The adapted source SHA-256 is
`1f84c27ee09b3d735d2bfa3069bd4a11234cdfb940e55b01e256ca98a972aef1`.
The base/serial builds record both identities and the selected adaptation;
`--timer-selection original-index` preserves the omission control.

The default CF2 metadata is unchanged. An explicitly authored native case adds
only key218 (`CFG_TIMERS_TO_USE`) with value `0x20111200`: TCC0, TC1, TC2.
The table's used count increases from62 to63; original entries, information and
reset-forward/application bytes retain their admitted identities. Both the
patched and original source bases compile a genuine empty native MakeCode
program, then use that same declared configuration.

Before merge, require native boot, natural TCC0 compare interrupts (no TC0
fallback), USB enumeration and HF2 BININFO in both scheduler profiles. The
original-index source must reach the real constructor panic90 before USB starts.
The TCC interrupt-mask control must retain a pending match with no handler or
USB start. Existing TC0 and all earlier native cases remain required. A timeout,
unrelated panic or generated source mismatch is failure. Preserve originals
once and audit them with a frozen standalone reader; do not infer success from
the earlier TCC leaf, which has no MakeCode runtime. Current source is a
candidate until those actual qualifications and every enabled exact-head check
pass. No upstream posting, hardware timing or new capture acknowledgement is
part of this change.

The first hosted candidate failed its observer assertions: it watched MC0, but
the pinned CODAL `Timer` constructor defaults period channel 0 and event channel
1. The corrected observer requires `TCC0_2_Handler` (MC1/IRQ87) and pending bit
23 in ISPR2 in the masked control. The MC0 TCC leaf remains separate and
unchanged. The original failed campaign is retained; the 70-million-instruction
control window and natural USB/HF2 requirements are unchanged.
