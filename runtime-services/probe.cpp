// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Actual services integration caller; no alternative Event/Heap/Fiber bodies.
#include "Runtime.h"
#include "MessageBus.h"
#include "CodalFiber.h"
#include "Timer.h"
#include "InterruptMask.h"

using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
extern "C" {
volatile unsigned runtime_listener_calls = 0, runtime_fiber_started = 0;
volatile unsigned runtime_fiber_woken = 0, runtime_main_woken = 0;
volatile unsigned runtime_worker_identity = 0, runtime_main_identity = 0;
volatile unsigned runtime_listener_exception = 0, runtime_checks = 0;
volatile unsigned runtime_mode = BW_RUNTIME_TEST_MODE;
volatile unsigned runtime_heap_reused = 0;
volatile unsigned runtime_clock_stage = 0, runtime_clock_pending_seen = 0;
volatile unsigned runtime_clock_before = 0, runtime_clock_after = 0;
}
static void require(bool condition, unsigned reason) {
    if (!condition) target_panic(reason);
}
static void timerListener(Event evt) {
    unsigned ipsr;
    __asm__ volatile("mrs %0, ipsr" : "=r"(ipsr) : : "memory");
    runtime_listener_exception = ipsr;
    require(evt.source == 42 && evt.value == 7 && ipsr == 15, 0xb01);
    ++runtime_listener_calls;
}
static void worker() {
    runtime_worker_identity = reinterpret_cast<unsigned>(currentFiber);
    runtime_fiber_started = 1;
    require(fiber_wait_for_event(42, 7) == DEVICE_OK, 0xb02);
    require(currentFiber && reinterpret_cast<unsigned>(currentFiber) == runtime_worker_identity,
            0xb03);
    runtime_fiber_woken = 1;
    Event(43, 9);
}
static void clock_adversary(brickwright_runtime::CoreClockTimer &low, Timer &timer) {
    Mask::restore(1);
    require(runtime_clock_irqs == 0 && runtime_listener_calls == 0, 0xb21);
    require(timer.eventAfterUs(1000, 42, 7) == DEVICE_OK, 0xb22);
    runtime_clock_stage = 1;
    const uint32_t start = low.captureCounter();
    // Real DWT time crosses the armed SysTick deadline under actual PRIMASK.
    // No host pend, artificial clock advance or callback/event injection.
    while (low.captureCounter() - start < 2000)
        __asm__ volatile("nop" : : : "memory");
    runtime_clock_pending_seen =
        (*reinterpret_cast<volatile uint32_t *>(0xe000e010) >> 16) & 1;
    require(runtime_clock_pending_seen == 1 && runtime_clock_irqs == 0 &&
            runtime_listener_calls == 0 && Mask::read() == 1, 0xb23);
    require((runtime_mode == 2 ? low.disableIRQ() : low.disable()) == DEVICE_OK, 0xb24);
    runtime_clock_before = low.captureCounter();
    runtime_clock_stage = 2;
    Mask::restore(0); // The already-pending real exception may enter and return.
    for (unsigned i = 0; i < 256; ++i) __asm__ volatile("nop" : : : "memory");
    runtime_clock_after = low.captureCounter();
    runtime_clock_stage = 3;
    require(runtime_listener_calls == 0, 0xb20); // Exact stale-event failure.
    require(runtime_clock_irqs == 1, 0xb25);
    require(runtime_mode == 2 ? runtime_clock_after > runtime_clock_before :
                               runtime_clock_after == runtime_clock_before, 0xb26);
    runtime_clock_stage = 4;
    require((runtime_mode == 2 ? low.enableIRQ() : low.enable()) == DEVICE_OK, 0xb27);
    const uint32_t resumed = low.captureCounter();
    while (runtime_listener_calls == 0 && low.captureCounter() - resumed < 2000)
        __asm__ volatile("nop" : : : "memory");
    require(runtime_listener_calls == 1 && runtime_listener_exception == 15, 0xb28);
    require(runtime_clock_irqs == 2 && runtime_switches == 0 && runtime_restores == 0, 0xb29);
    require(timer.cancel(42, 7) == DEVICE_INVALID_PARAMETER, 0xb2a);
    runtime_clock_stage = 5;
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    require(Mask::read() == 1, 0xb04);
    target_disable_irq();
    target_disable_irq();
    target_enable_irq();
    target_enable_irq();
    require(Mask::read() == 1, 0xb05);
    Mask::restore(0);
    void *first = malloc(64);
    require(first != NULL, 0xb06);
    memset(first, 0x5a, 64);
    free(first);
    void *second = malloc(64);
    require(second == first, 0xb07);
    runtime_heap_reused = 1;
    free(second);
    runtime_checks = 1;
    // CODAL fibers page their active stack. IRQ/global service pointers must
    // refer to stable heap objects, not objects on the shared paged stack.
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    require(fiber_scheduler_running() && currentFiber, 0xb08);
    runtime_main_identity = reinterpret_cast<unsigned>(currentFiber);
    require(bus->listen(42, 7, timerListener, MESSAGE_BUS_LISTENER_IMMEDIATE) == DEVICE_OK,
            0xb09);
    if (runtime_mode == 2 || runtime_mode == 3) {
        runtime_checks = 2;
        clock_adversary(*low, *timer);
        target_panic(0xb2b);
    }
    require(create_fiber(worker) != NULL, 0xb0a);
    runtime_checks = 2;
    if (runtime_mode == 1) {
        target_disable_irq();
        schedule();
        target_panic(0xb0b);
    }
    require(runtime_mode == 0, 0xb0c);
    require(timer->eventAfterUs(20000, 42, 7) == DEVICE_OK, 0xb0d);
    require(fiber_wait_for_event(43, 9) == DEVICE_OK, 0xb0e);
    runtime_main_woken = 1;
    require(runtime_fiber_started == 1 && runtime_fiber_woken == 1 &&
            runtime_listener_calls == 1 && runtime_listener_exception == 15,
            0xb0f);
    require(runtime_worker_identity != runtime_main_identity &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_main_identity,
            0xb10);
    require(runtime_switches >= 3 && runtime_clock_irqs && Mask::read() == 0,
            0xb11);
    require(timer->cancel(42, 7) == DEVICE_INVALID_PARAMETER, 0xb12);
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
