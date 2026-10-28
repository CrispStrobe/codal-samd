# RAFFS v2 payload integrity

Approved by the owner on 2026-10-28. The owned fullboard build adapts the exact
admitted RAFFS.cpp SHA through `REQUEST_FIXES`; upstream request bytes remain
unchanged. `--request-fixes none` builds the legacy omission control.

V2 changes bank magic from `0x7776e0da` to `0x7776e0db`. The 16-byte header and
8-byte metadata entries retain their layout. Every non-deleted payload is
followed by a four-byte little-endian CRC32/ISO-HDLC. The CRC covers the NUL-ended
key name, a four-byte little-endian payload length, and the payload. It detects
accidental corruption; it does not authenticate data. Metadata size still names
only the payload. Tombstones retain their original representation.

Before lookup, compaction or directory iteration follows an entry, the validator
checks name/payload/CRC bounds and validates v2 CRCs. An invalid newest entry is
skipped so an older valid value can be used. Cached reads are validated too.
Legacy banks are readable without requiring a CRC. The first write migrates
through the existing two-bank compaction: valid live entries and their CRCs are
copied, metadata is flushed, and the new bank header is the final commit. The
old bank remains available until that point. Migration preflights space before
erasing the alternate bank; if CRC overhead cannot fit, writing fails while the
old bank is preserved. Original firmware cannot interpret a v2 bank.

`test_raffs_checksum.py` compiles the actual CRC/entry-validator implementation
and compares CRCs with independent Python zlib vectors. It checks payload, CRC,
key and length corruption, erased data, bounds, and legacy entries. The byte-bound complete legacy and adapted filesystems also run against an
explicit host flash model: legacy reads preserve bytes, every migration commit
cut recovers an old or new value, and 122 legacy keys remain intact when v2 CRC
overhead cannot fit. This supplements the native board path.

Native `store-data-lost` and `store-data-bitflip` must recover the previous value;
`store-data-lost-original` retains the unadapted failure. `store-legacy` creates
its bank using the actual unadapted guest, migrates it, and reboots from every
committed-operation cut. Existing delete/compaction/interruption cases remain
required. Both scheduler profiles and hosted fullboard qualification are pending.
