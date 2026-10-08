// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "SpiTransferState.h"
#include <cassert>
#include <cstdint>
#include <iostream>

using State = brickwright_simulation::SpiTransferState;
using Result = State::Result;

// Table oracle, not the implementation's branch structure. Same author.
// Columns: DMA, TXC, error, cancel, timeout, consume. -1 means reject.
static const int edges[4][6] = {
    {-1, -1, -1, -1, -1, -1}, // idle
    { 2, -1,  3,  3,  3, -1}, // DMA in progress
    {-1,  3,  3,  3,  3, -1}, // waiting for SPI TXC
    {-1, -1, -1, -1, -1,  0}, // unconsumed terminal result
};

int main() {
    // Every shorter trace is also a prefix of a length-six trace.
    for (std::uint32_t trace = 0; trace < 1000000; ++trace) {
        State actual;
        State::Token current = 0;
        std::uint64_t issued = 0;
        int phase = 0;
        Result terminal = Result::cancelled;
        std::uint32_t remaining = trace;
        for (int step = 0; step < 6; ++step) {
            const unsigned action = remaining % 10;
            remaining /= 10;
            if (action < 2) {
                State::Token output = 999;
                const bool accepted = actual.begin(action == 0 ? 4 : 0, 4, output);
                assert(accepted == (phase == 0));
                if (accepted) {
                    ++issued;
                    assert(output == issued);
                    current = output;
                    phase = 1;
                } else {
                    assert(output == 999);
                }
            } else if (action == 2) {
                State::Token output = 999;
                assert(!actual.begin(0, 0, output) && output == 999);
                assert(!actual.begin(2, 3, output) && output == 999);
            } else if (action == 9) {
                const State::Token stale = current == 0 ? 0 : current - 1;
                Result output = Result::timeout;
                assert(!actual.dmaComplete(stale));
                assert(!actual.txComplete(stale));
                assert(!actual.dmaError(stale));
                assert(!actual.cancel(stale));
                assert(!actual.timeout(stale));
                assert(!actual.take(stale, output) && output == Result::timeout);
            } else {
                const unsigned column = action - 3;
                const int next = edges[phase][column];
                Result output = Result::timeout;
                bool accepted = false;
                switch (column) {
                case 0: accepted = actual.dmaComplete(current); break;
                case 1: accepted = actual.txComplete(current); break;
                case 2: accepted = actual.dmaError(current); break;
                case 3: accepted = actual.cancel(current); break;
                case 4: accepted = actual.timeout(current); break;
                case 5: accepted = actual.take(current, output); break;
                }
                assert(accepted == (next != -1));
                if (column == 5)
                    assert(output == (accepted ? terminal : Result::timeout));
                if (accepted) {
                    if (column == 1) terminal = Result::success;
                    if (column == 2) terminal = Result::dma_error;
                    if (column == 3) terminal = Result::cancelled;
                    if (column == 4) terminal = Result::timeout;
                    phase = next;
                }
            }
        }
    }
    std::cout << "1000000 length-six SPI state traces PASS\n";
}
