# Actual SPI fail-stop destruction and idle reconstruction — 2026-10-15

This bounded checkpoint landed through
[Runtime PR34](https://github.com/CrispStrobe/codal-samd/pull/34), merge
`580931ac258ffa4b1949165247418b4e2eeb4b99`, and
[engine PR239](https://github.com/CrispStrobe/labwired-core/pull/239), merge
`6cc18e36497423993e0224114f1bc759160d3631`.
Both Runtime checks and all 23 enabled engine checks passed; only the seven
declared optional engine jobs skipped. Reviewed, actual-tested and landed trees,
merge parents and default-branch refs matched. No driver/model repair or
consumer pin changed.

It qualifies deliberate fail-stop refusal of invalid destruction and successful
settled-idle reconstruction, not safe active cancellation-and-delete, arbitrary
concurrency, native Arcade boot, active RTx or whole-image licence admission.

## Exact source and evidence

- Runtime caller source: `932d476344107386d56dfd2d595d74d713780563`,
  [PR34](https://github.com/CrispStrobe/codal-samd/pull/34), reviewed tree
  `9b70f1cfa3af5ee34b1014c102071975c817b5f1`.
- Engine/test source: `e3d8c8b0149763f0bfed4d6cfde0a54b221257ed`,
  [PR239](https://github.com/CrispStrobe/labwired-core/pull/239), reviewed/tested
  tree `33238506e2050f54eb6d34df6725cbc52de68cec`, tested merge
  `9da8f81c16f27824a4732a8c5027b2a89b660de8` on final base
  `395c90ab7684bb07aab766d6aeb7975e2d7c6ac9`.
- [Separate nineteen-mode build38683267460](https://github.com/CrispStrobe/codal-samd/actions/runs/38683267460),
  job116252323637, passes. All fifteen previously qualified caller ELF hashes
  remain unchanged. New SPI callers retain49 selected inputs and14 compiler
  dependency rules, with separate clean immutable core checkouts. Driver/model
  and previous callers are unchanged.
- [Actual ten-case run38688077422](https://github.com/CrispStrobe/labwired-core/actions/runs/38688077422),
  job116267435110, passes all four new callers and unchanged positive8 in both
  default and event-scheduler profiles. The retained sole original log response
  is470983 bytes, SHA256
  `b9bd3b211fe54fb87075b894c2c008be9440f269e4ef0dfa42ad1cccd7a94737`.
  Official source/run/job/attempt/step/tested-tree/parent identity was admitted
  before acquisition. No guest replay or repeat download to inspect evidence.
- The standalone independent original reader was frozen before acquisition:
  predicate SHA256
  `ef4ae92b44a12427830c4286ecb2f7639ad150ea82751b26db3bf1518e16447a`,
  exact-source binding
  `ee75a09d1563e6029dc4c84e131afe8c6de08f6b61128e1f321f66b32b422429`.
  Ten manufactured valid case controls reject512 alterations; two plain/official-
  timestamp envelope controls reject50 alterations; one provider control rejects23.
  All ten actual cases pass without changing the frozen rules. No producer
  predicate import. The earlier source also had a separate source-aware read-only
  crosscheck of its own original; this is not independent implementation or
  clean-room evidence.

## Observed cases

| Caller | Both profiles |
| --- | --- |
|15: active asynchronous token |61940 steps,10 raw rows,0 context swaps;2 bytes accepted,0 wire bytes; deliberate `0xc42`, no destructor return or free. |
|16: inside the delivering success callback |83457 steps,21 rows,1 swap;4 accepted/4 wire bytes; DMA IRQ enters/returns once, SPI IRQ79 enters once and deliberately does not return after the fail-stop panic. |
|17: unconsumed blocking terminal result |83802 steps,23 rows,2 swaps;2 accepted/0 wire, writeback30 of32; actual owner enrollment, cancellation NotifyOne and cancel return before destruction refusal; original owner has not resumed. |
|18: settled idle deletion and reconstruction |203670 steps,84 rows,12 swaps;16 accepted/16 wire, two asynchronous callbacks and two blocking returns; four DMA and four SPI entries/returns match. |
| Unchanged positive8 |126844 steps,28 rows,6 swaps; two DMA and two SPI entries/returns match. |

These are guest instruction/observation counts, not time or RTx measurements.
All cases retain source-bound vector, initialized-data copy, BSS clearance and
fault-register checks. CFSR/HFSR are zero. Each refusal binds the actual panic
PC and `r0=0xc42`;32 additional actual steps follow that entry and establish the
expected panic status. The idle case observes32 steps after actual `0x600d`.

This is a deliberate fail-stop policy for invalid destruction, not a recoverable
BUSY API, cancellation-and-deletion, or permission to keep using the old object.
In-flight destruction never returns or frees the object within the observed
window. Masked interrupts do not freeze DMA: mode15's remaining count is a
pre-delete observation. Observed zero wire progress is not a universal promise.

Mode16 runs on an actual callback fiber distinct from main; its IRQ79 remains
active because panic is deliberate. Mode17 observes owner enrollment source1023/
value1024 and actual NotifyOne source1022/same value/mode1 on a distinct masked
canceller. Raw channel-disable bytes are `00 0d 20 00`; CS release and notification
precede cancel return and the virtual deletion attempt. No original-owner or
canceller function return is observed through the post-panic window.

Mode18 follows actual virtual delete through the deleting and complete/base
destructors, successful MessageBus remove, actual public free and their returns,
preserving incoming PRIMASK1 on the same live main fiber. Both relevant NVIC
enable words read `0x80000000` before the destructor and0 after; these are real
readbacks, not fabricated SCS write observations. SPI/DMAC interrupt-clear stores
remain raw attributed observations. MessageBus marks a listener for deletion;
its successful remove return is not synchronous node-free proof.

The fresh constructor returns against the same live Timer/MessageBus, then new
asynchronous and blocking work succeeds. The allocator actually reused the old
object address in this run. Logical reconstruction is established by the
destructor/free/constructor/work epochs, not an assumption of different addresses.

## Preserved review history and remaining work

Before compilation, source review caught and corrected witness/helper naming
collisions. Before the first engine push, review caught a missing complete-wire
predicate for mode16; it now requires all four bytes and the physical DMA/final-
wire/TXC/CS order. Synthetic positives and targeted ordering mutants were corrected.
These were pre-execution review findings, not failed actual guest runs. Previous
checkpoint failures and originals remain preserved in their dated results.

The initial source `aebbb80c2991e0f7c08fb7558506cf183c2b1d8f` passed
[ten-case run38686110477](https://github.com/CrispStrobe/labwired-core/actions/runs/38686110477).
Its515198-byte sole original SHA256
`bf11789c006b30b377f97e7dcb04af52f51f5fc2c2d6b2c23ba6481c822f8890`
and frozen original binding are retained separately. But its broader
[Rust CI38686110371](https://github.com/CrispStrobe/labwired-core/actions/runs/38686110371)
failed its feature-off unit job:4349 passed,1 failed,3 ignored. The new harness's
feature conditional raised the conditional-site census to88 against ceiling87.
The failed sole job-log original is preserved, SHA256
`0d12ed5ae4f2ec149137cbd01b20b322b212e76825b64058ab236591e8902277`.
The correction uses the core's existing compile-time availability query rather
than duplicating that conditional. No ratchet ceiling or assertion changed.
The fresh final-head qualification above passed; the earlier passing actual
run was not silently promoted to that source.

Next close required board/reset/NVM/serial and other APIs against compatible
immutable dependencies and the explicit no-ASF build. Complete selected source/
header/inline/toolchain notices and map/image/CF2 admission, then actual frames,
input press/release and debugger pause/step/reset. Only then measure representative
active native/WASM RTx and qualify the exact installed consumer artifact/pins;
optimize an observed bottleneck with controlled repeated A/B.

Concrete board/API blockers should not be hidden by this SPI checkpoint:

- The current service profile deliberately panics for `target_reset`; a real
  reset caller remains required. The selected engine has an AIRCR/reset-vector
  path, but its reset helper explicitly covers core-local state, not complete
  peripheral reset. Qualify actual reset reentry and name that boundary.
- The historical image's CODAL core uses pointer-address NVM methods and
  `copy`; the currently selected service core uses integer addresses and
  `read`. Compatible declarations are not proof that every actual board/settings
  consumer compiles. Keep old immutable caller pins and compile the new closure
  separately instead of silently mixing versions.
- The selected [PyBadge descriptor](https://github.com/CrispStrobe/labwired-core/blob/e3d8c8b0149763f0bfed4d6cfde0a54b221257ed/configs/chips/atsamd51-pybadge.yaml)
  has no NVM-controller entry and explicitly stubs USB/QSPI. A plain flash memory
  region does not implement NVM commands. Any service required by the chosen
  native image needs an admitted implementation; exclusion or a success stub
  cannot satisfy that requirement.

Natural stale IRQ/timeouts, deliberately pending-edge teardown, arbitrary
concurrency/lifetime, complete native Arcade, application adoption, physical
behavior and distribution/licence clearance remain separate. No app pin,
threshold, acknowledgement or physical capture changes.
