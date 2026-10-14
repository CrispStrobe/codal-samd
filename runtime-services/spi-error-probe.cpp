// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Actual controller-corruption AHB/writeback fault and recovery, not valid-buffer failure.
#include "Sercom4TxSPI.h"
#include "CodalFiber.h"
#include "InterruptMask.h"
using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
using Spi = brickwright_runtime::Sercom4TxSPI;
static Spi *spi;
static uint8_t bytes[] = {0x12, 0x34, 0x56, 0x78};
// Keep the admitted buffer unchanged; only the exclusive controller's disabled
// writeback target is corrupted by actual guest MMIO. Preserve partial progress.
static uint8_t error_bytes[32] = {0x91, 0x82, 0x73, 0x64};
extern "C" {
volatile unsigned runtime_checks = 0, runtime_heap_reused = 0;
volatile unsigned runtime_mode = BW_RUNTIME_TEST_MODE;
volatile unsigned runtime_spi_stage = 0, runtime_spi_callbacks = 0;
volatile unsigned runtime_spi_callback_ipsr = 0, runtime_spi_callback_cs = 0;
volatile unsigned runtime_spi_argument = 0, runtime_spi_reentry_result = 0;
volatile unsigned runtime_spi_main_identity = 0, runtime_spi_worker_identity = 0;
volatile unsigned runtime_spi_blocking_returned = 0;
volatile unsigned runtime_spi_error_stage = 0, runtime_spi_failures = 0;
volatile unsigned runtime_spi_failure_result = 0, runtime_spi_failure_ipsr = 0;
volatile unsigned runtime_spi_failure_mask = 0, runtime_spi_failure_cs = 0;
volatile unsigned runtime_spi_failure_argument = 0, runtime_spi_failure_reentry = 0;
volatile unsigned runtime_spi_error_worker = 0, runtime_spi_failure_fiber = 0;
volatile unsigned runtime_spi_error_returned = 0, runtime_spi_error_busy = 0;
volatile unsigned runtime_spi_error_saved_writeback = 0, runtime_spi_error_remaining = 0;
volatile unsigned runtime_spi_error_disabled_control = 0, runtime_spi_error_target = 0;
volatile unsigned runtime_spi_error_armed = 0, runtime_spi_cleanup_spi = 0;
volatile unsigned runtime_spi_cleanup_inten = 0, runtime_spi_cleanup_channel = 0;
}
static void require(bool ok, unsigned reason) { if (!ok) target_panic(reason); }
static unsigned exception() {
    unsigned value;
    __asm__ volatile("mrs %0, ipsr" : "=r"(value) : : "memory");
    return value;
}
static unsigned cs() {
    return (*reinterpret_cast<volatile uint32_t *>(0x41008090) >> 7) & 1;
}
extern "C" __attribute__((noinline)) void runtime_spi_done(void *argument) {
    runtime_spi_callback_ipsr = exception();
    runtime_spi_callback_cs = cs();
    runtime_spi_argument = reinterpret_cast<unsigned>(argument);
    require(argument == spi && exception() == 79 && cs() == 1, 0xc91);
    ++runtime_spi_callbacks;
    runtime_spi_reentry_result = unsigned(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi));
    require(runtime_spi_reentry_result == unsigned(DEVICE_BUSY), 0xc92);
    Event(52, 7);
}
extern "C" __attribute__((noinline)) void runtime_spi_failed(void *argument, Spi::State::Result result) {
    ++runtime_spi_failures;
    runtime_spi_failure_result = unsigned(result);
    runtime_spi_failure_ipsr = exception();
    runtime_spi_failure_mask = Mask::read();
    runtime_spi_failure_cs = cs();
    runtime_spi_failure_argument = reinterpret_cast<unsigned>(argument);
    runtime_spi_failure_fiber = reinterpret_cast<unsigned>(currentFiber);
    require(argument == spi && result == Spi::State::Result::dma_error &&
            exception() == 47 && Mask::read() == 0 && cs() == 1, 0xc93);
    runtime_spi_cleanup_spi = *reinterpret_cast<volatile uint32_t *>(0x43000000);
    runtime_spi_cleanup_inten = *reinterpret_cast<volatile uint8_t *>(0x43000016);
    runtime_spi_cleanup_channel = *reinterpret_cast<volatile uint32_t *>(0x4100a040);
    require(runtime_spi_cleanup_spi == 0x2000c && runtime_spi_cleanup_inten == 0 &&
            runtime_spi_cleanup_channel == 0x200d00 &&
            *reinterpret_cast<volatile uint32_t *>(0x4100a038) == 0x60000000 &&
            runtime_spi_error_armed == 1, 0xcd7);
    runtime_spi_failure_reentry = unsigned(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi));
    require(runtime_spi_failure_reentry == unsigned(DEVICE_BUSY), 0xc94);
    Event(56, 5); // Real MessageBus delivery wakes the enrolled main fiber.
}
static volatile uint32_t &word(uint32_t address) {
    return *reinterpret_cast<volatile uint32_t *>(address);
}
static void fault_worker() {
    const unsigned saved = Mask::saveAndDisable();
    runtime_spi_error_worker = reinterpret_cast<unsigned>(currentFiber);
    runtime_spi_error_busy = unsigned(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi));
    require(runtime_spi_error_busy == unsigned(DEVICE_BUSY), 0xcd1);
    runtime_spi_error_saved_writeback = word(0x4100a038);
    require(runtime_spi_error_saved_writeback >= 0x20000000 &&
            runtime_spi_error_saved_writeback <= 0x2002dff0 &&
            runtime_spi_error_saved_writeback % 16 == 0, 0xcd2);
    // WBADDR is globally enable-protected. Retain the live channel descriptor:
    // disabling/re-enabling the channel would restart it from its initial table.
    word(0x4100a000) = 0;
    runtime_spi_error_disabled_control = word(0x4100a000);
    require((runtime_spi_error_disabled_control & 2) == 0, 0xcd4);
    // Interrupt masking does not stop DMA ticks. Snapshot only AFTER global
    // disable, so no accepted beat can race this remaining-count witness.
    runtime_spi_error_remaining =
        *reinterpret_cast<volatile uint16_t *>(runtime_spi_error_saved_writeback + 2);
    require(runtime_spi_error_remaining > 0 && runtime_spi_error_remaining < 32, 0xcd3);
    word(0x4100a038) = 0x60000000;
    runtime_spi_error_target = word(0x4100a038);
    require(runtime_spi_error_target == 0x60000000, 0xcd5);
    runtime_spi_error_armed = 1;
    word(0x4100a000) = 0x102;
    require(word(0x4100a000) == 0x102, 0xcd6);
    runtime_spi_error_returned = 1;
    Mask::restore(saved);
}
static void blocking_worker() {
    runtime_spi_worker_identity = reinterpret_cast<unsigned>(currentFiber);
    require(spi->transfer(bytes, sizeof(bytes), nullptr, 0) == DEVICE_OK, 0xc99);
    require(reinterpret_cast<unsigned>(currentFiber) == runtime_spi_worker_identity, 0xc9a);
    runtime_spi_blocking_returned = 1;
    Event(53, 9);
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    require(runtime_mode == 12 && Mask::read() == 1, 0xc9b);
    Mask::restore(0);
    void *first = malloc(64);
    require(first != nullptr, 0xc9c);
    free(first);
    void *second = malloc(64);
    require(second == first, 0xc9d);
    free(second);
    runtime_heap_reused = 1;
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    require(fiber_scheduler_running() && currentFiber, 0xc9e);
    runtime_spi_main_identity = reinterpret_cast<unsigned>(currentFiber);
    spi = new Spi(*timer, *bus);
    require(reinterpret_cast<unsigned>(spi) % 8 == 0, 0xc9f);
    require(spi->setFailureHandler(runtime_spi_failed) == DEVICE_OK, 0xca0);
    require(create_fiber(fault_worker) != nullptr, 0xca1);
    require(fiber_wake_on_event(56, 5) == DEVICE_OK, 0xca2);
    runtime_spi_error_stage = 1;
    require(spi->startTransfer(error_bytes, sizeof(error_bytes), nullptr, 0, runtime_spi_done, spi) == DEVICE_OK, 0xca3);
    schedule();
    require(runtime_spi_error_returned == 1 && runtime_spi_failures == 1 &&
            runtime_spi_callbacks == 0 && runtime_spi_error_worker != runtime_spi_main_identity &&
            runtime_spi_error_armed == 1 &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xca4);
    runtime_spi_error_stage = 2;
    runtime_checks = 1;
    require(fiber_wake_on_event(52, 7) == DEVICE_OK, 0xca5);
    runtime_spi_stage = 1;
    require(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi) == DEVICE_OK, 0xca6);
    schedule();
    require(runtime_spi_callbacks == 1 && reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xca7);
    runtime_spi_stage = 2;
    runtime_checks = 2;
    require(create_fiber(blocking_worker) != nullptr, 0xca8);
    require(fiber_wait_for_event(53, 9) == DEVICE_OK, 0xca9);
    require(runtime_spi_blocking_returned == 1 && runtime_spi_callbacks == 1 &&
            runtime_spi_failures == 1 && runtime_spi_worker_identity != runtime_spi_main_identity &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xcaa);
    runtime_spi_stage = 3;
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
