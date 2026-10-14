// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "Sercom4TxSPI.h"
#include "CodalFiber.h"
#include "InterruptMask.h"

namespace brickwright_runtime {
using Mask = brickwright_simulation::InterruptMask;
static constexpr uint32_t DMA = 0x4100a000, SPI_BASE = 0x43000000;
static constexpr uint32_t PORT = 0x41008080, AHB = 0x40000810, APBD = 0x40000820;
static constexpr uint32_t GCLK = 0x40001c00u + 0x80 + 34 * 4;
static constexpr uint32_t MASTER = (3u << 2) | (2u << 16);
static constexpr uint32_t TX = (13u << 8) | (2u << 20);
static constexpr uint16_t TIMEOUT_SOURCE = 0x7ffb;
struct alignas(16) Descriptor {
    volatile uint16_t control, count;
    volatile uint32_t source, destination, next;
};
static_assert(sizeof(Descriptor) == 16, "DMAC descriptor ABI");
static Descriptor table, writeback;
static Sercom4TxSPI *instance;
static volatile uint32_t &word(uint32_t address) {
    return *reinterpret_cast<volatile uint32_t *>(address);
}
static volatile uint8_t &byte(uint32_t address) {
    return *reinterpret_cast<volatile uint8_t *>(address);
}
static void barrier() { __asm__ volatile("dmb" : : : "memory"); }

Sercom4TxSPI::Sercom4TxSPI(codal::Timer &timer, codal::MessageBus &bus)
    : timer_(timer), bus_(bus) {
    const unsigned saved = Mask::saveAndDisable();
    if (instance) target_panic(DEVICE_HARDWARE_CONFIGURATION_ERROR);
    event_ = codal::allocateNotifyEvent();
    if (bus_.listen(TIMEOUT_SOURCE, event_, this, &Sercom4TxSPI::timeout,
                    MESSAGE_BUS_LISTENER_IMMEDIATE) != DEVICE_OK)
        target_panic(0xc41);
    instance = this;
    Mask::restore(saved);
}
Sercom4TxSPI::~Sercom4TxSPI() {
    const unsigned saved = Mask::saveAndDisable();
    if (token_ || delivering_) target_panic(0xc42);
    if (initialized_) {
        word(0xe000e180) = 1u << 31;
        word(0xe000e184) = 1u << 31;
        byte(SPI_BASE + 0x14) = 0xff;
        byte(DMA + 0x4c) = 7;
    }
    bus_.ignore(TIMEOUT_SOURCE, event_, this, &Sercom4TxSPI::timeout);
    instance = nullptr;
    Mask::restore(saved);
}
int Sercom4TxSPI::init() {
    const unsigned saved = Mask::saveAndDisable();
    if (token_ || delivering_) { Mask::restore(saved); return DEVICE_BUSY; }
    if (!initialized_) {
        word(AHB) |= 1u << 9;
        word(APBD) |= 1;
        word(GCLK) = 1u << 6;
        byte(PORT + 0x36) = (byte(PORT + 0x36) & 0x0f) | 0x20;
        byte(PORT + 0x37) = (byte(PORT + 0x37) & 0x0f) | 0x20;
        byte(PORT + 0x4d) |= 1;
        byte(PORT + 0x4f) |= 1;
        word(PORT + 0x08) = 1u << 7;
        word(PORT + 0x18) = 1u << 7;
        word(SPI_BASE) = MASTER;
        word(SPI_BASE + 4) = 0;
        byte(SPI_BASE + 0x0c) = baud_;
        byte(SPI_BASE + 0x14) = 0xff;
        byte(SPI_BASE + 0x18) = 0x82;
        word(SPI_BASE) = MASTER | 2;
        // Cancellation/error disables the channel but leaves the controller
        // enabled. SWRST requires global disable, including recovery init.
        word(DMA) = 0;
        word(DMA) = 1; // Exclusive controller reset in this named profile.
        word(DMA + 0x34) = reinterpret_cast<uintptr_t>(&table);
        word(DMA + 0x38) = reinterpret_cast<uintptr_t>(&writeback);
        word(DMA) = 0x102;
        word(DMA + 0x40) = TX;
        byte(DMA + 0x4c) = 7;
        byte(DMA + 0x4e) = 7;
        word(0xe000e100) = 1u << 31;
        word(0xe000e104) = 1u << 31;
        initialized_ = true;
    }
    const bool ready = (word(AHB) & (1u << 9)) && (word(APBD) & 1) &&
        (word(GCLK) & (1u << 6)) && word(SPI_BASE) == (MASTER | 2) &&
        byte(SPI_BASE + 0x0c) == baud_ && word(DMA + 0x34) == reinterpret_cast<uintptr_t>(&table) &&
        word(DMA + 0x38) == reinterpret_cast<uintptr_t>(&writeback);
    Mask::restore(saved);
    return ready ? DEVICE_OK : DEVICE_SPI_ERROR;
}
int Sercom4TxSPI::setFrequency(uint32_t hz) {
    if (hz < 234375 || hz > 60000000) return DEVICE_NOT_SUPPORTED;
    const unsigned saved = Mask::saveAndDisable();
    if (token_ || delivering_) { Mask::restore(saved); return DEVICE_BUSY; }
    baud_ = uint8_t((120000000u + 2 * hz - 1) / (2 * hz) - 1);
    if (initialized_) {
        word(SPI_BASE) = MASTER;
        byte(SPI_BASE + 0x0c) = baud_;
        word(SPI_BASE) = MASTER | 2;
    }
    Mask::restore(saved);
    return DEVICE_OK;
}
int Sercom4TxSPI::setMode(int mode, int bits) {
    if (mode != 0 || bits != 8) return DEVICE_NOT_SUPPORTED;
    const unsigned saved = Mask::saveAndDisable();
    const int result = token_ || delivering_ ? DEVICE_BUSY : DEVICE_OK;
    Mask::restore(saved);
    return result;
}
int Sercom4TxSPI::write(int) { return DEVICE_NOT_SUPPORTED; }
int Sercom4TxSPI::setFailureHandler(FailureHandler handler) {
    const unsigned saved = Mask::saveAndDisable();
    if (token_ || delivering_) { Mask::restore(saved); return DEVICE_BUSY; }
    failure_ = handler;
    Mask::restore(saved);
    return DEVICE_OK;
}
int Sercom4TxSPI::setTimeoutUs(uint32_t value) {
    if (!value || value > 2000000) return DEVICE_INVALID_PARAMETER;
    const unsigned saved = Mask::saveAndDisable();
    if (token_ || delivering_) { Mask::restore(saved); return DEVICE_BUSY; }
    timeoutUs_ = value;
    Mask::restore(saved);
    return DEVICE_OK;
}
void Sercom4TxSPI::arm(const uint8_t *buffer, uint32_t size) {
    byte(SPI_BASE + 0x14) = 2;
    byte(SPI_BASE + 0x18) = 0x82;
    byte(DMA + 0x4e) = 7;
    table.control = 0x0409;
    table.count = uint16_t(size);
    table.source = reinterpret_cast<uintptr_t>(buffer) + size;
    table.destination = SPI_BASE + 0x28;
    table.next = 0;
    writeback.control = 0;
    writeback.count = 0xffff;
    writeback.source = writeback.destination = writeback.next = 0;
    word(PORT + 0x14) = 1u << 7;
    barrier();
    byte(DMA + 0x4d) = 3;
    word(DMA + 0x40) = TX | 2; // Hardware DRE request, never software trigger.
}
int Sercom4TxSPI::begin(const uint8_t *buffer, uint32_t size, uint8_t *rx,
                      uint32_t rxSize, codal::PVoidCallback callback,
                      void *argument, bool blocking) {
    if (rx || rxSize) return DEVICE_NOT_SUPPORTED;
    // Blocking zero bytes is a no-op. Do not silently accept an asynchronous
    // request whose promised callback would never be delivered. This initial
    // TX-only profile explicitly refuses that request before reserving state.
    if (!size) return blocking ? DEVICE_OK : DEVICE_INVALID_PARAMETER;
    const uintptr_t address = reinterpret_cast<uintptr_t>(buffer);
    const bool ram = address >= 0x20000000 && address < 0x2002e000 && size <= 0x2002e000-address;
    // The selected modeled byte-DMA profile admits primary RAM only. Refuse
    // flash here rather than accepting a request the controller cannot service.
    if (!buffer || size > 65535 || !ram) return DEVICE_INVALID_PARAMETER;
    unsigned exception;
    __asm__ volatile("mrs %0, ipsr" : "=r"(exception));
    if (blocking && (exception || Mask::read() || !codal::fiber_scheduler_running()))
        return DEVICE_NOT_SUPPORTED;
    const int ready = init();
    if (ready != DEVICE_OK) return ready;
    const unsigned saved = Mask::saveAndDisable();
    if (token_ || delivering_ || !state_.begin(size, 0, token_)) {
        Mask::restore(saved); return DEVICE_BUSY;
    }
    const int deadline = timer_.eventAfterUs(timeoutUs_, TIMEOUT_SOURCE, event_);
    int enrolled = DEVICE_OK;
    if (deadline == DEVICE_OK && blocking)
        enrolled = codal::fiber_wake_on_event(DEVICE_ID_NOTIFY, event_);
    if (deadline != DEVICE_OK || enrolled != DEVICE_OK) {
        timer_.cancel(TIMEOUT_SOURCE, event_);
        state_.cancel(token_);
        State::Result ignored;
        state_.take(token_, ignored);
        token_ = 0;
        Mask::restore(saved);
        return deadline != DEVICE_OK ? deadline : enrolled;
    }
    blocking_ = blocking;
    result_ = DEVICE_BUSY;
    callback_ = callback;
    argument_ = argument;
    arm(buffer, size);
    Mask::restore(saved);
    return DEVICE_OK;
}
int Sercom4TxSPI::startTransfer(const uint8_t *tx, uint32_t size, uint8_t *rx,
                              uint32_t rxSize, codal::PVoidCallback done, void *arg) {
    return begin(tx, size, rx, rxSize, done, arg, false);
}
int Sercom4TxSPI::transfer(const uint8_t *tx, uint32_t size, uint8_t *rx, uint32_t rxSize) {
    const int started = begin(tx, size, rx, rxSize, nullptr, nullptr, true);
    if (started != DEVICE_OK || !size) return started;
    codal::schedule();
    const unsigned saved = Mask::saveAndDisable();
    State::Result terminal;
    if (!blocking_ || !state_.take(token_, terminal)) target_panic(0xc43);
    token_ = 0;
    blocking_ = false;
    const int result = result_;
    Mask::restore(saved);
    return result;
}
void Sercom4TxSPI::finish(State::Result outcome) {
    const unsigned saved = Mask::saveAndDisable();
    word(DMA + 0x40) = TX;
    byte(DMA + 0x4c) = 7;
    if (outcome != State::Result::success) {
        // This named profile exclusively owns SERCOM4 and its clock controls.
        // Gated MMIO writes are dropped. Release GPIO CS before temporarily
        // restoring clocks so a pending frame cannot reach a selected slave.
        const uint32_t previousApbd = word(APBD), previousGclk = word(GCLK);
        word(PORT + 0x18) = 1u << 7;
        word(APBD) = previousApbd | 1;
        word(GCLK) = previousGclk | (1u << 6);
        byte(SPI_BASE + 0x14) = 0xff;
        word(SPI_BASE) = MASTER;
        byte(SPI_BASE + 0x18) = 0x82;
        word(GCLK) = previousGclk;
        word(APBD) = previousApbd;
        initialized_ = false;
    } else {
        byte(SPI_BASE + 0x14) = 2;
        word(PORT + 0x18) = 1u << 7;
    }
    Mask::restore(saved);
    timer_.cancel(TIMEOUT_SOURCE, event_);
    result_ = outcome == State::Result::success ? DEVICE_OK : DEVICE_SPI_ERROR;
    if (blocking_) { codal::Event(DEVICE_ID_NOTIFY_ONE, event_); return; }
    State::Result consumed;
    if (!state_.take(token_, consumed) || consumed != outcome) target_panic(0xc45);
    token_ = 0;
    const auto callback = callback_;
    const auto failure = failure_;
    void *argument = argument_;
    callback_ = nullptr;
    argument_ = nullptr;
    delivering_ = true;
    if (outcome == State::Result::success) {
        if (callback) callback(argument);
    } else if (failure) failure(argument, outcome);
    else target_panic(0xc44); // Explicit failure, never a success callback.
    delivering_ = false;
}
void Sercom4TxSPI::dmaInterrupt() {
    const uint8_t flags = byte(DMA + 0x4e);
    byte(DMA + 0x4e) = flags;
    if (!token_) return;
    barrier();
    // CHINTFLAG: TERR bit 0, TCMPL bit 1. Error wins if both are set.
    if ((flags & 1) || ((flags & 2) && writeback.count != 0)) {
        if (state_.dmaError(token_)) finish(State::Result::dma_error);
    } else if ((flags & 2) && state_.dmaComplete(token_)) {
        byte(SPI_BASE + 0x16) = 2; // Existing TXC also produces its real split IRQ.
    }
}
void Sercom4TxSPI::txcInterrupt() {
    if (token_ && (byte(SPI_BASE + 0x18) & 2) && state_.txComplete(token_))
        finish(State::Result::success);
}
void Sercom4TxSPI::timeout(codal::Event evt) {
    if (evt.source == TIMEOUT_SOURCE && evt.value == event_ && token_ && state_.timeout(token_))
        finish(State::Result::timeout);
}
int Sercom4TxSPI::cancel() {
    const unsigned saved = Mask::saveAndDisable();
    if (!token_ || !state_.cancel(token_)) { Mask::restore(saved); return DEVICE_INVALID_PARAMETER; }
    finish(State::Result::cancelled);
    Mask::restore(saved);
    return DEVICE_OK;
}
}
extern "C" void runtime_dma_irq() {
    if (brickwright_runtime::instance) brickwright_runtime::instance->dmaInterrupt();
    else target_panic(0xc46);
}
extern "C" void runtime_spi_irq() {
    if (brickwright_runtime::instance) brickwright_runtime::instance->txcInterrupt();
    else target_panic(0xc47);
}
