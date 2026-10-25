// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#include "samd/clocks.h"
#ifdef __cplusplus
extern "C" {
#endif
static inline void _gclk_enable_channel(uint8_t channel, uint8_t generator)
{
    connect_gclk_to_peripheral(generator, channel);
}
#ifdef __cplusplus
}
#endif
