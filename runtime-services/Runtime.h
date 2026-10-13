// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#include "LowLevelTimer.h"

namespace brickwright_runtime {
// An explicit modeled Cortex-M clock adapter, not a physical SAMD TC model.
// This profile exclusively owns DWT CYCCNT and SysTick at 120 cycles/us.
class CoreClockTimer final : public codal::LowLevelTimer {
    uint32_t lastCycles = 0, fractionalCycles = 0, timeUs = 0;
    uint32_t compare[2] = {};
    unsigned armed = 0;
    bool enabled = false, irqEnabled = true;
    void rearm();
public:
    CoreClockTimer();
    int enable() override;
    int disable() override;
    int enableIRQ() override;
    int disableIRQ() override;
    int reset() override;
    int setMode(codal::TimerMode) override;
    int setCompare(uint8_t, uint32_t) override;
    int offsetCompare(uint8_t, uint32_t) override;
    int clearCompare(uint8_t) override;
    uint32_t captureCounter() override;
    int setClockSpeed(uint32_t) override;
    int setBitMode(codal::TimerBitMode) override;
    void interrupt();
};
extern CoreClockTimer *clock;
}
extern "C" {
extern volatile unsigned runtime_status, runtime_switches, runtime_restores;
extern volatile unsigned runtime_clock_irqs;
void runtime_probe();
void runtime_systick();
void runtime_fault();
}
