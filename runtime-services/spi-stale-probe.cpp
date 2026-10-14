// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Actual software-pended idle IRQ dispatch; not naturally stale hardware IRQs.
#include "Sercom4TxSPI.h"
#include "CodalFiber.h"
#include "InterruptMask.h"
using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
using Spi = brickwright_runtime::Sercom4TxSPI;
static Spi *spi;
static uint8_t bytes[] = {0x12, 0x34, 0x56, 0x78};
extern "C" {
volatile unsigned runtime_checks = 0, runtime_heap_reused = 0;
volatile unsigned runtime_mode = BW_RUNTIME_TEST_MODE;
volatile unsigned runtime_spi_stage = 0, runtime_spi_callbacks = 0;
volatile unsigned runtime_spi_callback_ipsr = 0, runtime_spi_callback_cs = 0;
volatile unsigned runtime_spi_argument = 0, runtime_spi_reentry_result = 0;
volatile unsigned runtime_spi_main_identity = 0, runtime_spi_worker_identity = 0;
volatile unsigned runtime_spi_blocking_returned = 0;
volatile unsigned runtime_spi_stale_stage = 0, runtime_spi_stale_prime_callbacks = 0;
volatile unsigned runtime_spi_stale_prime_ipsr = 0, runtime_spi_stale_prime_cs = 0;
volatile unsigned runtime_spi_stale_prime_argument = 0, runtime_spi_stale_prime_reentry = 0;
volatile unsigned runtime_spi_stale_prime_fiber = 0, runtime_spi_stale_failures = 0;
volatile unsigned runtime_spi_stale_dispatch_fiber = 0;
volatile unsigned runtime_spi_stale_mask_before = 0, runtime_spi_stale_mask_after = 0;
volatile unsigned runtime_spi_stale_pending0_before = 0, runtime_spi_stale_pending1_before = 0;
volatile unsigned runtime_spi_stale_pending0_armed = 0, runtime_spi_stale_pending1_armed = 0;
volatile unsigned runtime_spi_stale_pending0_after = 0, runtime_spi_stale_pending1_after = 0;
volatile unsigned runtime_spi_stale_dma_flags_before = 0, runtime_spi_stale_spi_flags_before = 0;
volatile unsigned runtime_spi_stale_inten_before = 0, runtime_spi_stale_control_before = 0;
volatile unsigned runtime_spi_stale_dma_flags_after = 0, runtime_spi_stale_spi_flags_after = 0;
volatile unsigned runtime_spi_stale_inten_after = 0, runtime_spi_stale_control_after = 0;
}
static void require(bool ok, unsigned reason) { if (!ok) target_panic(reason); }
static volatile uint32_t &word(unsigned address) { return *reinterpret_cast<volatile uint32_t *>(address); }
static unsigned byte(unsigned address) { return *reinterpret_cast<volatile uint8_t *>(address); }
static unsigned exception() {
    unsigned value;
    __asm__ volatile("mrs %0, ipsr" : "=r"(value) : : "memory");
    return value;
}
static unsigned cs() { return (word(0x41008090) >> 7) & 1; }
extern "C" __attribute__((noinline)) void runtime_spi_stale_done(void *argument) {
    runtime_spi_stale_prime_ipsr = exception();
    runtime_spi_stale_prime_cs = cs();
    runtime_spi_stale_prime_argument = reinterpret_cast<unsigned>(argument);
    runtime_spi_stale_prime_fiber = reinterpret_cast<unsigned>(currentFiber);
    require(argument == spi && exception() == 79 && cs() == 1, 0xce1);
    ++runtime_spi_stale_prime_callbacks;
    runtime_spi_stale_prime_reentry = unsigned(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_stale_done, spi));
    require(runtime_spi_stale_prime_reentry == unsigned(DEVICE_BUSY), 0xce2);
    Event(57, 6);
}
extern "C" __attribute__((noinline)) void runtime_spi_stale_failed(void *, Spi::State::Result) {
    ++runtime_spi_stale_failures;
    target_panic(0xce3); // No terminal failure belongs to this successful caller.
}
extern "C" __attribute__((noinline)) void runtime_spi_stale_dispatch() {
    require(exception() == 0 && Mask::read() == 0 && runtime_spi_stale_stage == 2, 0xce4);
    runtime_spi_stale_dispatch_fiber = reinterpret_cast<unsigned>(currentFiber);
    runtime_spi_stale_mask_before = Mask::read();
    const unsigned saved = Mask::saveAndDisable();
    runtime_spi_stale_pending0_before = word(0xe000e200);
    runtime_spi_stale_pending1_before = word(0xe000e204);
    runtime_spi_stale_dma_flags_before = byte(0x4100a04e);
    runtime_spi_stale_spi_flags_before = byte(0x43000018);
    runtime_spi_stale_inten_before = byte(0x43000016);
    runtime_spi_stale_control_before = word(0x43000000);
    require((runtime_spi_stale_pending0_before & 0x80000000u) == 0 &&
            (runtime_spi_stale_pending1_before & 0x80000000u) == 0 &&
            runtime_spi_stale_dma_flags_before == 0 &&
            (runtime_spi_stale_spi_flags_before & 2) != 0 &&
            runtime_spi_stale_inten_before == 0 &&
            runtime_spi_stale_control_before == 0x2000e && cs() == 1, 0xce5);
    runtime_spi_stale_stage = 3;
    // These are actual guest NVIC ISPR stores, not peripheral flag injection.
    // IRQ31 and IRQ63 map to actual exceptions47 and79 respectively.
    word(0xe000e200) = 0x80000000u;
    word(0xe000e204) = 0x80000000u;
    __asm__ volatile("dsb" : : : "memory");
    runtime_spi_stale_pending0_armed = word(0xe000e200);
    runtime_spi_stale_pending1_armed = word(0xe000e204);
    require((runtime_spi_stale_pending0_armed & 0x80000000u) != 0 &&
            (runtime_spi_stale_pending1_armed & 0x80000000u) != 0, 0xce6);
    Mask::restore(saved);
    __asm__ volatile("dsb\n\tisb" : : : "memory");
    // Actual handler entry clears NVIC pending. Thread instructions resume only
    // after the interrupt returns; the engine also observes both entry/returns.
    while ((word(0xe000e200) | word(0xe000e204)) & 0x80000000u)
        __asm__ volatile("nop" : : : "memory");
    runtime_spi_stale_pending0_after = word(0xe000e200);
    runtime_spi_stale_pending1_after = word(0xe000e204);
    runtime_spi_stale_mask_after = Mask::read();
    runtime_spi_stale_dma_flags_after = byte(0x4100a04e);
    runtime_spi_stale_spi_flags_after = byte(0x43000018);
    runtime_spi_stale_inten_after = byte(0x43000016);
    runtime_spi_stale_control_after = word(0x43000000);
    require(exception() == 0 && Mask::read() == 0 && cs() == 1 &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity &&
            runtime_spi_stale_prime_callbacks == 1 && runtime_spi_callbacks == 0 &&
            runtime_spi_stale_failures == 0 && runtime_spi_stale_dma_flags_after == 0 &&
            runtime_spi_stale_spi_flags_after == runtime_spi_stale_spi_flags_before &&
            runtime_spi_stale_inten_after == 0 &&
            runtime_spi_stale_control_after == runtime_spi_stale_control_before, 0xce7);
    runtime_spi_stale_stage = 4;
}
extern "C" __attribute__((noinline)) void runtime_spi_done(void *argument) {
    runtime_spi_callback_ipsr = exception();
    runtime_spi_callback_cs = cs();
    runtime_spi_argument = reinterpret_cast<unsigned>(argument);
    require(argument == spi && exception() == 79 && cs() == 1, 0xc81);
    ++runtime_spi_callbacks;
    runtime_spi_reentry_result = unsigned(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi));
    require(runtime_spi_reentry_result == unsigned(DEVICE_BUSY), 0xc82);
    Event(52, 7);
}
static void blocking_worker() {
    runtime_spi_worker_identity = reinterpret_cast<unsigned>(currentFiber);
    require(spi->transfer(bytes, sizeof(bytes), nullptr, 0) == DEVICE_OK, 0xc83);
    require(reinterpret_cast<unsigned>(currentFiber) == runtime_spi_worker_identity, 0xc84);
    runtime_spi_blocking_returned = 1;
    Event(53, 9);
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    require(runtime_mode == 13 && Mask::read() == 1, 0xce8);
    Mask::restore(0);
    void *first = malloc(64); require(first != nullptr, 0xce9); free(first);
    void *second = malloc(64); require(second == first, 0xcea); free(second);
    runtime_heap_reused = 1;
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    require(fiber_scheduler_running() && currentFiber, 0xceb);
    runtime_spi_main_identity = reinterpret_cast<unsigned>(currentFiber);
    spi = new Spi(*timer, *bus);
    require(reinterpret_cast<unsigned>(spi) % 8 == 0, 0xcec);
    require(spi->setFailureHandler(runtime_spi_stale_failed) == DEVICE_OK &&
            fiber_wake_on_event(57, 6) == DEVICE_OK, 0xced);
    runtime_spi_stale_stage = 1;
    require(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_stale_done, spi) == DEVICE_OK, 0xcee);
    schedule();
    require(runtime_spi_stale_prime_callbacks == 1 && runtime_spi_callbacks == 0 &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xcef);
    runtime_spi_stale_stage = 2;
    runtime_spi_stale_dispatch();
    require(runtime_spi_stale_stage == 4, 0xcf0);
    // The original mode8 async/blocking recovery callback, Event and ordering.
    runtime_checks = 1;
    require(fiber_wake_on_event(52, 7) == DEVICE_OK, 0xc8c);
    runtime_spi_stage = 1;
    require(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi) == DEVICE_OK, 0xc8d);
    schedule();
    require(runtime_spi_callbacks == 1 && reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xc8e);
    runtime_spi_stage = 2;
    runtime_checks = 2;
    require(create_fiber(blocking_worker) != nullptr, 0xc8f);
    require(fiber_wait_for_event(53, 9) == DEVICE_OK, 0xc90);
    require(runtime_spi_blocking_returned == 1 && runtime_spi_callbacks == 1 &&
            runtime_spi_stale_prime_callbacks == 1 && runtime_spi_stale_failures == 0 &&
            runtime_spi_worker_identity != runtime_spi_main_identity &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xc91);
    runtime_spi_stage = 3;
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
