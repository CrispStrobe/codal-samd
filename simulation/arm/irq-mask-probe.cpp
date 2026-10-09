// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "InterruptMask.h"

namespace {
using Mask = brickwright_simulation::InterruptMask;
volatile unsigned& reg(unsigned address) {
    return *reinterpret_cast<volatile unsigned*>(address);
}
const unsigned enable = 0xe000e100, disable = 0xe000e180;
const unsigned pending = 0xe000e200, clearPending = 0xe000e280;

void settle() {
    // A bounded diagnostic window, not a physical IRQ-latency requirement.
    for (unsigned i = 0; i < 32; ++i) {
        __asm__ volatile("nop" : : : "memory");
    }
}

void quiesce() {
    Mask::restore(1);
    reg(disable) = 1;
    reg(clearPending) = 1;
    __asm__ volatile("dsb\n\tisb" : : : "memory");
}
}

extern "C" {
volatile unsigned control_status = 0;
volatile unsigned irq_count = 0, irq_stage = 0, irq_stage_seen = 0;
volatile unsigned irq_mask_seen = 0, irq_exception_seen = 0;

void authored_irq0() {
    unsigned exception;
    __asm__ volatile("mrs %0, ipsr" : "=r"(exception) : : "memory");
    irq_exception_seen = exception;
    irq_mask_seen = Mask::read();
    irq_stage_seen = irq_stage;
    irq_count = irq_count + 1;
    // NVIC clears software pending state on exception entry. No peripheral
    // level source is asserted in this fixture; no invented device ack here.
}

void unexpected_exception() {
    quiesce();
    control_status = 0xf00d;
    while (true) {
        __asm__ volatile("nop" : : : "memory");
    }
}

void irq_mask_probe() {
    control_status = 0x100;
#define CHECK(expression, code) do { if (!(expression)) { quiesce(); control_status = (code); return; } } while (false)
    CHECK(Mask::read() == 1 && irq_count == 0, 1);
    reg(disable) = 1;
    reg(clearPending) = 1;
    reg(enable) = 1;
    __asm__ volatile("dsb\n\tisb" : : : "memory");
    CHECK((reg(enable) & 1) == 1 && (reg(pending) & 1) == 0, 2);
    Mask::restore(0);
    const auto outer = Mask::saveAndDisable();
    const auto inner = Mask::saveAndDisable();
    CHECK(outer == 0 && inner == 1 && Mask::read() == 1, 3);
    irq_stage = 1;
    reg(pending) = 1;
    __asm__ volatile("dsb\n\tisb" : : : "memory");
    CHECK((reg(pending) & 1) == 1, 4);
    settle();
    CHECK(irq_count == 0 && Mask::read() == 1, 5);
    Mask::restore(inner);
    settle();
    CHECK(irq_count == 0 && Mask::read() == 1, 6);
    CHECK((reg(pending) & 1) == 1, 7);
    irq_stage = 2;
    Mask::restore(outer);
    settle();
    CHECK(irq_count == 1 && irq_stage_seen == 2, 8);
    CHECK(irq_mask_seen == 0 && irq_exception_seen == 16, 9);
    CHECK(Mask::read() == 0 && (reg(pending) & 1) == 0, 10);
    settle();
    CHECK(irq_count == 1, 11);
    quiesce();
    CHECK(Mask::read() == 1 && (reg(enable) & 1) == 0, 12);
    control_status = 0x600d;
#undef CHECK
}
}
