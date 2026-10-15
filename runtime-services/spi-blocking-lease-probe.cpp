// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Actual blocking cancellation/result lease, not a general lifetime proof.
#include "Sercom4TxSPI.h"
#include "CodalFiber.h"
#include "InterruptMask.h"
using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
using Spi = brickwright_runtime::Sercom4TxSPI;
static Spi *spi;
static uint8_t bytes[] = {0x12, 0x34, 0x56, 0x78};
static uint8_t lease_bytes[32] = {0x91, 0x82, 0x73, 0x64};
extern "C" {
volatile unsigned runtime_checks = 0, runtime_heap_reused = 0;
volatile unsigned runtime_mode = BW_RUNTIME_TEST_MODE;
volatile unsigned runtime_spi_stage = 0, runtime_spi_callbacks = 0;
volatile unsigned runtime_spi_callback_ipsr = 0, runtime_spi_callback_cs = 0;
volatile unsigned runtime_spi_argument = 0, runtime_spi_reentry_result = 0;
volatile unsigned runtime_spi_main_identity = 0, runtime_spi_worker_identity = 0;
volatile unsigned runtime_spi_blocking_returned = 0;
volatile unsigned runtime_spi_lease_stage = 0, runtime_spi_lease_owner_identity = 0;
volatile unsigned runtime_spi_lease_canceller_identity = 0, runtime_spi_lease_owner_entry_mask = 0;
volatile unsigned runtime_spi_lease_owner_entry_ipsr = 0, runtime_spi_lease_owner_return_mask = 0;
volatile unsigned runtime_spi_lease_owner_return_ipsr = 0, runtime_spi_lease_owner_result = 0;
volatile unsigned runtime_spi_lease_owner_returned = 0, runtime_spi_lease_cancel_result = 0;
volatile unsigned runtime_spi_lease_cancel_mask_before = 0, runtime_spi_lease_cancel_mask_after = 0;
volatile unsigned runtime_spi_lease_cancel_returned = 0, runtime_spi_lease_competing_result = 0;
volatile unsigned runtime_spi_lease_competing_returned = 0, runtime_spi_lease_failures = 0;
volatile unsigned runtime_spi_lease_cleanup_spi = 0, runtime_spi_lease_cleanup_inten = 0;
volatile unsigned runtime_spi_lease_cleanup_channel = 0, runtime_spi_lease_cleanup_cs = 0;
volatile unsigned runtime_spi_lease_saved_writeback = 0, runtime_spi_lease_remaining = 0;
volatile unsigned runtime_spi_lease_main_completed = 0;
// Assigned from the selected immutable CODAL header after BSS startup checks.
volatile unsigned runtime_spi_lease_expected_error = 0, runtime_spi_lease_expected_notify = 0;
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
extern "C" __attribute__((noinline)) void runtime_spi_lease_failed(void *, Spi::State::Result) {
    ++runtime_spi_lease_failures;
    target_panic(0xd31); // Blocking failures belong to the original waiter only.
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
extern "C" __attribute__((noinline)) int runtime_spi_lease_transfer() {
    int result = spi->transfer(lease_bytes, sizeof(lease_bytes), nullptr, 0);
    // Keep an observable wrapper return after the actual blocking API returns.
    __asm__ volatile("" : "+r"(result) : : "memory");
    return result;
}
extern "C" __attribute__((noinline)) int runtime_spi_lease_competing() {
    int result = spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi);
    __asm__ volatile("" : "+r"(result) : : "memory");
    return result;
}
extern "C" __attribute__((noinline)) int runtime_spi_lease_cancel() {
    int result = spi->cancel();
    __asm__ volatile("" : "+r"(result) : : "memory");
    return result;
}
extern "C" __attribute__((noinline)) void runtime_spi_lease_owner() {
    runtime_spi_lease_owner_identity = reinterpret_cast<unsigned>(currentFiber);
    runtime_spi_lease_owner_entry_mask = Mask::read();
    runtime_spi_lease_owner_entry_ipsr = exception();
    require(runtime_spi_lease_owner_identity != runtime_spi_main_identity &&
            Mask::read() == 0 && exception() == 0, 0xd32);
    runtime_spi_lease_stage = 2;
    runtime_spi_lease_owner_result = unsigned(runtime_spi_lease_transfer());
    runtime_spi_lease_owner_return_mask = Mask::read();
    runtime_spi_lease_owner_return_ipsr = exception();
    require(runtime_spi_lease_owner_result == unsigned(DEVICE_SPI_ERROR) &&
            runtime_spi_lease_owner_result == runtime_spi_lease_expected_error &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_lease_owner_identity &&
            Mask::read() == 0 && exception() == 0 &&
            runtime_spi_lease_cancel_returned == 1 && runtime_spi_lease_competing_returned == 1 &&
            runtime_spi_lease_failures == 0 && runtime_spi_callbacks == 0, 0xd33);
    runtime_spi_lease_owner_returned = 1;
    runtime_spi_lease_stage = 4;
    Event(58, 1); // Enrolled main wakes only after the actual owner consumes.
}
extern "C" __attribute__((noinline)) void runtime_spi_lease_canceller() {
    runtime_spi_lease_canceller_identity = reinterpret_cast<unsigned>(currentFiber);
    runtime_spi_lease_cancel_mask_before = Mask::read();
    require(runtime_spi_lease_stage == 2 && exception() == 0 && Mask::read() == 0 &&
            runtime_spi_lease_canceller_identity != runtime_spi_lease_owner_identity &&
            runtime_spi_lease_canceller_identity != runtime_spi_main_identity &&
            runtime_spi_lease_owner_returned == 0, 0xd34);
    const unsigned saved = Mask::saveAndDisable();
    runtime_spi_lease_cancel_result = unsigned(runtime_spi_lease_cancel());
    require(runtime_spi_lease_cancel_result == unsigned(DEVICE_OK), 0xd35);
    // The original owner has been woken but cannot consume while this real
    // canceller retains execution. No yield/Event is inserted before this call.
    runtime_spi_lease_competing_result = unsigned(runtime_spi_lease_competing());
    runtime_spi_lease_competing_returned = 1;
    require(runtime_spi_lease_competing_result == unsigned(DEVICE_BUSY) &&
            runtime_spi_lease_owner_returned == 0 &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_lease_canceller_identity &&
            Mask::read() == 1 && exception() == 0, 0xd36);
    runtime_spi_lease_cleanup_spi = word(0x43000000);
    runtime_spi_lease_cleanup_inten = byte(0x43000016);
    runtime_spi_lease_cleanup_channel = word(0x4100a040);
    runtime_spi_lease_cleanup_cs = cs();
    runtime_spi_lease_saved_writeback = word(0x4100a038);
    require(runtime_spi_lease_saved_writeback >= 0x20000000 &&
            runtime_spi_lease_saved_writeback <= 0x2002dff0 &&
            runtime_spi_lease_saved_writeback % 16 == 0, 0xd37);
    runtime_spi_lease_remaining = *reinterpret_cast<volatile uint16_t *>(runtime_spi_lease_saved_writeback + 2);
    require(runtime_spi_lease_cleanup_spi == 0x2000c && runtime_spi_lease_cleanup_inten == 0 &&
            runtime_spi_lease_cleanup_channel == 0x200d00 && runtime_spi_lease_cleanup_cs == 1 &&
            runtime_spi_lease_remaining > 0 && runtime_spi_lease_remaining < 32 &&
            runtime_spi_lease_failures == 0 && runtime_spi_callbacks == 0, 0xd38);
    runtime_spi_lease_stage = 3;
    runtime_spi_lease_cancel_returned = 1;
    Mask::restore(saved);
    runtime_spi_lease_cancel_mask_after = Mask::read();
    require(Mask::read() == 0 && exception() == 0, 0xd39);
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
    require(runtime_mode == 14 && Mask::read() == 1, 0xd3a);
    runtime_spi_lease_expected_error = unsigned(DEVICE_SPI_ERROR);
    runtime_spi_lease_expected_notify = unsigned(DEVICE_ID_NOTIFY_ONE);
    Mask::restore(0);
    void *first = malloc(64); require(first != nullptr, 0xd3b); free(first);
    void *second = malloc(64); require(second == first, 0xd3c); free(second);
    runtime_heap_reused = 1;
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    require(fiber_scheduler_running() && currentFiber, 0xd3d);
    runtime_spi_main_identity = reinterpret_cast<unsigned>(currentFiber);
    spi = new Spi(*timer, *bus);
    require(reinterpret_cast<unsigned>(spi) % 8 == 0 &&
            spi->setFailureHandler(runtime_spi_lease_failed) == DEVICE_OK, 0xd3e);
    require(fiber_wake_on_event(58, 1) == DEVICE_OK, 0xd3f);
    runtime_spi_lease_stage = 1;
    // Actual cooperative run-queue order: owner starts and suspends in transfer;
    // the distinct next worker cancels, probes busy without yielding and returns.
    require(create_fiber(runtime_spi_lease_owner) != nullptr &&
            create_fiber(runtime_spi_lease_canceller) != nullptr, 0xd40);
    schedule();
    require(runtime_spi_lease_owner_returned == 1 && runtime_spi_lease_cancel_returned == 1 &&
            runtime_spi_lease_competing_returned == 1 && runtime_spi_lease_stage == 4 &&
            runtime_spi_lease_failures == 0 && runtime_spi_callbacks == 0 &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xd41);
    runtime_spi_lease_main_completed = 1;
    runtime_spi_lease_stage = 5;
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
            runtime_spi_lease_failures == 0 && runtime_spi_worker_identity != runtime_spi_main_identity &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xc91);
    runtime_spi_stage = 3;
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
