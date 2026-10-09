// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "InterruptMask.h"

// Bounded PRIMASK state control; no peripheral, NVIC or interrupt-delivery test.
extern "C" {
volatile unsigned control_status = 0;
void interrupt_mask_probe() {
    using Mask = brickwright_simulation::InterruptMask;
    control_status = 0x100;
    // Every exit restores the diagnostic's disabled-interrupt boundary.
#define CHECK(expression, code) do { if (!(expression)) { Mask::restore(1); control_status = (code); return; } } while (false)
    CHECK(Mask::read() == 1, 1);
    const auto disabled = Mask::saveAndDisable();
    CHECK(disabled == 1 && Mask::read() == 1, 2);
    Mask::restore(disabled);
    CHECK(Mask::read() == 1, 3);

    // There are no enabled NVIC sources in this authored diagnostic profile.
    Mask::restore(0);
    CHECK(Mask::read() == 0, 4);
    const auto outer = Mask::saveAndDisable();
    CHECK(outer == 0 && Mask::read() == 1, 5);
    const auto inner = Mask::saveAndDisable();
    CHECK(inner == 1 && Mask::read() == 1, 6);
    Mask::restore(inner);
    CHECK(Mask::read() == 1, 7);
    Mask::restore(outer);
    CHECK(Mask::read() == 0, 8);
    Mask::restore(1);
    CHECK(Mask::read() == 1, 9);
    control_status = 0x600d;
#undef CHECK
}
}
