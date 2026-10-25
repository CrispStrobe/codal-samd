// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "hal_usart_async.h"
#include "sercom.h"

static int sercom_index(Sercom *s)
{
    for (int i = 0; i < SERCOM_INST_NUM; i++)
        if (sercom_insts[i] == s)
            return i;
    return -1;
}

static void sync(Sercom *s)
{
    while (s->USART.SYNCBUSY.reg) {
    }
}

// DRE -> byte sent, TXC -> transmission complete, RXC -> byte received,
// ERROR (or a receive status error) -> error. Each flag is cleared as served.
static void usart_irq(void *opaque)
{
    struct _usart_async_device *device = (struct _usart_async_device *)opaque;
    Sercom *s = (Sercom *)device->hw;
    uint8_t flags = s->USART.INTFLAG.reg & s->USART.INTENSET.reg;
    if (flags & SERCOM_USART_INTFLAG_DRE) {
        s->USART.INTENCLR.reg = SERCOM_USART_INTENCLR_DRE;
        if (device->usart_cb.tx_byte_sent)
            device->usart_cb.tx_byte_sent(device);
    } else if (flags & SERCOM_USART_INTFLAG_TXC) {
        s->USART.INTENCLR.reg = SERCOM_USART_INTENCLR_TXC;
        s->USART.INTFLAG.reg = SERCOM_USART_INTFLAG_TXC;
        if (device->usart_cb.tx_done_cb)
            device->usart_cb.tx_done_cb(device);
    } else if (flags & SERCOM_USART_INTFLAG_RXC) {
        if (s->USART.STATUS.reg & (SERCOM_USART_STATUS_PERR | SERCOM_USART_STATUS_FERR | SERCOM_USART_STATUS_BUFOVF |
                                   SERCOM_USART_STATUS_ISF | SERCOM_USART_STATUS_COLL)) {
            s->USART.STATUS.reg = SERCOM_USART_STATUS_MASK;
            (void)s->USART.DATA.reg;
            if (device->usart_cb.error_cb)
                device->usart_cb.error_cb(device);
            return;
        }
        uint8_t data = (uint8_t)s->USART.DATA.reg;
        if (device->usart_cb.rx_done_cb)
            device->usart_cb.rx_done_cb(device, data);
    } else if (flags & SERCOM_USART_INTFLAG_ERROR) {
        s->USART.INTFLAG.reg = SERCOM_USART_INTFLAG_ERROR;
        if (device->usart_cb.error_cb)
            device->usart_cb.error_cb(device);
    }
}

int32_t _usart_async_init(struct _usart_async_device *device, void *hw)
{
    Sercom *s = (Sercom *)hw;
    int index = sercom_index(s);
    if (!device || index < 0)
        return ERR_INVALID_ARG;
    s->USART.CTRLA.reg = SERCOM_USART_CTRLA_SWRST;
    while (s->USART.SYNCBUSY.reg & SERCOM_USART_SYNCBUSY_SWRST) {
    }
    // Internal clock, LSB first, 16x arithmetic sampling, 8N1.
    s->USART.CTRLA.reg = SERCOM_USART_CTRLA_MODE_USART_INT_CLK | SERCOM_USART_CTRLA_DORD | SERCOM_USART_CTRLA_SAMPR(0);
    s->USART.CTRLB.reg = SERCOM_USART_CTRLB_CHSIZE(0);
    sync(s);
    device->hw = hw;
    sercom_register_irq_handler((uint8_t)index, usart_irq, device);
    for (int line = 0; line < 4; line++) {
        IRQn_Type irq = (IRQn_Type)(SERCOM0_0_IRQn + 4 * index + line);
        NVIC_DisableIRQ(irq);
        NVIC_ClearPendingIRQ(irq);
        NVIC_EnableIRQ(irq);
    }
    return ERR_NONE;
}

void _usart_async_deinit(struct _usart_async_device *device)
{
    Sercom *s = (Sercom *)device->hw;
    int index = sercom_index(s);
    for (int line = 0; index >= 0 && line < 4; line++)
        NVIC_DisableIRQ((IRQn_Type)(SERCOM0_0_IRQn + 4 * index + line));
    if (index >= 0)
        sercom_register_irq_handler((uint8_t)index, 0, 0);
    s->USART.CTRLA.reg = SERCOM_USART_CTRLA_SWRST;
    while (s->USART.SYNCBUSY.reg & SERCOM_USART_SYNCBUSY_SWRST) {
    }
}

void _usart_async_set_irq_state(struct _usart_async_device *device, enum _usart_async_callback_type type, bool state)
{
    Sercom *s = (Sercom *)device->hw;
    uint8_t mask = 0;
    switch (type) {
    case USART_ASYNC_BYTE_SENT: mask = SERCOM_USART_INTENSET_DRE; break;
    case USART_ASYNC_RX_DONE: mask = SERCOM_USART_INTENSET_RXC; break;
    case USART_ASYNC_TX_DONE: mask = SERCOM_USART_INTENSET_TXC; break;
    case USART_ASYNC_ERROR: mask = SERCOM_USART_INTENSET_ERROR; break;
    }
    if (state)
        s->USART.INTENSET.reg = mask;
    else
        s->USART.INTENCLR.reg = mask;
}

// Arithmetic: BAUD = 65536 * (1 - samples * f_baud / f_ref).
uint16_t _usart_async_calculate_baud_rate(uint32_t baud, uint32_t clock_rate, uint8_t samples,
                                          enum usart_baud_rate_mode mode, uint8_t fraction)
{
    (void)fraction;
    if (mode != USART_BAUDRATE_ASYNCH_ARITHMETIC || clock_rate == 0)
        return 0;
    uint64_t ratio = ((uint64_t)samples * baud << 16) / clock_rate;
    return ratio >= 65536 ? 0 : (uint16_t)(65536 - ratio);
}

void _usart_async_set_baud_rate(struct _usart_async_device *device, uint32_t value)
{
    Sercom *s = (Sercom *)device->hw;
    bool enabled = s->USART.CTRLA.reg & SERCOM_USART_CTRLA_ENABLE;
    if (enabled) {
        s->USART.CTRLA.reg &= ~SERCOM_USART_CTRLA_ENABLE;
        sync(s);
    }
    s->USART.BAUD.reg = (uint16_t)value;
    if (enabled) {
        s->USART.CTRLA.reg |= SERCOM_USART_CTRLA_ENABLE;
        sync(s);
    }
}
