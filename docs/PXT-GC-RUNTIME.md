# Real PXT GC and CODAL services: prospective linked caller

This [hosted builder](../pxt-gc-runtime/build_gc.py) progresses beyond separate
translation-unit compilation. It selects the immutable public PXT GC and object
implementation, actual CODAL services and the approved
[allocator ownership split](PXT-NATIVE-ALLOCATION.md). The original hosted link
passed, but its default guest failed to reach the unchanged 12-million-step
terminal bound. The second scheduler profile did not run. The corrected caller
now has the bounded pass described below; complete runtime or native Arcade
qualification is not claimed.

## Exact source selection

The existing closed public source profile and generator verification remain
mandatory. The complete real `gc.cpp` body is retained with one include change
and one exact public `realloc` copy-length correction described below.
The complete real `pxt.cpp` body is retained with one include change and removal
of its unused `using namespace std` directive. The complete actual PXT base
header retains all type/field/method/GC definitions; only seven include lines
select explicitly named freestanding declarations and the real IPSR observation.
No substitute PXT classes or empty GC implementation is supplied.

The actual SAMD adapter's `list_fibers`, thread-context access, copied-stack
address mapping and `gcProcessStacks` functions are extracted byte-for-byte
between unique source boundaries. Actual configuration lookup and its original
weak bootloader fallback are extracted likewise from `base/core.cpp`. Excluding
unrelated board/application functions is explicit; this does not qualify the
complete SAMD adapter, board initialization or generated program inputs.

The authored probe supplies a declared small GC metadata/configuration fragment,
not a generated `pointers.cpp` or an Arcade executable. Actual configuration
lookup reads its 32 KiB system-heap reserve. The existing three exact CODAL
service patches remain selected at the unchanged core pin. Native C++ new/delete
and TCB allocation calls in a separately adapted target source use only the
named aligned domain; real PXT placement-new objects retain GC ownership.
Existing service source files and builds are not modified.

The new profile also adapts the actual patched CODAL fiber implementation's
one allocation and four direct frees to the matching native domain. This is
essential: freeing a native TCB or Fiber through PXT's public `free` would lose
its aligned-allocation metadata. The timer's one event-buffer allocation is
native too. Three exact weak public allocator aliases are removed from the
separately adapted raw heap source, so missing GC cannot silently fall back to
the raw allocation API. Original patched sources and exact adaptations are kept.

The new startup retains and invokes actual `.init_array` constructors before
entering the probe. Silently discarding GC static initialization is not allowed.
The linker requires no unresolved symbols; each object is checked so that only
actual `gc.cpp` defines the public `malloc/free/realloc` names. This does not
assert complete C library compatibility or general-purpose allocation semantics.

## Authored caller and qualification owed

The probe checks native alignment/reallocation preservation, simultaneous public
and native allocation contents, real managed objects across collections, global
roots and current/suspended fiber stack roots. Its C++ stack spans are published
explicitly to the actual thread context; they are not generated PXT ABI evidence.
The [hosted guest fixture](../pxt-gc-runtime/guest.rs) targets the unchanged,
previously qualified engine revision `73f8475f8e406b1ff4475bc9886e22479881d67a`,
using its existing lockfile and both scheduler profiles. Only this owned consumer
test is added to a fresh checkout; engine product sources and dependencies stay
unchanged. It poisons RAM before startup, checks real data/BSS initialization
before constructors, observes constructor entry and actual context switches,
and requires four ordered stages and the exact 60-check terminal witness within
12 million steps. Missing-witness controls are synthetic, not guest results.

The first corrected caller pass below has successful clean hosted execution
and independently audited originals. Actual-source nonvacuous mutation controls
remain required before merging this runtime slice. A successful link alone
does not qualify the caller.
ISR/in-GC routing, exhaustive failure/churn cases
and DMA buffer lifetime remain additional affected tests before runtime adoption.

Pure controls:

```sh
python3 -B -m unittest discover -s pxt-gc-runtime -p 'test_*.py' -v
```

Original/adapted selected public sources, dependency hashes, compiler/link logs,
symbol lists, image digests and component notices are retained. The selected
profile disables USB/WebUSB and is not the original board configuration. Full
board services, flash/controller models, actual native Arcade frames/input/debug,
representative native/WASM RTx and installed-app adoption remain required.
No engine/package pin, physical capture or hardware acknowledgement changes.

## Original failure and bounded diagnostic — 2026-10-15

The original [hosted run](https://github.com/CrispStrobe/codal-samd/actions/runs/38774479471)
compiled and linked the selected real sources. The synthetic terminal controls
passed; the default actual guest exhausted 12 million steps without reaching
the required terminal witness. No second-profile result exists. This is a
preserved failed qualification, not a successful runtime or an engine diagnosis.
Its official artifact and log were retained once after exact provider/source
metadata admission. Independent component checks cover 12 adapted inputs,
17 primary units, the dependency-rule census, allocator symbol ownership and
actual image hashes; these do not turn the guest failure into a pass.

The next diagnostic requires the exact original ELF SHA-256
`755575add70c5c21df7d73768020c90b2a791cb66cbfb6c67de0a727cd8e4d44`
before execution. It adds read-only stage/final-state and actual instruction-PC
counts within `memset`/`memcpy`, plus the real public `realloc` copy call's ABI
arguments and allocation-header capacity. It does not change firmware inputs,
source algorithms, assertions, the 60-check witness or the step bound. A
nonterminal run still fails. The immutable image and extra observations are
required to choose a correction from evidence, not to justify a longer timeout.

The host harness now resolves immutable ELF entry symbols once before stepping,
instead of repeatedly walking the full symbol table on every guest instruction.
This changes host observation overhead only; it is not a firmware optimization
or a representative target RTx result. No GC copy-length or runtime code change
was included in that diagnostic.

## Observed copy defect and clearing bottleneck

The [unchanged-image diagnostic](https://github.com/CrispStrobe/codal-samd/actions/runs/38779809420)
also failed the original bound, at stage 3 with 56 of 60 checks. Its official
artifact was independently audited against the original ELF and binary hashes,
actual consumer source and ordered tested-commit parents. At the real `memcpy`
entry, public `realloc` requested 28 bytes from a 24-byte payload: the decoded
seven-word block size includes the one-word array header, but the source pointer
is already past that header. This is an observed four-byte over-read, not merely
a source prediction. The independently retained failure remains a failure.

Of 12 million pre-step PC samples, 11,479,882 (95.7%) were within actual
`memset`; the final PC was also there. These are host observation samples, not
retired instruction counts, representative RTx or physical timing. The second
scheduler profile again did not run after the default failure.

The new candidate subtracts the one header word at the sole, exact public GC
copy site. Original selected public sources remain retained. Its consumer now
requires observation of the actual 24-byte copy before accepting the unchanged
60-check terminal witness. Restoring the old copy must fail this observation.
Because this is a changed firmware candidate, it is bound to its own build
report and independently audited image, not mislabeled with the diagnostic's
original ELF hash.

The candidate additionally applies GCC's
`optimize("O2", "no-tree-loop-distribute-patterns")` attribute only to the
existing `memset` definition in the separately adapted target source. Its byte
loop is unchanged; other functions retain the existing `-O0` flags. This avoids
introducing word-aliasing, alignment or tail-write assumptions. Existing target
source and service-only builds remain byte-for-byte unchanged. No bound,
pressure rounds, root scan, terminal assertions, scheduler profile, dependency
pin or hardware acknowledgement is relaxed. Fresh actual execution in both
profiles, independent artifact audit and nonvacuous source mutations remain
owed before qualification or merge.

The first optimization attempt, [run 38790535583](https://github.com/CrispStrobe/codal-samd/actions/runs/38790535583),
failed during early guest initialization. The retained actual ARM disassembly
shows that `optimize("O2")` alone transformed the byte loop into a call to
`memset` itself, despite the translation unit's freestanding/no-builtin flags.
That failed image is preserved; no successful guest or copy observation is
claimed from it. The revised attribute explicitly disables loop-distribution
pattern recognition. The builder now inspects the actual ARM object and refuses
emitted calls or self-recursive branches in `memset`. A separate hosted control
executes the exact selected function body through a volatile function pointer
over 41,280 combinations of alignment, length and fill value, checking the
return pointer and every byte inside and outside the requested range. Host
controls are supplementary and do not replace either actual ARM guest profile.

## First independently audited finite caller pass

The corrected [run 38792565299](https://github.com/CrispStrobe/codal-samd/actions/runs/38792565299)
passed both actual scheduler profiles. Independent source/dependency/image/fixture
audit verifies the original official artifact, all 46 selected dependency bytes,
component notices, call-free ARM clearing routine, exact host boundary harness,
and the actual corrected 24-byte public copy. Each profile reached all 60 checks
in 5,853,557 steps with four context swaps and one static constructor observed,
within the unchanged 12-million-step bound. This is a finite authored GC caller
result, not generated Arcade execution, representative RTx or board qualification.

The next campaign additionally builds three explicit source mutants, each under
both scheduler profiles: restore the old public copy length, omit current-fiber
roots, or omit suspended-fiber roots. It reuses baseline objects except for the
one mutated translation unit and relinks into isolated images; it never
overwrites the successful baseline image. Each must fail its named correctness
check, not merely time out or fail setup. Raw output, explicit source deltas and
image hashes are retained in a separately named mutation artifact. Original
baseline receipt fields remain labeled as baseline facts; no mutant is admitted
as a positive source-bound build. These six actual negative results are still
owed, alongside ISR/in-GC/failure/churn/DMA coverage before runtime adoption.
