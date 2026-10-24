// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored TC/TCC helpers and IRQ dispatch for the no-ASF board.
#pragma once
#include <stdbool.h>
#include <stdint.h>
#include "sam.h"

#ifdef __cplusplus
extern "C" {
#endif

extern Tc *const tc_insts[TC_INST_NUM];
extern Tcc *const tcc_insts[TCC_INST_NUM];
// Compare channels per TCC and the TC/TCC PRESCALER divisors, by register value.
extern const uint8_t tcc_cc_num[TCC_INST_NUM];
extern const uint16_t prescaler[8];

void tc_wait_for_sync(Tc *tc);
void tc_set_enable(Tc *tc, bool enable);
void tc_reset(Tc *tc);
void tcc_set_enable(Tcc *tcc, bool enable);
// The application handler receives the instance index of the interrupting TC/TCC.
void tc_set_app_handler(void (*handler)(uint8_t index));
void tcc_set_app_handler(void (*handler)(uint8_t index));

#ifdef __cplusplus
}
#endif
