// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "SpiTransferState.h"
#include <cassert>
#include <initializer_list>
#include <iostream>

using State = brickwright_simulation::SpiTransferState;
using Result = State::Result;

int main() {
    State s;
    State::Token a = 123, b = 0;
    Result result = Result::timeout;
    assert(!s.begin(0, 0, a) && a == 123);
    assert(!s.begin(2, 3, a) && a == 123);
    assert(!s.dmaComplete(0) && !s.txComplete(0));
    assert(!s.take(0, result) && result == Result::timeout);

    assert(s.begin(1, 0, a));
    assert(!s.begin(1, 0, b) && b == 0);
    assert(!s.txComplete(a));
    assert(!s.take(a, result));
    assert(s.dmaComplete(a));
    assert(!s.dmaComplete(a));
    assert(!s.take(a, result));
    assert(s.txComplete(a));
    assert(!s.txComplete(a) && !s.dmaError(a));
    assert(!s.cancel(a) && !s.timeout(a));
    assert(!s.begin(1, 0, b));
    assert(s.take(a, result) && result == Result::success);
    assert(!s.take(a, result));

    assert(s.begin(0, 4, b) && b != a);
    assert(!s.dmaComplete(a) && !s.txComplete(a));
    assert(!s.dmaError(a) && !s.cancel(a) && !s.timeout(a));
    assert(!s.take(a, result));
    assert(s.dmaError(b));
    assert(!s.dmaComplete(b) && !s.txComplete(b));
    assert(s.take(b, result) && result == Result::dma_error);

    assert(s.begin(4, 4, a));
    assert(s.dmaComplete(a));
    assert(s.dmaError(a));
    assert(!s.txComplete(a));
    assert(s.take(a, result) && result == Result::dma_error);

    for (bool afterDma : {false, true}) {
        assert(s.begin(2, 0, a));
        if (afterDma) assert(s.dmaComplete(a));
        assert(s.cancel(a));
        assert(!s.txComplete(a) && !s.dmaComplete(a));
        assert(s.take(a, result) && result == Result::cancelled);
        assert(s.begin(2, 0, a));
        if (afterDma) assert(s.dmaComplete(a));
        assert(s.timeout(a));
        assert(!s.txComplete(a));
        assert(s.take(a, result) && result == Result::timeout);
    }
    std::cout << "SPI transfer state controls PASS\n";
}
