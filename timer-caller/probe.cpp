// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Actual pinned Timer.cpp caller, with explicitly bounded diagnostic adapters.
// The clock/component/heap adapters below are NOT a shipped CODAL backend.
#include "Timer.h"
#include "Event.h"
#include "InterruptMask.h"

using Mask = brickwright_simulation::InterruptMask;
extern "C" {
extern volatile unsigned bw_simulation_irq_fault;
bool bw_simulation_irq_may_switch_context();
volatile unsigned control_status = 0;
volatile unsigned irq_count = 0, irq_mask_seen = 0, irq_exception_seen = 0;
volatile unsigned irq_closed_seen = 0, compare_count = 0, component_count = 0;
volatile unsigned caller_checks = 0;
unsigned bw_simulation_fiber_identity() { return 1; } // Diagnostic, not a fiber.
}

static volatile unsigned &reg(unsigned address) {
    return *reinterpret_cast<volatile unsigned *>(address);
}
static void quiesce() {
    Mask::restore(1);
    reg(0xe000e180) = 1;
    reg(0xe000e280) = 1;
    __asm__ volatile("dsb\n\tisb" : : : "memory");
}
[[noreturn]] static void reject(unsigned reason) {
    quiesce();
    control_status = reason;
    while (true) __asm__ volatile("nop" : : : "memory");
}
static void require(bool condition, unsigned reason) {
    if (!condition) reject(reason);
}
static void settle() {
    for (unsigned i = 0; i < 32; ++i) __asm__ volatile("nop" : : : "memory");
}

// A single bounded allocation, only for Timer's event list. No heap admission.
alignas(8) static unsigned char event_storage[1024];
static bool allocated;
extern "C" void *malloc(size_t bytes) {
    if (allocated || bytes == 0 || bytes > sizeof(event_storage)) reject(0xa01);
    allocated = true;
    return event_storage;
}
extern "C" void *memset(void *destination, int value, size_t bytes) {
    unsigned char *out = static_cast<unsigned char *>(destination);
    for (size_t i = 0; i < bytes; ++i) out[i] = static_cast<unsigned char>(value);
    return destination;
}
void operator delete(void *) noexcept { reject(0xa02); }
void operator delete(void *, size_t) noexcept { reject(0xa03); }
extern "C" void target_panic(int) { reject(0xa04); }
extern "C" void __cxa_pure_virtual() { reject(0xa05); }

// Record construction/destruction without pretending to install component
// listeners, a default event bus or a scheduler system-tick subscription.
void codal::CodalComponent::addComponent() { component_count = component_count + 1; }
void codal::CodalComponent::removeComponent() {
    require(component_count == 1, 0xa06);
    component_count = component_count - 1;
}
// Timer callback reaches this constructor if event firing is attempted.
// Event dispatch is excluded and MUST visibly reject rather than claim success.
codal::Event::Event(uint16_t, uint16_t, CODAL_TIMESTAMP, EventLaunchMode) {
    reject(0xa07);
}

class RecordingTimer final : public codal::LowLevelTimer {
    uint32_t counter = 0;
public:
    bool arm_pending = false;
    RecordingTimer() : LowLevelTimer(2) { bitMode = codal::BitMode32; }
    int enable() override { return DEVICE_OK; } // Diagnostic counter only.
    int enableIRQ() override { reject(0xa08); }
    int disable() override { reject(0xa09); }
    int disableIRQ() override { reject(0xa0a); }
    int reset() override { reject(0xa0b); }
    int setMode(codal::TimerMode) override { reject(0xa0c); }
    int offsetCompare(uint8_t, uint32_t) override { reject(0xa0d); }
    int clearCompare(uint8_t) override { reject(0xa0e); }
    int setClockSpeed(uint32_t) override { reject(0xa0f); }
    int setBitMode(codal::TimerBitMode) override { reject(0xa10); }
    uint32_t captureCounter() override {
        // Explicit authored increments allow the ACTUAL constructor calibration
        // to finish; this is not elapsed physical time or hardware calibration.
        counter += 10;
        return counter;
    }
    int setCompare(uint8_t channel, uint32_t) override {
        require(channel < 2, 0xa11);
        if (channel == 1) {
            require(Mask::read() == 1 && !bw_simulation_irq_may_switch_context(), 0xa12);
            compare_count = compare_count + 1;
            if (arm_pending) {
                reg(0xe000e200) = 1;
                __asm__ volatile("dsb\n\tisb" : : : "memory");
                settle();
                require(Mask::read() == 1, 0xa13);
            }
        }
        return DEVICE_OK;
    }
};

extern "C" void authored_irq0() {
    unsigned exception;
    __asm__ volatile("mrs %0, ipsr" : "=r"(exception) : : "memory");
    irq_mask_seen = Mask::read();
    irq_exception_seen = exception;
    irq_closed_seen = bw_simulation_irq_may_switch_context();
    require(irq_mask_seen == 0 && exception == 16 && irq_closed_seen == 1, 0xa14);
    irq_count = irq_count + 1;
}
extern "C" void unexpected_exception() { reject(0xf00d); }

extern "C" void irq_mask_probe() {
    control_status = 0x100;
    require(Mask::read() == 1 && bw_simulation_irq_fault == 0, 0xb01);
    {
        RecordingTimer low;
        codal::Timer timer(low); // Actual constructor and actual sync/calibration.
        require(Mask::read() == 1 && component_count == 1, 0xb02);
        require(bw_simulation_irq_may_switch_context(), 0xb03);
        caller_checks = 1;
        reg(0xe000e100) = 1;
        low.arm_pending = true;
        Mask::restore(0);
        // Actual setEvent -> actual triggerIn: nested pair must not deliver early.
        require(timer.eventAfterUs(1000, 42, 7) == DEVICE_OK, 0xb04);
        settle();
        require(irq_count == 1 && Mask::read() == 0 && compare_count == 1, 0xb05);
        caller_checks = 2;
        // Already-disabled entry must remain disabled with IRQ still pending.
        Mask::restore(1);
        require(timer.eventAfterUs(500, 43, 9) == DEVICE_OK, 0xb06);
        settle();
        require(irq_count == 1 && Mask::read() == 1 && compare_count == 2, 0xb07);
        require((reg(0xe000e200) & 1) && bw_simulation_irq_may_switch_context(), 0xb08);
        Mask::restore(0);
        settle();
        require(irq_count == 2, 0xb09);
        low.arm_pending = false;
        caller_checks = 3;
        // Cancel the next event: recompute -> triggerIn nests in actual cancel.
        require(timer.cancel(43, 9) == DEVICE_OK && compare_count == 3, 0xb0a);
        require(timer.cancel(99, 99) == DEVICE_INVALID_PARAMETER, 0xb0b);
        require(Mask::read() == 0 && bw_simulation_irq_may_switch_context(), 0xb0c);
        caller_checks = 4;
        // Real sync entered beneath an authored outer pair must not enable it.
        target_disable_irq();
        timer.getTimeUs();
        require(Mask::read() == 1 && !bw_simulation_irq_may_switch_context(), 0xb0d);
        target_enable_irq();
        require(Mask::read() == 0 && bw_simulation_irq_may_switch_context(), 0xb0e);
        caller_checks = 5;
        // Actual no-free-slot early return must not create an unmatched pair.
        for (unsigned i = 0; i < CODAL_TIMER_DEFAULT_EVENT_LIST_SIZE - 1; ++i)
            require(timer.eventAfterUs(10000, 50 + i, 1) == DEVICE_OK, 0xb0f);
        require(timer.eventAfterUs(10000, 99, 1) == DEVICE_NO_RESOURCES, 0xb10);
        require(Mask::read() == 0 && bw_simulation_irq_fault == 0, 0xb11);
        caller_checks = 6;
    }
    require(component_count == 0 && irq_count == 2, 0xb12);
    quiesce();
    control_status = 0x600d;
}
