// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#ifndef BRICKWRIGHT_SIMULATION_INTERRUPT_MASK_H
#define BRICKWRIGHT_SIMULATION_INTERRUPT_MASK_H

#if !defined(__ARM_ARCH_7EM__) || !defined(__thumb__) || \
    !(defined(__GNUC__) || defined(__clang__))
#error "InterruptMask requires privileged ARMv7E-M Thumb with GCC-compatible assembly"
#endif

namespace brickwright_simulation {

// A same-core privileged critical-section primitive, not a DMA memory barrier.
// Callers must restore saved masks in reverse order on the same execution context.
class InterruptMask {
public:
    using Saved = unsigned;

    static Saved read() {
        Saved value;
        __asm__ volatile("mrs %0, primask" : "=r"(value) : : "memory");
        return value;
    }

    static Saved saveAndDisable() {
        Saved value;
        __asm__ volatile("mrs %0, primask\n\tcpsid i" : "=r"(value) : : "memory");
        return value;
    }

    // Only pass a value returned by read/saveAndDisable in this context.
    static void restore(Saved saved) {
        __asm__ volatile("msr primask, %0" : : "r"(saved) : "memory");
    }
};

} // namespace brickwright_simulation
#endif
