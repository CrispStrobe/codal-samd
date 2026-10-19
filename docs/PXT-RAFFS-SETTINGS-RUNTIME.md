# Actual RAFFS/settings composition contract

This separate lane follows the fixed-engine adoption in PR72. Preparation may
proceed on the proposed parent, but actual qualification and merge require its
qualified landed composition. Earlier compile-only reports and ZFlash driver
results do not qualify this new filesystem/services composition.

## Declared persistence boundary

Select exactly 32 KiB of manual-storage flash at `0x78000..0x80000`, matching the
actual SAMD51 settings default. Persistence means completed filesystem/settings
operations followed by destruction of the settings wrapper and a new object
mount over the same machine and same flash backing. There is no CPU reset,
engine restart, power loss, interrupted-write recovery or physical endurance
claim. Never hold two live filesystem caches on the same banks.

The original settings singleton has no public teardown hook. A separately
selected authored same-translation-unit hook may destroy it and clear its
pointer only after operations finish. Keep the actual settings and RAFFS method
bodies unchanged; the hook must call the original object APIs and cannot invent
filesystem returns or repair storage bytes. Retain the empty original FS
destructor and disclose its finite-object lifetime limit.

## Source and qualification envelope

Own a new `pxt-storage-runtime/` producer, authored caller, observer, descriptor,
mutation controller and `.github/workflows/pxt-storage-runtime.yml`, plus this
contract and the roadmap links. Reuse the admitted framework/CODAL source pins
and qualified engine, actual GC, objects, roots, configuration, clock, message bus,
fiber and scheduler services. Preserve the actual FNV implementation from
`base/core.cpp`, actual string/buffer/array definitions and vtables and any real
transitive implementation they retain. Enumerate selected dependencies and
notices; supply no success stubs, fake filesystem or fabricated object layout.
New toolchain library inputs, if needed, require explicit source/license and
archive-member admission before a positive result can qualify.

The authored caller must exercise actual format, mount, write, read, remove,
existence and listing, missing/refused operations, object remount persistence,
and both live native and rooted managed payloads across real GC pressure.
Observe real method returns, finite ordered stages, exact payloads and declared
code/data/neighbor canaries in both scheduler profiles. Scope source mutants to
specific data, removal/remount or retention outcomes; setup, compiler failure or
timeout cannot qualify a mutant. Freeze independent original readers before
hosted execution, acquire each official original once and preserve failures.

No existing GC/DMA case, bound, terminal, assertion, historical census, descriptor
default, capture or ACK changes. This does not qualify full board IRQ/USB,
generated Arcade, browser/installed application adoption or RTx. Every enabled
final-head check, affected clean guest, original audit and exact reviewed,
tested, predicted and landed tree/ordered-parent join remains required.
