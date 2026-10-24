// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "samd/timers.h"

Tc *const tc_insts[TC_INST_NUM] = TC_INSTS;
Tcc *const tcc_insts[TCC_INST_NUM] = TCC_INSTS;
const uint8_t tcc_cc_num[TCC_INST_NUM] = {TCC0_CC_NUM, TCC1_CC_NUM, TCC2_CC_NUM};
const uint16_t prescaler[8] = {1, 2, 4, 8, 16, 64, 256, 1024};

static void (*tc_handler)(uint8_t);
static void (*tcc_handler)(uint8_t);

void tc_wait_for_sync(Tc *tc)
{
    while (tc->COUNT16.SYNCBUSY.reg) {
    }
}

void tc_set_enable(Tc *tc, bool enable)
{
    tc->COUNT16.CTRLA.bit.ENABLE = enable;
    while (tc->COUNT16.SYNCBUSY.bit.ENABLE) {
    }
}

void tc_reset(Tc *tc)
{
    tc->COUNT16.CTRLA.bit.SWRST = 1;
    while (tc->COUNT16.SYNCBUSY.bit.SWRST) {
    }
}

void tcc_set_enable(Tcc *tcc, bool enable)
{
    tcc->CTRLA.bit.ENABLE = enable;
    while (tcc->SYNCBUSY.bit.ENABLE) {
    }
}

void tc_set_app_handler(void (*handler)(uint8_t)) { tc_handler = handler; }
void tcc_set_app_handler(void (*handler)(uint8_t)) { tcc_handler = handler; }

static void tc_irq(uint8_t index)
{
    if (tc_handler)
        tc_handler(index);
}

static void tcc_irq(uint8_t index)
{
    if (tcc_handler)
        tcc_handler(index);
}

void TC0_Handler(void) { tc_irq(0); }
void TC1_Handler(void) { tc_irq(1); }
void TC2_Handler(void) { tc_irq(2); }
void TC3_Handler(void) { tc_irq(3); }
// Line 0 of each TCC carries OVF/TRG/CNT/ERR/DFS/UFS; MCn lines follow it.
void TCC0_0_Handler(void) { tcc_irq(0); }
void TCC1_0_Handler(void) { tcc_irq(1); }
void TCC2_0_Handler(void) { tcc_irq(2); }
