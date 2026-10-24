// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#include "hal_adc_sync.h"
#ifdef __cplusplus
extern "C" {
#endif
// Bus and generic clock, controller reset and factory calibration for ADC0/ADC1.
void samd_peripherals_adc_setup(struct adc_sync_descriptor *descr, Adc *adc);
#ifdef __cplusplus
}
#endif
