// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored SERCOM clocking and SPI pad/baud helpers.
#pragma once
#include <stdbool.h>
#include <stdint.h>
#include "sam.h"

#ifdef __cplusplus
extern "C" {
#endif

extern Sercom *sercom_insts[SERCOM_INST_NUM];
// Bus clock, 88 MHz SPI/USART core (SERCOM_CORE_CLOCK) and 32 kHz slow clock.
void samd_peripherals_sercom_clock_init(Sercom *sercom, uint8_t sercom_index);
// SAMD51 DOPO: 0 = DO PAD0/SCK PAD1, 2 = DO PAD3/SCK PAD1; 0xff if not permitted.
uint8_t samd_peripherals_get_spi_dopo(uint8_t clock_pad, uint8_t mosi_pad);
bool samd_peripherals_valid_spi_clock_pad(uint8_t clock_pad);
uint8_t samd_peripherals_spi_baudrate_to_baud_reg_value(uint32_t baudrate);
uint32_t samd_peripherals_spi_baud_reg_value_to_baudrate(uint8_t baud_reg_value);
// Route a SERCOM's interrupt lines to a device registered by the USART layer.
void sercom_register_irq_handler(uint8_t sercom_index, void (*handler)(void *), void *device);

#ifdef __cplusplus
}
#endif
