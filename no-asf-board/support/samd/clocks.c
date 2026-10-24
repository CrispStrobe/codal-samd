// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "samd/clocks.h"

static void wait_generator(uint8_t generator)
{
    while (GCLK->SYNCBUSY.reg & (1u << (GCLK_SYNCBUSY_GENCTRL_Pos + generator))) {
    }
}

void enable_clock_generator(uint8_t generator, uint32_t source, uint16_t divisor)
{
    // DIVSEL=0 divides by DIV directly; 0 and 1 both mean undivided.
    GCLK->GENCTRL[generator].reg = GCLK_GENCTRL_SRC(source) | GCLK_GENCTRL_DIV(divisor) | GCLK_GENCTRL_GENEN;
    wait_generator(generator);
}

void disable_clock_generator(uint8_t generator)
{
    GCLK->GENCTRL[generator].reg &= ~GCLK_GENCTRL_GENEN;
    wait_generator(generator);
}

void connect_gclk_to_peripheral(uint8_t generator, uint8_t peripheral)
{
    GCLK->PCHCTRL[peripheral].reg = GCLK_PCHCTRL_GEN(generator) | GCLK_PCHCTRL_CHEN;
    while (!(GCLK->PCHCTRL[peripheral].reg & GCLK_PCHCTRL_CHEN)) {
    }
}

void disconnect_gclk_from_peripheral(uint8_t generator, uint8_t peripheral)
{
    (void)generator;
    GCLK->PCHCTRL[peripheral].reg = 0;
    while (GCLK->PCHCTRL[peripheral].reg & GCLK_PCHCTRL_CHEN) {
    }
}

static void enable_dpll(volatile OscctrlDpll *dpll, uint8_t peripheral, uint16_t ratio)
{
    // Reference: GCLK5 at 1 MHz. Output = 1 MHz * (LDR + 1).
    connect_gclk_to_peripheral(CLK_GEN_1MHZ, peripheral);
    dpll->DPLLRATIO.reg = OSCCTRL_DPLLRATIO_LDR(ratio - 1u) | OSCCTRL_DPLLRATIO_LDRFRAC(0);
    while (dpll->DPLLSYNCBUSY.reg & OSCCTRL_DPLLSYNCBUSY_DPLLRATIO) {
    }
    dpll->DPLLCTRLB.reg = OSCCTRL_DPLLCTRLB_REFCLK_GCLK;
    dpll->DPLLCTRLA.reg = OSCCTRL_DPLLCTRLA_ENABLE;
    while (dpll->DPLLSYNCBUSY.reg & OSCCTRL_DPLLSYNCBUSY_ENABLE) {
    }
    const uint32_t ready = OSCCTRL_DPLLSTATUS_LOCK | OSCCTRL_DPLLSTATUS_CLKRDY;
    while ((dpll->DPLLSTATUS.reg & ready) != ready) {
    }
}

void clock_init(void)
{
    // The DFLL runs open-loop at 48 MHz out of reset; OSCULP32K is always on.
    enable_clock_generator(CLK_GEN_32KHZ, CLOCK_32KHZ, 1);
    enable_clock_generator(CLK_GEN_1MHZ, CLOCK_48MHZ, 48);
    enable_dpll(&OSCCTRL->Dpll[0], OSCCTRL_GCLK_ID_FDPLL0, 120);
    enable_dpll(&OSCCTRL->Dpll[1], OSCCTRL_GCLK_ID_FDPLL1, 100);
    enable_clock_generator(CLK_GEN_120MHZ, CLOCK_120MHZ, 1);
    enable_clock_generator(CLK_GEN_48MHZ, CLOCK_48MHZ, 1);
    enable_clock_generator(CLK_GEN_100MHZ, CLOCK_100MHZ, 1);
    enable_clock_generator(CLK_GEN_8MHZ, CLOCK_48MHZ, 6);
}

uint32_t clock_generator_frequency(uint8_t generator)
{
    static const uint32_t nominal[] = {120000000u, 48000000u, 100000000u, 32768u, 8000000u, 1000000u};
    return generator < sizeof(nominal) / sizeof(nominal[0]) ? nominal[generator] : 0;
}

// TC/TCC bus clocks live in different MCLK masks per instance (DS60001507, MCLK).
static void enable_bus_clock(bool is_tc, uint8_t index)
{
    if (is_tc) {
        switch (index) {
        case 0: MCLK->APBAMASK.reg |= MCLK_APBAMASK_TC0; break;
        case 1: MCLK->APBAMASK.reg |= MCLK_APBAMASK_TC1; break;
        case 2: MCLK->APBBMASK.reg |= MCLK_APBBMASK_TC2; break;
        case 3: MCLK->APBBMASK.reg |= MCLK_APBBMASK_TC3; break;
        }
    } else {
        switch (index) {
        case 0: MCLK->APBBMASK.reg |= MCLK_APBBMASK_TCC0; break;
        case 1: MCLK->APBBMASK.reg |= MCLK_APBBMASK_TCC1; break;
        case 2: MCLK->APBCMASK.reg |= MCLK_APBCMASK_TCC2; break;
        }
    }
}

static const uint8_t tc_gclk[] = {TC0_GCLK_ID, TC1_GCLK_ID, TC2_GCLK_ID, TC3_GCLK_ID};
static const uint8_t tcc_gclk[] = {TCC0_GCLK_ID, TCC1_GCLK_ID, TCC2_GCLK_ID};

void turn_on_clocks(bool is_tc, uint8_t index, uint32_t generator)
{
    enable_bus_clock(is_tc, index);
    connect_gclk_to_peripheral((uint8_t)generator, is_tc ? tc_gclk[index] : tcc_gclk[index]);
}
