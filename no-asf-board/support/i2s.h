// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// The I2S bus clock the selected PDM microphone driver enables.
#pragma once
#include "sam.h"
#ifdef __cplusplus
extern "C" {
#endif
static inline void turn_on_i2s(void) { MCLK->APBDMASK.reg |= MCLK_APBDMASK_I2S; }
#ifdef __cplusplus
}
#endif
