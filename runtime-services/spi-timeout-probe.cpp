// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Actual gated timeout/cleanup and recovery; not full native Arcade admission.
#include "Sercom4TxSPI.h"
#include "CodalFiber.h"
#include "InterruptMask.h"
#include "codal_target_hal.h"
using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
using Spi = brickwright_runtime::Sercom4TxSPI;
static Spi *spi;
static uint8_t bytes[] = {0x12, 0x34, 0x56, 0x78};
static uint8_t timeout_bytes[32] = {0x91, 0x82, 0x73, 0x64};
static constexpr unsigned GATE = BW_RUNTIME_TEST_MODE == 10 ? 0x40000820 : 0x40001d08;
static constexpr unsigned GATE_BIT = BW_RUNTIME_TEST_MODE == 10 ? 1 : 64;
static volatile uint32_t &word(unsigned address) { return *reinterpret_cast<volatile uint32_t *>(address); }
static unsigned byte(unsigned address) { return *reinterpret_cast<volatile uint8_t *>(address); }
extern "C" {
volatile unsigned runtime_checks = 0, runtime_heap_reused = 0;
volatile unsigned runtime_mode = BW_RUNTIME_TEST_MODE;
volatile unsigned runtime_spi_stage = 0, runtime_spi_callbacks = 0;
volatile unsigned runtime_spi_callback_ipsr = 0, runtime_spi_callback_cs = 0;
volatile unsigned runtime_spi_argument = 0, runtime_spi_reentry_result = 0;
volatile unsigned runtime_spi_main_identity = 0, runtime_spi_worker_identity = 0;
volatile unsigned runtime_spi_blocking_returned = 0;
volatile unsigned runtime_spi_timeout_stage = 0, runtime_spi_failures = 0;
volatile unsigned runtime_spi_failure_result = 0, runtime_spi_failure_ipsr = 0;
volatile unsigned runtime_spi_failure_mask = 0, runtime_spi_failure_cs = 0;
volatile unsigned runtime_spi_failure_argument = 0, runtime_spi_failure_reentry = 0;
volatile unsigned runtime_spi_gate_worker = 0, runtime_spi_failure_fiber = 0;
volatile unsigned runtime_spi_gate_saved = 0, runtime_spi_gate_held = 0;
volatile unsigned runtime_spi_gate_restored = 0, runtime_spi_cleanup_spi = 0;
volatile unsigned runtime_spi_cleanup_inten = 0, runtime_spi_cleanup_channel = 0;
volatile unsigned runtime_spi_gate_remaining = 0, runtime_spi_timeout_remaining = 0;
volatile unsigned runtime_spi_gate_cycles = 0, runtime_spi_timeout_cycles = 0;
volatile unsigned runtime_spi_cleanup_cycles = 0, runtime_spi_gate_busy = 0;
volatile unsigned runtime_spi_after_wait = 0;
volatile unsigned runtime_spi_wait_begin_cycles = 0, runtime_spi_wait_end_cycles = 0;
volatile unsigned runtime_spi_wait_begin_us = 0, runtime_spi_wait_end_us = 0;
}
static void require(bool ok, unsigned reason) { if (!ok) target_panic(reason); }
static unsigned exception() { unsigned value; __asm__ volatile("mrs %0, ipsr" : "=r"(value) : : "memory"); return value; }
static unsigned cs() { return (word(0x41008090) >> 7) & 1; }
static unsigned remaining() {
    const unsigned wb = word(0x4100a038);
    require(wb >= 0x20000000 && wb <= 0x2002dff0 && wb % 16 == 0, 0xcb0);
    return *reinterpret_cast<volatile uint16_t *>(wb + 2);
}
extern "C" __attribute__((noinline)) void runtime_spi_done(void *argument) {
    runtime_spi_callback_ipsr = exception();
    runtime_spi_callback_cs = cs();
    runtime_spi_argument = reinterpret_cast<unsigned>(argument);
    require(argument == spi && exception() == 79 && cs() == 1, 0xcb1);
    ++runtime_spi_callbacks;
    runtime_spi_reentry_result = unsigned(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi));
    require(runtime_spi_reentry_result == unsigned(DEVICE_BUSY), 0xcb2);
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
    runtime_spi_timeout_cycles = word(0xe0001004);
    runtime_spi_gate_held = word(GATE);
    runtime_spi_timeout_remaining = remaining();
    runtime_spi_timeout_stage = 2;
    require(argument == spi && result == Spi::State::Result::timeout &&
            exception() == 15 && Mask::read() == 0 && cs() == 1, 0xcb3);
    require((runtime_spi_gate_held & GATE_BIT) == 0 &&
            runtime_spi_timeout_remaining == runtime_spi_gate_remaining, 0xcb4);
    // Gated reads return zero and gated writes are dropped. Restore the real
    // gate BEFORE checking cleanup, and BEFORE any recovery init can erase it.
    word(GATE) = runtime_spi_gate_saved;
    runtime_spi_gate_restored = word(GATE);
    runtime_spi_cleanup_spi = word(0x43000000);
    runtime_spi_cleanup_inten = byte(0x43000016);
    runtime_spi_cleanup_channel = word(0x4100a040);
    runtime_spi_cleanup_cycles = word(0xe0001004);
    runtime_spi_timeout_stage = 3;
    require(runtime_spi_gate_restored == runtime_spi_gate_saved &&
            (runtime_spi_cleanup_spi & 2) == 0 && runtime_spi_cleanup_inten == 0 &&
            (runtime_spi_cleanup_channel & 2) == 0 && cs() == 1, 0xcb5);
    runtime_spi_failure_reentry = unsigned(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi));
    require(runtime_spi_failure_reentry == unsigned(DEVICE_BUSY), 0xcb6);
    Event(55, 4); // Actual Timer -> immediate listener -> Event wakes main.
}
static void gating_worker() {
    // Publish the gate/writeback/clock witnesses as one short transaction;
    // a pending deadline may fire only after all of them are initialized.
    const unsigned saved = Mask::saveAndDisable();
    runtime_spi_gate_worker = reinterpret_cast<unsigned>(currentFiber);
    runtime_spi_gate_busy = unsigned(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi));
    require(runtime_spi_gate_busy == unsigned(DEVICE_BUSY), 0xcb7);
    runtime_spi_gate_saved = word(GATE);
    require((runtime_spi_gate_saved & GATE_BIT) != 0, 0xcb8);
    word(GATE) = runtime_spi_gate_saved & ~GATE_BIT;
    runtime_spi_gate_remaining = remaining();
    runtime_spi_gate_cycles = word(0xe0001004);
    runtime_spi_timeout_stage = 1;
    require(runtime_spi_gate_remaining > 0 && runtime_spi_gate_remaining < 32 &&
            (word(0x40000810) & (1u << 9)) != 0, 0xcb9);
    Mask::restore(saved);
}
static void blocking_worker() {
    runtime_spi_worker_identity = reinterpret_cast<unsigned>(currentFiber);
    require(spi->transfer(bytes, sizeof(bytes), nullptr, 0) == DEVICE_OK, 0xcba);
    require(reinterpret_cast<unsigned>(currentFiber) == runtime_spi_worker_identity, 0xcbb);
    runtime_spi_blocking_returned = 1;
    Event(53, 9);
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    require((runtime_mode == 10 || runtime_mode == 11) && Mask::read() == 1, 0xcbc);
    Mask::restore(0);
    void *first = malloc(64); require(first != nullptr, 0xcbd); free(first);
    void *second = malloc(64); require(second == first, 0xcbe); free(second);
    runtime_heap_reused = 1;
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    require(fiber_scheduler_running() && currentFiber, 0xcbf);
    runtime_spi_main_identity = reinterpret_cast<unsigned>(currentFiber);
    spi = new Spi(*timer, *bus);
    require(reinterpret_cast<unsigned>(spi) % 8 == 0, 0xcc0);
    require(spi->setFailureHandler(runtime_spi_failed) == DEVICE_OK && spi->setTimeoutUs(1000) == DEVICE_OK, 0xcc1);
    require(create_fiber(gating_worker) != nullptr && fiber_wake_on_event(55, 4) == DEVICE_OK, 0xcc2);
    require(spi->startTransfer(timeout_bytes, sizeof(timeout_bytes), nullptr, 0, runtime_spi_done, spi) == DEVICE_OK, 0xcc3);
    schedule();
    require(runtime_spi_failures == 1 && runtime_spi_callbacks == 0 &&
            runtime_spi_gate_worker != runtime_spi_main_identity &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xcc4);
    runtime_spi_timeout_stage = 4;
    runtime_spi_wait_begin_us = low->captureCounter();
    runtime_spi_wait_begin_cycles = word(0xe0001004);
    target_wait_us(1500); // Actual clock, beyond the original one-shot deadline.
    runtime_spi_wait_end_cycles = word(0xe0001004);
    runtime_spi_wait_end_us = low->captureCounter();
    require(runtime_spi_failures == 1 && runtime_spi_callbacks == 0, 0xcc5);
    require(runtime_spi_wait_end_us - runtime_spi_wait_begin_us >= 1500, 0xccb);
    runtime_spi_after_wait = 1;
    // Recovery uses the unchanged positive async/blocking ordering contract.
    require(spi->setTimeoutUs(1000000) == DEVICE_OK && fiber_wake_on_event(52, 7) == DEVICE_OK, 0xcc6);
    runtime_spi_stage = 1;
    runtime_checks = 1;
    require(spi->startTransfer(bytes, sizeof(bytes), nullptr, 0, runtime_spi_done, spi) == DEVICE_OK, 0xcc7);
    schedule();
    require(runtime_spi_callbacks == 1 && reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xcc8);
    runtime_spi_stage = 2;
    runtime_checks = 2;
    require(create_fiber(blocking_worker) != nullptr && fiber_wait_for_event(53, 9) == DEVICE_OK, 0xcc9);
    require(runtime_spi_blocking_returned == 1 && runtime_spi_callbacks == 1 && runtime_spi_failures == 1 &&
            runtime_spi_worker_identity != runtime_spi_main_identity &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_spi_main_identity, 0xcca);
    runtime_spi_stage = 3;
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
