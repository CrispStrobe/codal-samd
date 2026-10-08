// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#ifndef BRICKWRIGHT_SIMULATION_SPI_TRANSFER_STATE_H
#define BRICKWRIGHT_SIMULATION_SPI_TRANSFER_STATE_H

#include <cstdint>
#include <limits>

namespace brickwright_simulation {

// Pure state only: the driver owns MMIO, buffers, deadlines and notification.
// Serialize calls in the driver. Tokens are scoped to this object's lifetime.
class SpiTransferState {
public:
    using Token = std::uint64_t;
    enum class Result { success, dma_error, cancelled, timeout };

    SpiTransferState() = default;
    SpiTransferState(const SpiTransferState &) = delete;
    SpiTransferState &operator=(const SpiTransferState &) = delete;

    // RX-only transfers require the driver to supply real TX clocks.
    // A zero-length request is a caller no-op, not a DMA completion.
    bool begin(std::uint32_t txBytes, std::uint32_t rxBytes, Token &token) {
        if (phase_ != Phase::idle || (txBytes == 0 && rxBytes == 0) ||
            (txBytes != 0 && rxBytes != 0 && txBytes != rxBytes) ||
            generation_ == std::numeric_limits<Token>::max())
            return false;
        ++generation_;
        phase_ = Phase::dma;
        token = generation_;
        return true;
    }

    bool dmaComplete(Token token) {
        if (!matches(token) || phase_ != Phase::dma)
            return false;
        phase_ = Phase::draining;
        return true;
    }

    // Call only after observing real TXC for this transfer, including an
    // already-set TXC sampled after dmaComplete. DMA alone is not success.
    bool txComplete(Token token) {
        if (!matches(token) || phase_ != Phase::draining)
            return false;
        finish(Result::success);
        return true;
    }

    bool dmaError(Token token) { return fail(token, Result::dma_error); }
    bool cancel(Token token) { return fail(token, Result::cancelled); }
    bool timeout(Token token) { return fail(token, Result::timeout); }

    // The driver consumes a terminal result before scheduling a new transfer.
    // This is result consumption, not proof of callback or fiber delivery.
    bool take(Token token, Result &result) {
        if (!matches(token) || phase_ != Phase::terminal)
            return false;
        result = result_;
        phase_ = Phase::idle;
        return true;
    }

private:
    enum class Phase { idle, dma, draining, terminal };
    Phase phase_ = Phase::idle;
    Token generation_ = 0;
    Result result_ = Result::cancelled;

    bool matches(Token token) const { return token != 0 && token == generation_; }
    void finish(Result result) { result_ = result; phase_ = Phase::terminal; }
    bool fail(Token token, Result result) {
        if (!matches(token) || (phase_ != Phase::dma && phase_ != Phase::draining))
            return false;
        finish(result);
        return true;
    }
};

} // namespace brickwright_simulation
#endif
