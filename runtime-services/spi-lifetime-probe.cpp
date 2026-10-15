// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Separate expected-refusal callers and a settled-idle reconstruction control.
// Not cancellation-and-destruction, arbitrary concurrency or native Arcade.
#include "Sercom4TxSPI.h"
#include "CodalFiber.h"
#include "InterruptMask.h"
using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
using Spi = brickwright_runtime::Sercom4TxSPI;
static Spi *spi;
static uint8_t bytes[] = {0x12, 0x34, 0x56, 0x78};
static uint8_t partial_bytes[32] = {0x91, 0x82, 0x73, 0x64};
extern "C" {
volatile unsigned runtime_checks = 0, runtime_heap_reused = 0;
volatile unsigned runtime_mode = BW_RUNTIME_TEST_MODE;
volatile unsigned runtime_spi_stage = 0, runtime_spi_callbacks = 0;
volatile unsigned runtime_spi_callback_ipsr = 0, runtime_spi_callback_cs = 0;
volatile unsigned runtime_spi_argument = 0, runtime_spi_reentry_result = 0;
volatile unsigned runtime_spi_main_identity = 0, runtime_spi_worker_identity = 0;
volatile unsigned runtime_spi_blocking_returned = 0;
volatile unsigned runtime_spi_lifetime_stage = 0, runtime_spi_lifetime_object = 0;
volatile unsigned runtime_spi_lifetime_fresh = 0, runtime_spi_lifetime_actor = 0;
volatile unsigned runtime_spi_lifetime_mask = 0, runtime_spi_lifetime_ipsr = 0;
volatile unsigned runtime_spi_lifetime_returned = 0;
volatile unsigned runtime_spi_lifetime_return_mask = 0;
volatile unsigned runtime_spi_lifetime_return_ipsr = 0;
volatile unsigned runtime_spi_lifetime_return_actor = 0;
volatile unsigned runtime_spi_lifetime_owner_identity = 0, runtime_spi_lifetime_canceller_identity = 0;
volatile unsigned runtime_spi_lifetime_cancel_result = 0;
volatile unsigned runtime_spi_lifetime_owner_returned = 0;
volatile unsigned runtime_spi_lifetime_failures = 0;
volatile unsigned runtime_spi_lifetime_writeback = 0, runtime_spi_lifetime_remaining = 0;
volatile unsigned runtime_spi_lifetime_expected_panic = 0;
}
static void require(bool ok, unsigned reason) { if (!ok) target_panic(reason); }
static volatile uint32_t &word(unsigned address) {
    return *reinterpret_cast<volatile uint32_t *>(address);
}
static unsigned exception() {
    unsigned value;
    __asm__ volatile("mrs %0, ipsr" : "=r"(value) : : "memory");
    return value;
}
static unsigned identity() { return reinterpret_cast<unsigned>(currentFiber); }
static unsigned cs() { return (word(0x41008090) >> 7) & 1; }
static unsigned remaining() {
    const unsigned address = word(0x4100a038);
    require(address >= 0x20000000 && address <= 0x2002dff0 && address % 16 == 0, 0xe31);
    runtime_spi_lifetime_writeback = address;
    return *reinterpret_cast<volatile uint16_t *>(address + 2);
}
extern "C" __attribute__((noinline)) void runtime_spi_lifetime_failed(void *, Spi::State::Result) {
    ++runtime_spi_lifetime_failures;
    target_panic(0xe32);
}
// An actual virtual delete through the selected upstream SPI interface. The
// outer wrapper records return only AFTER the deleting destructor and free.
extern "C" __attribute__((noinline)) void runtime_spi_lifetime_delete(codal::SPI *value) {
    delete value;
    __asm__ volatile("" : : : "memory");
}
extern "C" __attribute__((noinline)) void runtime_spi_lifetime_destroy() {
    runtime_spi_lifetime_actor = identity();
    runtime_spi_lifetime_mask = Mask::read();
    runtime_spi_lifetime_ipsr = exception();
    runtime_spi_lifetime_stage = 3;
    runtime_spi_lifetime_delete(spi);
    runtime_spi_lifetime_return_actor = identity();
    runtime_spi_lifetime_return_mask = Mask::read();
    runtime_spi_lifetime_return_ipsr = exception();
    runtime_spi_lifetime_returned = 1;
    runtime_spi_lifetime_stage = 4;
    if (BW_RUNTIME_TEST_MODE != 18) target_panic(0xe33);
    spi = nullptr;
}
extern "C" __attribute__((noinline)) void runtime_spi_done(void *argument) {
    runtime_spi_callback_ipsr = exception();
    runtime_spi_callback_cs = cs();
    runtime_spi_argument = reinterpret_cast<unsigned>(argument);
    require(argument == spi && exception() == 79 && cs() == 1, 0xe34);
    ++runtime_spi_callbacks;
    if (BW_RUNTIME_TEST_MODE == 16) {
        // finish() consumed its token, but still owns delivering_.
        runtime_spi_lifetime_destroy();
        target_panic(0xe35);
    }
    require(BW_RUNTIME_TEST_MODE == 18, 0xe36);
    runtime_spi_reentry_result = unsigned(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0,
                                                           runtime_spi_done, spi));
    require(runtime_spi_reentry_result == unsigned(DEVICE_BUSY), 0xe37);
    Event(52, 7);
}
extern "C" __attribute__((noinline)) void runtime_spi_lifetime_owner() {
    runtime_spi_lifetime_owner_identity = identity();
    require(identity() != runtime_spi_main_identity && exception() == 0 && Mask::read() == 0, 0xe38);
    runtime_spi_lifetime_stage = 2;
    const int result = spi->transfer(partial_bytes, sizeof(partial_bytes), nullptr, 0);
    runtime_spi_lifetime_owner_returned = 1;
    // This owner MUST NOT resume before the canceller's destructor refusal.
    (void)result;
    target_panic(0xe39);
}
extern "C" __attribute__((noinline)) int runtime_spi_lifetime_cancel() {
    int result = spi->cancel();
    __asm__ volatile("" : "+r"(result) : : "memory");
    return result;
}
extern "C" __attribute__((noinline)) void runtime_spi_lifetime_canceller() {
    runtime_spi_lifetime_canceller_identity = identity();
    require(runtime_spi_lifetime_stage == 2 && identity() != runtime_spi_lifetime_owner_identity &&
            identity() != runtime_spi_main_identity && exception() == 0 && Mask::read() == 0, 0xe3a);
    Mask::restore(1);
    runtime_spi_lifetime_cancel_result = unsigned(runtime_spi_lifetime_cancel());
    require(runtime_spi_lifetime_cancel_result == unsigned(DEVICE_OK) &&
            runtime_spi_lifetime_owner_returned == 0 && Mask::read() == 1, 0xe3b);
    runtime_spi_lifetime_remaining = remaining();
    require(runtime_spi_lifetime_remaining > 0 && runtime_spi_lifetime_remaining < 32, 0xe3c);
    // NotifyOne made the owner runnable, but the terminal token is still owned.
    runtime_spi_lifetime_destroy();
    target_panic(0xe3d);
}
extern "C" __attribute__((noinline)) void runtime_spi_lifetime_blocking_worker() {
    runtime_spi_worker_identity = identity();
    require(spi->transfer(bytes, sizeof(bytes), nullptr, 0) == DEVICE_OK, 0xe3e);
    require(identity() == runtime_spi_worker_identity && exception() == 0 && Mask::read() == 0, 0xe3f);
    ++runtime_spi_blocking_returned;
    Event(53, 9);
}
static void positive(unsigned ordinal) {
    require(fiber_wake_on_event(52, 7) == DEVICE_OK, 0xe40);
    runtime_spi_stage = ordinal * 2 - 1;
    require(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi) == DEVICE_OK, 0xe41);
    schedule();
    require(runtime_spi_callbacks == ordinal && identity() == runtime_spi_main_identity, 0xe42);
    runtime_spi_stage = ordinal * 2;
    require(create_fiber(runtime_spi_lifetime_blocking_worker) != nullptr, 0xe43);
    require(fiber_wait_for_event(53, 9) == DEVICE_OK, 0xe44);
    require(runtime_spi_blocking_returned == ordinal && runtime_spi_callbacks == ordinal &&
            runtime_spi_worker_identity != runtime_spi_main_identity &&
            identity() == runtime_spi_main_identity, 0xe45);
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    require(runtime_mode >= 15 && runtime_mode <= 18 && Mask::read() == 1, 0xe46);
    runtime_spi_lifetime_expected_panic = runtime_mode == 18 ? 0 : 0xc42;
    Mask::restore(0);
    void *first = malloc(64);
    require(first != nullptr, 0xe47);
    free(first);
    void *second = malloc(64);
    require(second == first, 0xe48);
    free(second);
    runtime_heap_reused = 1;
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    require(fiber_scheduler_running() && currentFiber, 0xe49);
    runtime_spi_main_identity = identity();
    spi = new Spi(*timer, *bus);
    runtime_spi_lifetime_object = reinterpret_cast<unsigned>(spi);
    require(runtime_spi_lifetime_object % 8 == 0 &&
            spi->setFailureHandler(runtime_spi_lifetime_failed) == DEVICE_OK, 0xe4a);
    runtime_checks = 1;
    runtime_spi_lifetime_stage = 1;
    if (runtime_mode == 15) {
        require(spi->startTransfer(partial_bytes, sizeof(partial_bytes), nullptr, 0,
                                  runtime_spi_done, spi) == DEVICE_OK, 0xe4b);
        Mask::restore(1);
        // Observe actual writeback progress, never host-seed a partial result.
        unsigned budget = 10000;
        unsigned count;
        do { count = remaining(); } while (count >= 32 && --budget);
        runtime_spi_lifetime_remaining = count;
        require(budget && count > 0 && count < 32 && cs() == 0, 0xe4c);
        runtime_spi_lifetime_stage = 2;
        runtime_spi_lifetime_destroy();
        target_panic(0xe4d);
    }
    if (runtime_mode == 16) {
        require(fiber_wake_on_event(52, 7) == DEVICE_OK, 0xe4e);
        require(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi) == DEVICE_OK, 0xe4f);
        runtime_spi_lifetime_stage = 2;
        schedule();
        target_panic(0xe50);
    }
    if (runtime_mode == 17) {
        require(fiber_wake_on_event(58, 1) == DEVICE_OK, 0xe51);
        require(create_fiber(runtime_spi_lifetime_owner) != nullptr &&
                create_fiber(runtime_spi_lifetime_canceller) != nullptr, 0xe52);
        schedule();
        target_panic(0xe53);
    }
    positive(1);
    runtime_checks = 2;
    Mask::restore(1);
    runtime_spi_lifetime_destroy();
    require(runtime_spi_lifetime_returned == 1 &&
            runtime_spi_lifetime_return_mask == 1 && runtime_spi_lifetime_return_ipsr == 0 &&
            runtime_spi_lifetime_return_actor == runtime_spi_main_identity && spi == nullptr, 0xe54);
    spi = new Spi(*timer, *bus);
    runtime_spi_lifetime_fresh = reinterpret_cast<unsigned>(spi);
    require(runtime_spi_lifetime_fresh % 8 == 0 &&
            spi->setFailureHandler(runtime_spi_lifetime_failed) == DEVICE_OK, 0xe55);
    runtime_spi_lifetime_stage = 5;
    Mask::restore(0);
    positive(2);
    require(runtime_spi_lifetime_failures == 0, 0xe56);
    runtime_spi_lifetime_stage = 6;
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
