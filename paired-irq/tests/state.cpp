// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "PairedIrqState.h"
#include <initializer_list>

using State = brickwright_simulation::PairedIrqState<2>;
using Fault = State::Fault;
static const State::Owner thread = {0, 1}, other = {0, 2}, irq = {16, 0};
#define CHECK(condition) do { if (!(condition)) return __LINE__; } while (false)

int main() {
    // Disabled entry first: the always-enable mutant must fail restoration
    // here before the later IRQ-owner check. Both entry masks remain covered.
    for (unsigned entry : {1u, 0u}) {
        State state;
        CHECK(state.maySwitchContext());
        CHECK(state.enter(entry, thread).restoreMask == 1);
        CHECK(!state.maySwitchContext() && state.depth() == 1);
        CHECK(state.enter(1, thread).ok());
        CHECK(state.leave(1, thread).restoreMask == 1);
        auto final = state.leave(1, thread);
        CHECK(final.ok() && final.restoreMask == entry && state.depth() == 0);
        CHECK(state.maySwitchContext());
        CHECK(state.enter(1, irq).ok());
        CHECK(state.leave(1, irq).restoreMask == 1);
    }
    {
        State state;
        auto result = state.leave(0, thread);
        CHECK(result.fault == Fault::underflow && result.restoreMask == 1);
        CHECK(!state.maySwitchContext());
        CHECK(state.enter(0, thread).fault == Fault::underflow);
    }
    {
        State state;
        CHECK(state.enter(0, thread).ok());
        CHECK(state.enter(1, thread).ok());
        CHECK(state.enter(1, thread).fault == Fault::depth_limit);
        CHECK(state.depth() == 2 && state.leave(1, thread).restoreMask == 1);
    }
    for (bool entering : {false, true}) {
        State state;
        CHECK(state.enter(0, thread).ok());
        auto result = entering ? state.enter(1, other) : state.leave(1, other);
        CHECK(result.fault == Fault::owner_changed && result.restoreMask == 1);
        CHECK(state.depth() == 1 && state.leave(1, thread).fault == Fault::owner_changed);
    }
    for (bool entering : {false, true}) {
        State state;
        CHECK(state.enter(0, thread).ok());
        auto result = entering ? state.enter(0, thread) : state.leave(0, thread);
        CHECK(result.fault == Fault::mask_lost && result.restoreMask == 1);
    }
    for (const auto bad : {State::Owner{0, 0}, State::Owner{2, 0}, State::Owner{3, 0},
                           State::Owner{7, 0}, State::Owner{13, 0},
                           State::Owner{16, 1}, State::Owner{256, 0}}) {
        State state;
        CHECK(state.enter(1, bad).fault == Fault::invalid_owner);
        CHECK(state.leave(1, thread).restoreMask == 1);
    }
    {
        State state;
        CHECK(state.enter(2, thread).fault == Fault::invalid_mask);
        CHECK(state.enter(0, thread).restoreMask == 1);
    }
    return 0;
}
