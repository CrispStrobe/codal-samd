# Prospective public request-size diagnostics

This separate investigation follows the qualified
[public fatal-exhaustion checkpoint](PXT-PUBLIC-GC-REAL-OOM.md) and addresses
item A5 of the [failure/lifetime lane](PXT-GC-FAILURE-LIFETIME-LANES.md).
The new callers and observer have no actual hosted guest result yet. All
outcomes below are selected-source predictions to test, not qualified behavior
or a claim that oversized requests are correctly refused.

## Separate callers and source predictions

The builder explicitly selects `public-size-cap` or `public-size-conversion`.
Each uses the existing reset, static constructors and unchanged held dependency
pins. No allocator implementation, cap, panic policy or previous caller changes.
Each first checks public zero allocation, retains a 32-byte permanent allocation,
then allocates, fills and frees 17-byte, 33-byte and 131068-byte arrays. The
observer checks all live payload bytes and actual ownership headers at each
boundary and the surviving payload after actual collection returns and terminal
publication. The three temporary frees must traverse the real public and
`app_free` entries and complete their actual returns before terminal admission.

The selected cap in `pxtcore.h` is 131072 bytes. After alignment and the four-byte
array header, a 131068-byte payload should exactly fit. The cap caller then
requests 131069 bytes, which should reach `gcAllocate(131076)` and actual
`target_panic(22)`, followed by status `0xe0000016`. Neither aggregate OOM nor
normal return qualifies that cap observation.

The conversion caller instead finally requests the selected 32-bit `SIZE_MAX`,
4294967295. The retained public signature accepts `size_t`; `app_alloc` and
`gcAllocateArray` accept `int`. Selected source tests only zero before alignment
and header addition. The diagnostic predicts actual arguments `0xffffffff` at
the first three chain entries and 4 at `gcAllocate`, followed by a nonnull public
return whose actual permanent array header is `0x60000004`. This prediction is
specific to the held source, compiler and target. No requested payload range is
written or read for this final request. A diagnostic normal terminal records the
returned pointer; it does not declare that request satisfied or correctly refused.
An unexpected original result must be preserved and investigated.

## Qualification requirements

Both cases must execute their own source-bound ELF in both scheduler profiles,
within 12000000 steps. Require real startup/data/BSS/constructors, thread context,
all allocator chain entries, LR returns, exact arguments, ownership/capacity
headers, real frees, surviving bytes and fault registers. A raw allocation
failure, unrelated panic, compile/setup failure or exhausted bound is not a pass.

The cap's isolated source mutant changes only its unique too-big panic reason
from 22 to 21 and must expose actual reason 21 at the final cap request. The
conversion case's isolated mutant removes permanent ownership and must expose
the first 32-byte allocation's actual header `0x40000024` instead of
`0x60000024`. That ownership mutant stops before the final oversized request.
Both mutants must fail the named observation in both profiles; manufactured host
matcher inputs alone do not qualify them.

Before merge, require every enabled exact-head check, affected prior guest
campaigns, frozen independent readers and ONCE acquisition/audit of original
source, dependencies, commands, checkout, images and raw streams. Preserve every
failure and verify tested/reviewed/predicted/landed trees and ordered parents.

## Remaining work

This is a finite diagnostic, not a policy fix or exhaustive coverage of public
integer boundaries. Any resulting refusal change needs its own reviewed source
scope and fresh affected qualification. Allocated DMA/TXC/CS/callback lifetime
follows closure of A5. Full board/services/storage, generated native Arcade,
representative RTx and installed Lite adoption remain later work.
