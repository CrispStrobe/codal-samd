// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Real CODAL callback/Event/fiber caller, not complete native Arcade admission.
#include "Sercom4TxSPI.h"
#include "CodalFiber.h"
#include "InterruptMask.h"
using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
static brickwright_runtime::Sercom4TxSPI *spi;
// This named DMAC profile admits primary RAM sources only. Startup must copy
// these initialized bytes into RAM; a const flash buffer is not admitted.
static uint8_t bytes[] = {0x12, 0x34, 0x56, 0x78};
static const uint8_t flash_bytes[] = {0x12, 0x34, 0x56, 0x78};
extern "C" {
volatile unsigned runtime_checks = 0, runtime_heap_reused = 0;
volatile unsigned runtime_mode = BW_RUNTIME_TEST_MODE;
volatile unsigned runtime_spi_stage = 0, runtime_spi_callbacks = 0;
volatile unsigned runtime_spi_callback_ipsr = 0, runtime_spi_callback_cs = 0;
volatile unsigned runtime_spi_argument = 0, runtime_spi_reentry_result = 0;
volatile unsigned runtime_spi_main_identity = 0, runtime_spi_worker_identity = 0;
volatile unsigned runtime_spi_blocking_returned = 0;
}
static void require(bool ok, unsigned reason) { if (!ok) target_panic(reason); }
extern "C" __attribute__((noinline)) void runtime_spi_done(void *argument) {
    unsigned ipsr;
    __asm__ volatile("mrs %0, ipsr" : "=r"(ipsr) : : "memory");
    runtime_spi_callback_ipsr = ipsr;
    runtime_spi_callback_cs = (*reinterpret_cast<volatile uint32_t *>(0x41008090) >> 7) & 1;
    runtime_spi_argument = reinterpret_cast<unsigned>(argument);
    require(argument == spi && ipsr == 79 && runtime_spi_callback_cs == 1, 0xc81);
    ++runtime_spi_callbacks;
    const int reentry = spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi);
    runtime_spi_reentry_result = unsigned(reentry);
    require(reentry == DEVICE_BUSY, 0xc82);
    Event(52, 7); // Actual MessageBus/Fiber wake, never host callback injection.
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
    require(runtime_mode == 8 && Mask::read() == 1, 0xc85);
    Mask::restore(0);
    void *first = malloc(64);
    require(first != nullptr, 0xc86);
    free(first);
    void *second = malloc(64);
    require(second == first, 0xc87);
    free(second);
    runtime_heap_reused = 1;
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    require(fiber_scheduler_running() && currentFiber, 0xc88);
    runtime_spi_main_identity = reinterpret_cast<unsigned>(currentFiber);
    spi = new brickwright_runtime::Sercom4TxSPI(*timer, *bus);
    require(reinterpret_cast<unsigned>(spi) % 8 == 0, 0xc89);
    require(spi->startTransfer(nullptr, 0, nullptr, 0, runtime_spi_done, spi) == DEVICE_INVALID_PARAMETER, 0xc8a);
    require(spi->transfer(nullptr, 0, nullptr, 0) == DEVICE_OK, 0xc8b);
    require(spi->startTransfer(flash_bytes, sizeof(flash_bytes), nullptr, 0,
                              runtime_spi_done, spi) == DEVICE_INVALID_PARAMETER, 0xc92);
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
            runtime_spi_worker_identity != runtime_spi_main_identity &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xc91);
    runtime_spi_stage = 3;
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
