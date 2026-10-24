// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored interrupt-driven SERCOM USART device with the callback
// interface the selected SAMDSerial/ZSingleWireSerial drivers use. The drivers
// themselves set pads, enable the USART and move data.
#pragma once
#include <stdbool.h>
#include <stdint.h>
#include "sam.h"
#include "err_codes.h"

#ifdef __cplusplus
extern "C" {
#endif

enum _usart_async_callback_type { USART_ASYNC_BYTE_SENT, USART_ASYNC_RX_DONE, USART_ASYNC_TX_DONE, USART_ASYNC_ERROR };
enum usart_baud_rate_mode {
    USART_BAUDRATE_ASYNCH_ARITHMETIC,
    USART_BAUDRATE_ASYNCH_FRACTIONAL,
    USART_BAUDRATE_SYNCH
};

struct _usart_async_device;
struct _usart_async_callbacks {
    void (*tx_byte_sent)(struct _usart_async_device *device);
    void (*rx_done_cb)(struct _usart_async_device *device, uint8_t data);
    void (*tx_done_cb)(struct _usart_async_device *device);
    void (*error_cb)(struct _usart_async_device *device);
};
struct _usart_async_device {
    struct _usart_async_callbacks usart_cb;
    void *hw;
};

int32_t _usart_async_init(struct _usart_async_device *device, void *hw);
void _usart_async_deinit(struct _usart_async_device *device);
void _usart_async_set_irq_state(struct _usart_async_device *device, enum _usart_async_callback_type type, bool state);
uint16_t _usart_async_calculate_baud_rate(uint32_t baud, uint32_t clock_rate, uint8_t samples,
                                          enum usart_baud_rate_mode mode, uint8_t fraction);
void _usart_async_set_baud_rate(struct _usart_async_device *device, uint32_t baud_reg_value);

#ifdef __cplusplus
}
#endif
