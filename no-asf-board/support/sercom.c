// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "sercom.h"
#include "samd/clocks.h"

Sercom *sercom_insts[SERCOM_INST_NUM] = SERCOM_INSTS;

static const uint8_t core_gclk[SERCOM_INST_NUM] = {SERCOM0_GCLK_ID_CORE, SERCOM1_GCLK_ID_CORE, SERCOM2_GCLK_ID_CORE,
                                                   SERCOM3_GCLK_ID_CORE, SERCOM4_GCLK_ID_CORE, SERCOM5_GCLK_ID_CORE};
static const uint8_t slow_gclk[SERCOM_INST_NUM] = {SERCOM0_GCLK_ID_SLOW, SERCOM1_GCLK_ID_SLOW, SERCOM2_GCLK_ID_SLOW,
                                                   SERCOM3_GCLK_ID_SLOW, SERCOM4_GCLK_ID_SLOW, SERCOM5_GCLK_ID_SLOW};

void samd_peripherals_sercom_clock_init(Sercom *sercom, uint8_t index)
{
    (void)sercom;
    switch (index) {
    case 0: MCLK->APBAMASK.reg |= MCLK_APBAMASK_SERCOM0; break;
    case 1: MCLK->APBAMASK.reg |= MCLK_APBAMASK_SERCOM1; break;
    case 2: MCLK->APBBMASK.reg |= MCLK_APBBMASK_SERCOM2; break;
    case 3: MCLK->APBBMASK.reg |= MCLK_APBBMASK_SERCOM3; break;
    case 4: MCLK->APBDMASK.reg |= MCLK_APBDMASK_SERCOM4; break;
    case 5: MCLK->APBDMASK.reg |= MCLK_APBDMASK_SERCOM5; break;
    }
    connect_gclk_to_peripheral(SERCOM_CORE_CLOCK, core_gclk[index]);
    connect_gclk_to_peripheral(CLK_GEN_32KHZ, slow_gclk[index]);
}

uint8_t samd_peripherals_get_spi_dopo(uint8_t clock_pad, uint8_t mosi_pad)
{
    if (clock_pad != 1)
        return 0xff;
    if (mosi_pad == 0)
        return 0;
    if (mosi_pad == 3)
        return 2;
    return 0xff;
}

bool samd_peripherals_valid_spi_clock_pad(uint8_t clock_pad) { return clock_pad == 1; }

// Master SPI: f_SCK = f_core / (2 * (BAUD + 1)). The requested rate is a
// ceiling, as in the samd-peripherals original: the smallest BAUD whose rate
// does not exceed it, ceil(f_core / (2 * baudrate)) - 1. Rounding to the
// nearest value instead ran a 24 MHz request at 25 MHz.
uint8_t samd_peripherals_spi_baudrate_to_baud_reg_value(uint32_t baudrate)
{
    if (baudrate == 0)
        return 255;
    uint32_t divisor = 2 * baudrate;
    uint32_t value = (SERCOM_CORE_FREQUENCY + divisor - 1) / divisor;
    value = value ? value - 1 : 0;
    return (uint8_t)(value > 255 ? 255 : value);
}

uint32_t samd_peripherals_spi_baud_reg_value_to_baudrate(uint8_t value)
{
    return SERCOM_CORE_FREQUENCY / (2 * ((uint32_t)value + 1));
}

static void (*irq_handlers[SERCOM_INST_NUM])(void *);
static void *irq_devices[SERCOM_INST_NUM];

void sercom_register_irq_handler(uint8_t index, void (*handler)(void *), void *device)
{
    irq_handlers[index] = handler;
    irq_devices[index] = device;
}

static void sercom_irq(uint8_t index)
{
    if (irq_handlers[index])
        irq_handlers[index](irq_devices[index]);
}

#define SERCOM_LINES(n)                                                                             \
    void SERCOM##n##_0_Handler(void) { sercom_irq(n); }                                             \
    void SERCOM##n##_1_Handler(void) { sercom_irq(n); }                                             \
    void SERCOM##n##_2_Handler(void) { sercom_irq(n); }                                             \
    void SERCOM##n##_3_Handler(void) { sercom_irq(n); }
SERCOM_LINES(0)
SERCOM_LINES(1)
SERCOM_LINES(2)
SERCOM_LINES(3)
SERCOM_LINES(4)
SERCOM_LINES(5)
