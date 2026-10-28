// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored SAMD51 clock tree for the no-ASF board. Generator plan:
//   GCLK0 120 MHz DPLL0 (CPU)     GCLK1 48 MHz DFLL (USB)
//   GCLK2 88 MHz DPLL1 (SPI/USART)  GCLK3 32.768 kHz OSCULP32K
//   GCLK4 8 MHz DFLL/6 (TC/TCC)   GCLK5 1 MHz DFLL/48 (DPLL reference)
#pragma once
#include <stdbool.h>
#include <stdint.h>
#include "sam.h"

#ifdef __cplusplus
extern "C" {
#endif

#define CLK_GEN_120MHZ 0
#define CLK_GEN_48MHZ 1
#define CLK_GEN_88MHZ 2
#define CLK_GEN_32KHZ 3
#define CLK_GEN_8MHZ 4
#define CLK_GEN_1MHZ 5

// Generator sources, as GENCTRL.SRC values.
#define CLOCK_48MHZ GCLK_GENCTRL_SRC_DFLL_Val
#define CLOCK_120MHZ GCLK_GENCTRL_SRC_DPLL0_Val
#define CLOCK_88MHZ GCLK_GENCTRL_SRC_DPLL1_Val
#define CLOCK_32KHZ GCLK_GENCTRL_SRC_OSCULP32K_Val

#define SERCOM_CORE_FREQUENCY 88000000u
#define SERCOM_CORE_CLOCK CLK_GEN_88MHZ

void clock_init(void);
void enable_clock_generator(uint8_t generator, uint32_t source, uint16_t divisor);
void disable_clock_generator(uint8_t generator);
void connect_gclk_to_peripheral(uint8_t generator, uint8_t peripheral);
void disconnect_gclk_from_peripheral(uint8_t generator, uint8_t peripheral);
void turn_on_clocks(bool is_tc, uint8_t index, uint32_t generator);
uint32_t clock_generator_frequency(uint8_t generator);

#ifdef __cplusplus
}
#endif
