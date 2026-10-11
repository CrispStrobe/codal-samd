// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Explicit separate paired-irq profile, NOT the default historical ABI selection.
#if !defined(BW_SIMULATION_STRICT_PAIRED_IRQ) || BW_SIMULATION_STRICT_PAIRED_IRQ != 1
#error "Select the strict simulation-only paired IRQ profile explicitly"
#endif
#include "InterruptMask.h"
#include "PairedIrqState.h"

using State = brickwright_simulation::PairedIrqState<>;
using Mask = brickwright_simulation::InterruptMask;
static State state;

extern "C" {
// The integrating scheduler must provide a stable nonzero thread/fiber identity.
// There is deliberately no default stub, weak symbol or fabricated identity.
unsigned bw_simulation_fiber_identity();
volatile unsigned bw_simulation_irq_fault = 0;
}

static State::Owner owner() {
    unsigned exception;
    __asm__ volatile("mrs %0, ipsr" : "=r"(exception) : : "memory");
    return {exception, exception == 0 ? bw_simulation_fiber_identity() : 0};
}

static void apply(State::Transition transition) {
    if (!transition.ok()) {
        // Fault is observable, but not recoverable by a later enable or switch.
        // Fault paths never restore an enabled mask or return invented success.
        bw_simulation_irq_fault = static_cast<unsigned>(transition.fault);
        Mask::restore(1);
        while (true) __asm__ volatile("nop" : : : "memory");
    }
    Mask::restore(transition.restoreMask);
}

extern "C" void target_disable_irq() {
    const unsigned saved = Mask::saveAndDisable();
    const State::Owner context = owner();
    apply(state.enter(saved, context));
}

extern "C" void target_enable_irq() {
    const unsigned saved = Mask::saveAndDisable();
    const State::Owner context = owner();
    apply(state.leave(saved, context));
}

extern "C" bool bw_simulation_irq_may_switch_context() {
    const unsigned saved = Mask::saveAndDisable();
    const bool allowed = state.maySwitchContext();
    Mask::restore(saved);
    return allowed;
}
