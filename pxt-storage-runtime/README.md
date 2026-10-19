# Selected actual RAFFS/settings runtime caller

Candidate only; no hosted runtime result is yet qualified. Read the
[scope and persistence contract](../docs/PXT-RAFFS-SETTINGS-RUNTIME.md) first.
The existing GC/DMA and historical census inputs remain unchanged.

The producer retains the existing GC/services and driver link-only packets,
then replaces the authored caller and extracted configuration objects with the
new caller and actual complete `base/core.cpp`. Actual RAFFS/settings methods,
hash implementation, PXT object/vtable definitions and CODAL compatibility
implementation are retained. Public-header adaptations change includes only;
a separate same-translation-unit authored hook calls original settings object
APIs to format, inspect geometry and destroy the single wrapper before remount.

The new settings translation unit uses explicit `-O2` concrete construction,
as the qualified standalone ZFlash boot caller does. The public abstract Flash
base declares a non-pure `totalSize` without an implementation; no fallback body
is supplied here. Qualification concerns the actual derived ZFlash vtable.
Other new translation units and the unchanged GC/services producer keep their
recorded flags. Every compile command is retained; this changes no existing
release, performance or qualification profile.

Full actual core object/vtable linkage retains real math and string dependencies.
The workflow installs newlib explicitly and retains exact source package bytes,
package identities, archive hashes, link-map members and member bytes, source
candidates and notices. An archive member's possible source paths are not an
independent source-to-binary rebuild. Independently review their component and
notice bindings before admitting a positive result. Do not provide fake math,
I/O, syscall, object or filesystem success stubs to satisfy a missing symbol.

The guest first retains the real GC/global/current/suspended-root caller, then
runs six ordered storage phases with 1871 authored checks. It checks missing
`_get`/`_remove` results, three writes, exact values and listings, rooted saved
buffers and a live native buffer through actual collections, removal and a
fresh wrapper mount over the same flash bytes. The original FS destructor is
empty; this is a finite object-remount boundary, without reset, power loss,
concurrent filesystem contention, exhaustion/wear-out or full-board claims.
The observer checks real method returns, initialized data canaries and the
entire flash range below storage at phase boundaries and the terminal.

Each scheduler profile must detect four isolated actual source mutations:
missing settings read-copy, falsely successful removal, an omitted saved-buffer
root, and keeping the old cache instead of remounting. Each requires its named
observed wrong result; compilation/setup failure, timeout, unrelated panic or
a positive terminal is insufficient. Preserve failures and acquire each
source-bound official original once with a previously frozen independent reader.
