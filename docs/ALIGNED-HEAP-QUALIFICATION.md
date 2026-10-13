# Prospective aligned public allocation over the actual CODAL heap

The original source-bound guest diagnostic reproduced the raw heap's missing
eight-byte alignment in both engine profiles. Runtime source
`92e99cc881db3e5d8f0186821d8c83bc241eb3b2`, engine source
`dd83ffda58e51a175de89cbed4dee2d66a0b2c66`, and
[actual guest run](https://github.com/CrispStrobe/labwired-core/actions/runs/38485055870)
bind that original result. Two retained eight-byte allocations were twelve bytes
apart; one was four modulo eight. The expected refusal was `0xe0000c60`.
The ten retained service/clock cases also passed. This is a reproduced allocator
deficiency, not native Arcade or allocator admission.

The candidate public adapter reserves eight bytes of raw-pointer/requested-size
metadata and up to seven bytes of alignment padding from actual `device_malloc`.
Public malloc/free/realloc and C++ allocation use that domain. Public realloc
copies only the smaller requested payload and preserves the old allocation on
arithmetic/NULL refusal. Zero/null behavior is explicit; calloc refuses product
overflow before allocating. Fundamental eight-byte alignment does not support
arbitrary over-alignment or authenticate invalid caller pointers.

`heap-domains.patch` preserves credited core source and makes raw
`device_realloc` call raw allocation/free, with its header excluded from copy
capacity. Its raw capacity remains word-rounded, not the original requested
length; raw zero-size realloc still returns NULL without freeing the old block.
Do not interchange raw and public pointer domains or claim universal libc
semantics. The configured fatal heap-exhaustion policy remains enabled.

Eight separately built modes retain the existing service/clock cases, an
explicit raw-domain alignment diagnostic, and two new callers:

- Mode 6: simultaneous odd-sized allocations and churn, overflow/null/zero,
  realloc growth/shrink/content preservation and arithmetic refusal, calloc,
  actual volatile 64-bit C++ objects/arrays, and raw-domain realloc/free.
- Mode 7: actual heap exhaustion during public realloc, with the prior payload
  retained for independent observation after the declared fatal OOM result.

Build provenance must record the additional adapter, actual heap patch and all
selected compiler dependencies/notices. Both hosted engine profiles must
execute these source-bound callers. Compilation alone is not qualification.
Native SPI, board/CF2/image licensing, actual Arcade frames/input/debugger,
representative performance and installed consumer adoption remain pending.
