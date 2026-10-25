// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#include "sam.h"
#ifdef __cplusplus
extern "C" {
#endif
// Enable the CMCC instruction/data cache after invalidating it.
static inline void samd_peripherals_enable_cache(void)
{
    CMCC->CTRL.bit.CEN = 0;
    while (CMCC->SR.bit.CSTS) {
    }
    CMCC->MAINT0.bit.INVALL = 1;
    CMCC->CTRL.bit.CEN = 1;
}
#ifdef __cplusplus
}
#endif
