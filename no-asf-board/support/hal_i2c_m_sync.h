// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored polled SERCOM I2C master with the message interface the
// selected ZI2C driver uses. Status codes keep the caller's recovery contract:
// -4 address not acknowledged, -5 bus error (both trigger ZI2C bus recovery).
#pragma once
#include <stdint.h>
#include "sam.h"
#include "err_codes.h"

#ifdef __cplusplus
extern "C" {
#endif

#define I2C_OK 0
#define I2C_ACK -1
#define I2C_NACK -2
#define I2C_ERR_ARBLOST -3
#define I2C_ERR_BAD_ADDRESS -4
#define I2C_ERR_BUS -5
#define I2C_ERR_BUSY -6

#define I2C_M_RD 0x0001
#define I2C_M_BUSY 0x0100
#define I2C_M_TEN 0x0400
#define I2C_M_SEVEN 0x0800
#define I2C_M_FAIL 0x1000
#define I2C_M_STOP 0x8000

struct _i2c_m_msg {
    uint16_t addr;
    volatile uint16_t flags;
    int32_t len;
    uint8_t *buffer;
};
struct _i2c_m_sync_device {
    void *hw;
    uint32_t timeout;
};
struct i2c_m_sync_desc {
    struct _i2c_m_sync_device device;
    uint16_t slave_addr;
};

int32_t i2c_m_sync_init(struct i2c_m_sync_desc *i2c, void *hw);
void i2c_m_sync_deinit(struct i2c_m_sync_desc *i2c);
int32_t i2c_m_sync_enable(struct i2c_m_sync_desc *i2c);
void i2c_m_sync_disable(struct i2c_m_sync_desc *i2c);
// clkrate is unused (the core clock is fixed); baudrate is in kHz.
int32_t i2c_m_sync_set_baudrate(struct i2c_m_sync_desc *i2c, uint32_t clkrate, uint32_t baudrate);
int32_t _i2c_m_sync_transfer(struct _i2c_m_sync_device *device, struct _i2c_m_msg *msg);

#ifdef __cplusplus
}
#endif
