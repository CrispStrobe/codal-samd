// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Pin objects for the selected CODAL drivers. The table itself is generated from
// the Apache-2.0 ATDF (samd51g19a_pins.c); these declarations are authored.
#pragma once
#include <stdbool.h>
#include <stdint.h>
#include "sam.h"
#include "hal_gpio.h"

#ifdef __cplusplus
extern "C" {
#endif

#define PIN_NONE 0xff
#define NO_ADC 0xff
#define NO_INDEX 0x3f
#define NUM_SERCOMS_PER_PIN 2
#define NUM_TIMERS_PER_PIN 3
#define NUM_ADC_PER_PIN 2

#define MUX_A 0
#define MUX_B 1
#define MUX_C 2
#define MUX_D 3
#define MUX_E 4
#define MUX_F 5
#define MUX_G 6
#define MUX_H 7

// index NO_INDEX means absent; pad is the SERCOM PAD[n] a pin can drive.
typedef struct { uint8_t index; uint8_t pad; } pin_sercom_t;
// timer[0] is the TC on function E, timer[1]/[2] the TCC on functions F/G.
typedef struct { uint8_t index; uint8_t is_tc; uint8_t wave_output; } pin_timer_t;
typedef struct {
    uint8_t number;
    bool has_extint;
    uint8_t extint_channel;
    uint8_t adc_input[NUM_ADC_PER_PIN];
    pin_timer_t timer[NUM_TIMERS_PER_PIN];
    pin_sercom_t sercom[NUM_SERCOMS_PER_PIN];
} mcu_pin_obj_t;

extern const mcu_pin_obj_t samd_pins[];
const mcu_pin_obj_t *samd_peripherals_get_pin(uint32_t number);

#ifdef __cplusplus
}
#endif
