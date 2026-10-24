// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored single-conversion SAMD51 ADC with the call surface the
// selected ZPin analog read uses. One channel, software-triggered, polled.
#pragma once
#include <stdint.h>
#include "sam.h"
#include "err_codes.h"

#ifdef __cplusplus
extern "C" {
#endif

struct _adc_sync_device {
    void *hw;
};
struct adc_sync_descriptor {
    struct _adc_sync_device device;
};

int32_t adc_sync_init(struct adc_sync_descriptor *descr, void *hw, void *func);
int32_t adc_sync_deinit(struct adc_sync_descriptor *descr);
int32_t adc_sync_enable_channel(struct adc_sync_descriptor *descr, uint8_t channel);
int32_t adc_sync_set_reference(struct adc_sync_descriptor *descr, uint8_t reference);
int32_t adc_sync_set_resolution(struct adc_sync_descriptor *descr, uint8_t resolution);
int32_t adc_sync_set_inputs(struct adc_sync_descriptor *descr, uint8_t pos_input, uint8_t neg_input, uint8_t channel);
int32_t adc_sync_set_channel_gain(struct adc_sync_descriptor *descr, uint8_t channel, uint8_t gain);
// Returns the number of result bytes written (1 or 2), or a negative error.
int32_t adc_sync_read_channel(struct adc_sync_descriptor *descr, uint8_t channel, uint8_t *buffer, uint16_t length);

#ifdef __cplusplus
}
#endif
