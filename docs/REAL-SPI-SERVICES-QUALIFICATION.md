# Prospective real CODAL SERCOM4 TX service

This separately selected mode-8 caller connects the actual CODAL SPI interface,
Timer, MessageBus, heap and fibers to the modeled SERCOM4 hardware-triggered
byte-DMA profile. It is not complete native Arcade, physical hardware, CF2,
installed-app or performance admission. No source build or guest outcome is
claimed before the corresponding hosted checks complete.

The driver exclusively owns DMAC channel 0, SERCOM4 and PyBadge PB7 chip select,
with PB13/PB15 peripheral mux. Display D/C remains the display caller's
responsibility. Static descriptors are sixteen-byte aligned; driver/service
objects use the separately qualified eight-byte-aligned public heap adapter.
Source dependencies and patches remain explicit and retain their notices.
The historical foundation `simulation/include/SpiTransferState.h` remains
byte-for-byte unchanged. The separately named `RuntimeSpiTransferState.h`
selects the runtime types; hosted positive/five mutation controls reuse the
authored assertions with explicit include/type substitutions. Those are host
state controls, not ARM ABI, driver or guest qualification.

The mode-8 build selects both authored `Sercom4TxSPI.cpp` and actual core
`source/driver-models/SPI.cpp`, with fourteen C++ dependency rules. Its separate
startup routes vector indices 47 and 79 to real DMA and SPI handlers (IRQs 31
and 63). Existing eight service/clock/heap modes retain their original startup
and source-unit selection. The generic SAMD51 chip descriptor alone does not
select this DMA model; hosted execution needs the explicit named DMA profile.

The caller uses a real asynchronous callback in SPI IRQ context, tests that
callback reentry refuses busy, and emits an actual Event to wake its enrolled
main fiber. It then runs a distinct real worker through blocking transfer,
result consumption and Event delivery back to the waiting main fiber. Async
zero length explicitly refuses; blocking zero length is a no-op.

Required hosted qualification observes actual accepted DATA, DMA writeback
zero/completion, DMA IRQ, final wire bytes, SPI TXC/IRQ, CS release, callback
entry/return and actual fiber resume. Callback counters alone are insufficient.
No host callback/event injection or replacement service implementation is used.
Error, cancellation, clock-gated timeout, stale IRQ, ownership and recovery
controls remain required before claiming the complete selected TX service.
RX and other SPI modes are unsupported in this first profile; full native
Arcade required APIs, board/CF2/build/component notices, frames/input/debugger,
representative RTx and installed consumer qualification remain pending.
