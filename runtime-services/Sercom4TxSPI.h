// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#include "Runtime.h"
#include "SPI.h"
#include "MessageBus.h"
#include "Timer.h"
#include "RuntimeSpiTransferState.h"

namespace brickwright_runtime {
// First modeled PyBadge TX-only profile. Exclusively owns DMAC/channel0,
// SERCOM4, PB7 CS and PB13/PB15 mux. DC belongs to the display caller.
// Not physical clock qualification.
// Keep this object, Timer, bus and asynchronous buffers alive through delivery.
// Zero-length async requests refuse; blocking zero length is a no-op.
class Sercom4TxSPI final : public codal::SPI {
public:
    using State = RuntimeSpiTransferState;
    using FailureHandler = void (*)(void *, State::Result);
    Sercom4TxSPI(codal::Timer &, codal::MessageBus &);
    ~Sercom4TxSPI() override;
    Sercom4TxSPI(const Sercom4TxSPI &) = delete;
    Sercom4TxSPI &operator=(const Sercom4TxSPI &) = delete;
    int init();
    int setFrequency(uint32_t) override;
    int setMode(int mode, int bits = 8) override;
    int write(int) override; // RX response not qualified in this first slice.
    int transfer(const uint8_t *, uint32_t, uint8_t *, uint32_t) override;
    int startTransfer(const uint8_t *, uint32_t, uint8_t *, uint32_t,
                      codal::PVoidCallback, void *) override;
    int setFailureHandler(FailureHandler);
    int setTimeoutUs(uint32_t);
    int cancel();
    void dmaInterrupt();
    void txcInterrupt();
private:
    codal::Timer &timer_;
    codal::MessageBus &bus_;
    State state_;
    State::Token token_ = 0;
    uint16_t event_ = 0;
    uint8_t baud_ = 255;
    uint32_t timeoutUs_ = 1000000;
    bool initialized_ = false, blocking_ = false, delivering_ = false;
    int result_ = DEVICE_BUSY;
    codal::PVoidCallback callback_ = nullptr;
    FailureHandler failure_ = nullptr;
    void *argument_ = nullptr;
    int begin(const uint8_t *, uint32_t, uint8_t *, uint32_t,
              codal::PVoidCallback, void *, bool);
    void arm(const uint8_t *, uint32_t);
    void finish(State::Result);
    void timeout(codal::Event);
};
}
extern "C" void runtime_dma_irq();
extern "C" void runtime_spi_irq();
