// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "external_interrupts.h"
#include "samd/clocks.h"

static void (*app_handler)(uint8_t);
static uint8_t handler_kinds[EIC_CHANNEL_COUNT];

static void eic_sync(void)
{
    while (EIC->SYNCBUSY.reg) {
    }
}

void turn_on_external_interrupt_controller(void)
{
    MCLK->APBAMASK.reg |= MCLK_APBAMASK_EIC;
    connect_gclk_to_peripheral(CLK_GEN_48MHZ, EIC_GCLK_ID);
}

void turn_off_external_interrupt_controller(void)
{
    eic_set_enable(false);
    MCLK->APBAMASK.reg &= ~MCLK_APBAMASK_EIC;
}

void eic_reset(void)
{
    EIC->CTRLA.bit.SWRST = 1;
    while (EIC->SYNCBUSY.bit.SWRST) {
    }
    for (int i = 0; i < EIC_CHANNEL_COUNT; i++)
        NVIC_DisableIRQ((IRQn_Type)(EIC_0_IRQn + i));
}

void eic_set_enable(bool enable)
{
    EIC->CTRLA.bit.ENABLE = enable;
    while (EIC->SYNCBUSY.bit.ENABLE) {
    }
}

bool eic_get_enable(void) { return EIC->CTRLA.bit.ENABLE; }

// CONFIG is enable-protected: write it with the controller disabled.
void configure_eic_channel(uint8_t channel, uint32_t sense)
{
    bool enabled = eic_get_enable();
    if (enabled)
        eic_set_enable(false);
    uint8_t shift = (uint8_t)(4 * (channel % 8));
    uint32_t value = EIC->CONFIG[channel / 8].reg;
    value &= ~(0xfu << shift);
    value |= (sense & 0x7u) << shift;
    EIC->CONFIG[channel / 8].reg = value;
    if (enabled)
        eic_set_enable(true);
    eic_sync();
}

void turn_on_cpu_interrupt(uint8_t channel)
{
    IRQn_Type irq = (IRQn_Type)(EIC_0_IRQn + channel);
    NVIC_DisableIRQ(irq);
    NVIC_ClearPendingIRQ(irq);
    NVIC_EnableIRQ(irq);
}

void turn_on_eic_channel(uint8_t channel, uint32_t sense, uint8_t handler_kind)
{
    handler_kinds[channel] = handler_kind;
    configure_eic_channel(channel, sense);
    EIC->INTFLAG.reg = 1u << channel;
    EIC->INTENSET.reg = 1u << channel;
    turn_on_cpu_interrupt(channel);
}

void turn_off_eic_channel(uint8_t channel)
{
    EIC->INTENCLR.reg = 1u << channel;
    configure_eic_channel(channel, 0);
    NVIC_DisableIRQ((IRQn_Type)(EIC_0_IRQn + channel));
    handler_kinds[channel] = EIC_HANDLER_NO_INTERRUPT;
}

void set_eic_irq_handler(void (*handler)(uint8_t)) { app_handler = handler; }

static void eic_irq(uint8_t channel)
{
    EIC->INTFLAG.reg = 1u << channel;
    if (handler_kinds[channel] == EIC_HANDLER_APP && app_handler)
        app_handler(channel);
}

void EIC_0_Handler(void) { eic_irq(0); }
void EIC_1_Handler(void) { eic_irq(1); }
void EIC_2_Handler(void) { eic_irq(2); }
void EIC_3_Handler(void) { eic_irq(3); }
void EIC_4_Handler(void) { eic_irq(4); }
void EIC_5_Handler(void) { eic_irq(5); }
void EIC_6_Handler(void) { eic_irq(6); }
void EIC_7_Handler(void) { eic_irq(7); }
void EIC_8_Handler(void) { eic_irq(8); }
void EIC_9_Handler(void) { eic_irq(9); }
void EIC_10_Handler(void) { eic_irq(10); }
void EIC_11_Handler(void) { eic_irq(11); }
void EIC_12_Handler(void) { eic_irq(12); }
void EIC_13_Handler(void) { eic_irq(13); }
void EIC_14_Handler(void) { eic_irq(14); }
void EIC_15_Handler(void) { eic_irq(15); }
