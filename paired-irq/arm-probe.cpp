// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Authored ABI control only; identity changes below are NOT actual fiber switches.
#include "InterruptMask.h"

#ifndef BW_PAIRED_IRQ_ADVERSARY
#define BW_PAIRED_IRQ_ADVERSARY 0
#endif
#if BW_PAIRED_IRQ_ADVERSARY < 0 || BW_PAIRED_IRQ_ADVERSARY > 4
#error "Unknown authored paired IRQ adversary"
#endif

using Mask = brickwright_simulation::InterruptMask;
extern "C" {
void target_disable_irq();
void target_enable_irq();
bool bw_simulation_irq_may_switch_context();
extern volatile unsigned bw_simulation_irq_fault;
volatile unsigned control_status = 0;
volatile unsigned irq_count = 0, irq_stage = 0, irq_stage_seen = 0;
volatile unsigned irq_mask_seen = 0, irq_exception_seen = 0;
volatile unsigned fixture_identity = 1;
unsigned bw_simulation_fiber_identity() { return fixture_identity; }
}

static volatile unsigned& reg(unsigned address) {
    return *reinterpret_cast<volatile unsigned*>(address);
}
static const unsigned disable = 0xe000e180, clearPending = 0xe000e280;
#if BW_PAIRED_IRQ_ADVERSARY == 0
static const unsigned enable = 0xe000e100, pending = 0xe000e200;
static void settle() {
    for (unsigned i = 0; i < 32; ++i) __asm__ volatile("nop" : : : "memory");
}
#endif
static void quiesce() {
    Mask::restore(1);
    reg(disable) = 1;
    reg(clearPending) = 1;
    __asm__ volatile("dsb\n\tisb" : : : "memory");
}

extern "C" void authored_irq0() {
    unsigned exception;
    __asm__ volatile("mrs %0, ipsr" : "=r"(exception) : : "memory");
    irq_exception_seen = exception;
    irq_stage_seen = irq_stage;
    target_disable_irq();
    target_enable_irq();
    irq_mask_seen = Mask::read();
    irq_count = irq_count + 1;
}

extern "C" void unexpected_exception() {
    quiesce();
    control_status = 0xf00d;
    while (true) __asm__ volatile("nop" : : : "memory");
}

extern "C" void irq_mask_probe() {
    control_status = 0x100;
#define CHECK(condition, code) do { if (!(condition)) { quiesce(); control_status = code; return; } } while (false)
    CHECK(Mask::read() == 1 && irq_count == 0 && bw_simulation_irq_fault == 0, 1);
    CHECK(bw_simulation_irq_may_switch_context(), 2);
#if BW_PAIRED_IRQ_ADVERSARY != 0
    control_status = 0x200 + BW_PAIRED_IRQ_ADVERSARY;
#if BW_PAIRED_IRQ_ADVERSARY == 1
    target_enable_irq(); // Unmatched enable must latch underflow3 and never return.
#elif BW_PAIRED_IRQ_ADVERSARY == 2
    target_disable_irq();
    fixture_identity = 2;
    target_enable_irq(); // Wrong fixture identity must latch owner_changed5.
#elif BW_PAIRED_IRQ_ADVERSARY == 3
    for (unsigned i = 0; i < 65; ++i) target_disable_irq(); // depth_limit4.
#elif BW_PAIRED_IRQ_ADVERSARY == 4
    target_disable_irq();
    Mask::restore(0);
    target_enable_irq(); // Deliberately lost PRIMASK must latch mask_lost6.
#endif
    control_status = 0xbad; // Returning from a refusal is never passing evidence.
    quiesce();
#else
    reg(disable) = 1;
    reg(clearPending) = 1;
    // Already-disabled outer entry: final leave must NOT invent an enable.
    target_disable_irq();
    CHECK(!bw_simulation_irq_may_switch_context(), 3);
    target_disable_irq();
    target_enable_irq();
    CHECK(Mask::read() == 1, 4);
    target_enable_irq();
    CHECK(Mask::read() == 1 && bw_simulation_irq_may_switch_context(), 5);
    // Next logical identity is permitted only after the paired depth is zero.
    fixture_identity = 2;
    reg(enable) = 1;
    Mask::restore(0);
    target_disable_irq();
    target_disable_irq();
    irq_stage = 1;
    reg(pending) = 1;
    __asm__ volatile("dsb\n\tisb" : : : "memory");
    settle();
    CHECK(irq_count == 0 && Mask::read() == 1, 6);
    target_enable_irq();
    settle();
    CHECK(irq_count == 0 && Mask::read() == 1 && (reg(pending) & 1), 7);
    irq_stage = 2;
    target_enable_irq();
    settle();
    CHECK(irq_count == 1 && irq_stage_seen == 2, 8);
    CHECK(irq_mask_seen == 0 && irq_exception_seen == 16, 9);
    CHECK(Mask::read() == 0 && bw_simulation_irq_may_switch_context(), 10);
    CHECK((reg(pending) & 1) == 0 && bw_simulation_irq_fault == 0, 11);
    quiesce();
    control_status = 0x600d;
#endif
#undef CHECK
}
