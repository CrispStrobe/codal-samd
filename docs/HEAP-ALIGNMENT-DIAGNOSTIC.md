# Real CODAL heap alignment diagnostic

This source-only caller is not runtime admission. Mode 5 retains two separate
eight-byte allocations from the actual selected CODAL heap, records their
addresses, and checks eight-byte alignment before constructing any Timer or
fiber service. It retains the existing real-heap free/reuse control first.

The selected `codal-core` source is
`2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`. Its allocator uses four-byte
words and a one-word allocation header. Two fresh eight-byte allocations
therefore have a twelve-byte stride; source review predicts that at least one
address will not meet the eight-byte alignment required by 64-bit objects.
That prediction is not an observed guest result.

The hosted build creates mode 5 in a separate clean core checkout. Its
`CodalHeapAllocator.cpp` is unchanged and recorded against the original pin.
Existing fiber and Timer integration patches remain explicit and do not change
allocation. The original build and subsequent bounded execution in both engine
profiles must be retained before implementing an aligned public adapter.

Expected diagnostic refusal is `0xc60`, with stage 1, two distinct nonzero
addresses and payload size 8. Unexpected allocation refusal is `0xc61`.
The engine must observe actual BSS witnesses, startup, source/image provenance,
architectural mask, and absence of interrupts/context switches. A diagnostic
that reproduces the alignment deficiency is not an allocator success claim.

Next implementation: keep raw `device_*` allocation APIs separate, place an
eight-byte-aligned public adapter over the real heap, record requested payload
size, preserve old allocation on realloc failure, and qualify odd sizes/churn,
content preservation, overflow/null/zero behavior and actual 64-bit accesses.
Raw `device_realloc` currently calls public allocation functions and includes
the header in its copy bound; both need explicit review before changing public
allocation. Do not claim arbitrary over-alignment or complete libc semantics.

Native Arcade build, component notices, API qualification, actual frames/input,
debugger and consumer admission remain separate pending gates.
