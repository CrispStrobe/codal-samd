# Prospective actual CODAL Timer caller — build control only

This separate fixture compiles the **unchanged complete** `Timer.cpp` and its
actual public headers from CODAL core candidate
`2ed19d64ec3359143409a4b0a9758b4aeef3ee7d`. Timer source blob:
`3429b24ca26654ca68cbdbf75e0d30de046496f0`; header:
`aa09bc0f54ebd914dfcce3585e78f2b28dd7e6f1`. It links our explicitly selected
strict paired-IRQ profile. No method extraction, vendor headers, changed upstream
source, default runtime adoption or whole-core compatibility is claimed.

The candidate's integer-address NVM header matches current SAMDNVM signatures;
that separate observation does not qualify its complete JACDAC/storage/board ABI.
The historical dependency pin and its original results remain unchanged.

## Implemented prospective observations

The authored probe intends to exercise actual constructor/sync/calibration,
`setEvent` nesting through `triggerIn`, already-disabled entry, cancellation
through `recomputeNextTimerEvent`, unmatched cancellation, `sync` under an outer
pair, event-list exhaustion and destruction. It uses software-pended IRQ0;
delivery must occur with mask0 and a closed paired state, never inside an open
outer Timer pair. The probe completes six numbered caller checks, two IRQ entries,
fault0 and final masked status `0x600d`; it records terminal failure codes rather
than returning invented success. None of those outcomes has been observed yet.

## Explicit test-adapter boundaries

- `RecordingTimer` advances an authored counter by ten per capture and records
  compare calls. Constructor calibration uses that counter, **not physical time**.
  There is no SAMD timer MMIO, peripheral IRQ, asynchronous callback or DMA.
- A single 1KiB allocation serves the event list. This is not the actual CODAL
  allocator or its out-of-memory behavior; unexpected allocation/deletion rejects.
- Component construction/destruction is counted, not installed in the actual
  component registry, scheduler tick or default event bus.
- Event dispatch and every unsupported low-level operation visibly reject.
  There is no success stub for an event bus, USB, timer IRQ controls or hardware.
- The fixture platform selects USB/WebUSB and diagnostics off. It is NOT the
  historical application configuration, whose required USB/WebUSB remain owed.
  Diagnostic identity1 is not an actual fiber or scheduler guard.

Timer/header notices and the core repository notice are retained in the immutable
dependency checkout. The header without a local notice and exact compiler/libgcc
components still require component-specific review; this build is not whole-runtime
licence or redistribution clearance. This is source-reviewed integration, not
isolated or whole-firmware clean-room implementation.

## Acceptance before merge or a caller-qualified claim

1. Pass explicit-selection refusal, clean immutable dependency checks and hosted
   compile/link gates. Bind source/config/compiler/header closure, ELF/map identity
   and retained upstream notices. Do not call compilation guest execution.
2. Add a separately named LabWired guest harness for **this** caller (not the five
   earlier authored policy cases). Check vectors, BSS/data initialization, each
   numbered witness, masked entry/restoration, exact IRQ count/exception/context,
   thread return and fault0 under ordinary/event schedulers with bounded execution.
3. Build guest adversaries for unconditional/early restoration and unsupported
   event/callback entry. Require exact observable failures; compiler failure,
   missing witness or timeout cannot count as successful rejection.
4. Audit the original source-bound packet without producer imports; retain all
   failures and all enabled final-head checks before a matched-head own-fork merge.
5. Then replace these explicitly excluded adapters through actual component,
   allocator, event-bus, clock and fiber integration. Continue startup/panic,
   required services, named no-ASF build, native boot and active RTx separately.

The code is prospective and must remain draft until affected guest qualification.
The existing [paired-IRQ result](../docs/PAIRED-IRQ-QUALIFICATION.md) is a prerequisite,
not proof of this actual-caller fixture.
