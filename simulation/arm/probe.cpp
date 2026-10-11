// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "SpiTransferState.h"

// This is a future authored state-control guest, NOT a peripheral/DMA guest.
extern "C" {
volatile unsigned control_status = 0;
void transfer_state_probe() {
    using State = brickwright_simulation::SpiTransferState;
    State state;
    State::Token oldToken = 0, token = 0;
    State::Result result = State::Result::timeout;
    control_status = 0x100;
#define CHECK(expression, code) do { if (!(expression)) { control_status = (code); return; } } while (false)
    CHECK(state.begin(1, 0, oldToken), 1);
    CHECK(!state.txComplete(oldToken), 2);
    CHECK(state.dmaComplete(oldToken), 3);
    CHECK(!state.take(oldToken, result), 4);
    CHECK(state.txComplete(oldToken), 5);
    CHECK(state.take(oldToken, result) && result == State::Result::success, 6);
    CHECK(!state.take(oldToken, result), 7);
    CHECK(state.begin(1, 1, token) && token != oldToken, 8);
    CHECK(!state.dmaComplete(oldToken) && !state.cancel(oldToken), 9);
    CHECK(state.dmaError(token), 10);
    CHECK(!state.txComplete(token), 11);
    CHECK(state.take(token, result) && result == State::Result::dma_error, 12);
    CHECK(state.begin(1, 0, token), 13);
    CHECK(state.cancel(token), 14);
    CHECK(state.take(token, result) && result == State::Result::cancelled, 15);
    CHECK(state.begin(1, 0, token), 16);
    CHECK(state.dmaComplete(token) && state.timeout(token), 17);
    CHECK(state.take(token, result) && result == State::Result::timeout, 18);
    control_status = 0x600d;
#undef CHECK
}
}
