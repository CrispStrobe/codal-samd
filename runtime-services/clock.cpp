// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "Runtime.h"
#include "InterruptMask.h"
#include "codal_target_hal.h"

using Mask = brickwright_simulation::InterruptMask;
namespace brickwright_runtime {
CoreClockTimer *clock;
static volatile uint32_t &reg(uint32_t address) {
    return *reinterpret_cast<volatile uint32_t *>(address);
}
CoreClockTimer::CoreClockTimer() : LowLevelTimer(2) {
    if (clock) target_panic(0xc01);
    clock = this;
    bitMode = codal::BitMode32;
    reg(0xe0001000) = 1;
    lastCycles = reg(0xe0001004);
}
uint32_t CoreClockTimer::captureCounter() {
    const unsigned saved = Mask::saveAndDisable();
    const uint32_t now = reg(0xe0001004);
    const uint32_t elapsed = now - lastCycles;
    lastCycles = now;
    const uint64_t cycles = uint64_t(elapsed) + fractionalCycles;
    timeUs += uint32_t(cycles / 120);
    fractionalCycles = uint32_t(cycles % 120);
    Mask::restore(saved);
    return timeUs;
}
void CoreClockTimer::rearm() {
    reg(0xe000e010) = 0;
    if (!enabled || !irqEnabled) return;
    const uint32_t now = captureCounter();
    // Also service the DWT wrap extension at least every 100ms. No fabricated
    // time or callback comes from that service: only actual register deltas.
    uint32_t delay = 100000;
    for (unsigned c = 0; c < 2; ++c) {
        if (armed & (1u << c)) {
            const int32_t distance = int32_t(compare[c] - now);
            const uint32_t until = distance <= 0 ? 10 : uint32_t(distance);
            if (until < delay) delay = until;
        }
    }
    // The bounded profile uses actual core-clock SysTick exception 15.
    reg(0xe000e014) = delay * 120 - 1;
    reg(0xe000e018) = 0;
    reg(0xe000e010) = 7;
}
int CoreClockTimer::enable() {
    const unsigned saved = Mask::saveAndDisable();
    enabled = true;
    reg(0xe0001000) = 1;
    lastCycles = reg(0xe0001004);
    rearm();
    Mask::restore(saved);
    return DEVICE_OK;
}
int CoreClockTimer::disable() {
    const unsigned saved = Mask::saveAndDisable();
    captureCounter();
    enabled = false;
    reg(0xe0001000) = 0;
    rearm();
    Mask::restore(saved);
    return DEVICE_OK;
}
int CoreClockTimer::enableIRQ() {
    const unsigned saved = Mask::saveAndDisable();
    irqEnabled = true;
    rearm();
    Mask::restore(saved);
    return DEVICE_OK;
}
int CoreClockTimer::disableIRQ() {
    const unsigned saved = Mask::saveAndDisable();
    irqEnabled = false;
    rearm();
    Mask::restore(saved);
    return DEVICE_OK;
}
int CoreClockTimer::reset() { return DEVICE_NOT_SUPPORTED; }
int CoreClockTimer::setMode(codal::TimerMode) { return DEVICE_NOT_SUPPORTED; }
int CoreClockTimer::setClockSpeed(uint32_t speed) {
    return speed == 1000 ? DEVICE_OK : DEVICE_NOT_SUPPORTED;
}
int CoreClockTimer::setBitMode(codal::TimerBitMode mode) {
    return mode == codal::BitMode32 ? DEVICE_OK : DEVICE_NOT_SUPPORTED;
}
int CoreClockTimer::setCompare(uint8_t channel, uint32_t value) {
    if (channel >= 2) return DEVICE_INVALID_PARAMETER;
    const unsigned saved = Mask::saveAndDisable();
    compare[channel] = value;
    armed |= 1u << channel;
    rearm();
    Mask::restore(saved);
    return DEVICE_OK;
}
int CoreClockTimer::offsetCompare(uint8_t channel, uint32_t value) {
    if (channel >= 2) return DEVICE_INVALID_PARAMETER;
    const unsigned saved = Mask::saveAndDisable();
    if (!(armed & (1u << channel))) {
        Mask::restore(saved);
        return DEVICE_INVALID_PARAMETER;
    }
    compare[channel] += value;
    rearm();
    Mask::restore(saved);
    return DEVICE_OK;
}
int CoreClockTimer::clearCompare(uint8_t channel) {
    if (channel >= 2) return DEVICE_INVALID_PARAMETER;
    const unsigned saved = Mask::saveAndDisable();
    armed &= ~(1u << channel);
    rearm();
    Mask::restore(saved);
    return DEVICE_OK;
}
void CoreClockTimer::interrupt() {
    ++runtime_clock_irqs;
    // A real exception can already be pending when the caller disables its
    // source. Retire that entry without consuming deadlines or firing events.
    if (!enabled || !irqEnabled) return;
    const uint32_t now = captureCounter();
    unsigned due = 0;
    for (unsigned c = 0; c < 2; ++c)
        if ((armed & (1u << c)) && int32_t(now - compare[c]) >= 0) due |= 1u << c;
    armed &= ~due;
    if (due && timer_pointer) timer_pointer(uint16_t(due));
    rearm();
}
}
extern "C" void runtime_systick() {
    if (!brickwright_runtime::clock) target_panic(0xc02);
    brickwright_runtime::clock->interrupt();
}
