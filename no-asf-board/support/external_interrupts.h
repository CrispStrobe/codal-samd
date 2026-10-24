// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored SAMD51 EIC helpers and per-channel IRQ dispatch.
#pragma once
#include <stdbool.h>
#include <stdint.h>
#include "sam.h"

#ifdef __cplusplus
extern "C" {
#endif

#define EIC_CHANNEL_COUNT 16
#define EIC_HANDLER_NO_INTERRUPT 0
#define EIC_HANDLER_APP 1

void turn_on_external_interrupt_controller(void);
void turn_off_external_interrupt_controller(void);
void eic_reset(void);
void eic_set_enable(bool enable);
bool eic_get_enable(void);
// sense is a CONFIG.SENSEn value: 0 none, 1 rise, 2 fall, 3 both, 4 high, 5 low.
void configure_eic_channel(uint8_t channel, uint32_t sense);
void turn_on_eic_channel(uint8_t channel, uint32_t sense, uint8_t handler_kind);
void turn_off_eic_channel(uint8_t channel);
void turn_on_cpu_interrupt(uint8_t channel);
void set_eic_irq_handler(void (*handler)(uint8_t channel));

#ifdef __cplusplus
}
#endif
