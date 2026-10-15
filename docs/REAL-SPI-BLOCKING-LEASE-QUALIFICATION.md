# Prospective actual blocking terminal-result ownership

The bounded caller is now qualified and landed through Runtime32/Core237.
Read the [exact actual result, original failures and remaining limits](REAL-SPI-BLOCKING-LEASE-RESULTS.md).
The retained prospective contract below is not object-lifetime, complete native
Arcade, installed-consumer, performance or distribution admission.

Mode14 uses separate actual blocking-owner and cancelling fibers, with a stable
primary-RAM buffer. The owner calls the real blocking transfer and suspends;
the next worker cancels while retaining execution with interrupts masked. That
worker immediately attempts a valid competing asynchronous start and must get
DEVICE_BUSY before the original owner can resume and consume its terminal result.
No yield or invented Event separates cancellation from the competing attempt.

The owner must return DEVICE_SPI_ERROR from the selected immutable header,
in its original thread/fiber context and with interrupts unmasked. Blocking
cancellation must wake it through the real NotifyOne Event, not invoke an
asynchronous failure handler. The separately noinline transfer, cancellation,
competing-start, owner and canceller functions permit actual PC/LR and
return-register observations. Header-derived error and NotifyOne constants are
assigned after zero-BSS startup checks, before workers start. The worker's entry
and final mask witnesses are zero; cancellation and its immediate competing
attempt run inside the masked section and must be observed with mask one.
Main enrolls before either fiber is created and wakes only from the owner's
completion Event. Both worker returns must precede main's recovery epoch.

Preserve every partial accepted/wire byte, frozen writeback count, actual
NotifyOne wake, assembly context switch and strict SPI/channel/CS cleanup.
Observe competing entry/return and DEVICE_BUSY before original-owner resume;
scalar flags alone do not establish result ownership. After original-owner
consumption, the unchanged positive asynchronous and distinct blocking recovery
must succeed. All driver, Timer, MessageBus and buffers remain alive throughout.

This is source-only preparation, with unchanged driver/model and old callers.
Mode14 requires a separate clean source-bound build and full fourteen dependency
rules; old modes0–13 images must stay unchanged. Both actual engine profiles,
positive controls, an independent frozen source/image-bound reader, preserved
failure originals and every enabled exact-head check remain required before
matched-head/tree-qualified merge. Object destruction, arbitrary concurrent
access, natural faults, complete native Arcade, physical behavior, installed
consumers, RTx and distribution/licence clearance remain separate boundaries.
