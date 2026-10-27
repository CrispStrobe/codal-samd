// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "hal_i2c_m_sync.h"
#include "samd/clocks.h"

// I2C needs standard-mode rates, unreachable from 100 MHz with 8-bit BAUD
// fields, so its core clock is the 48 MHz generator.
#define I2C_CORE_FREQUENCY 48000000u
#define I2C_RISE_NS 215u
// Flag polls before a transfer is abandoned: longer than the SCL low time-out
// (25-35 ms), so a device holding SCL ends in LOWTOUT rather than in the
// budget, which remains for a bus that never moves (no clock, no pads).
#define I2C_POLL_LIMIT 1000000u

static const uint8_t core_gclk[SERCOM_INST_NUM] = {SERCOM0_GCLK_ID_CORE, SERCOM1_GCLK_ID_CORE, SERCOM2_GCLK_ID_CORE,
                                                   SERCOM3_GCLK_ID_CORE, SERCOM4_GCLK_ID_CORE, SERCOM5_GCLK_ID_CORE};

static Sercom *hw_of(struct _i2c_m_sync_device *d) { return (Sercom *)d->hw; }

static void sync(Sercom *s)
{
    while (s->I2CM.SYNCBUSY.reg) {
    }
}

static int sercom_index(Sercom *s)
{
    static Sercom *const all[SERCOM_INST_NUM] = SERCOM_INSTS;
    for (int i = 0; i < SERCOM_INST_NUM; i++)
        if (all[i] == s)
            return i;
    return -1;
}

int32_t i2c_m_sync_init(struct i2c_m_sync_desc *i2c, void *hw)
{
    Sercom *s = (Sercom *)hw;
    int index = sercom_index(s);
    if (!i2c || index < 0)
        return ERR_INVALID_ARG;
    connect_gclk_to_peripheral(CLK_GEN_48MHZ, core_gclk[index]);
    s->I2CM.CTRLA.reg = SERCOM_I2CM_CTRLA_SWRST;
    while (s->I2CM.SYNCBUSY.reg & SERCOM_I2CM_SYNCBUSY_SWRST) {
    }
    // Master mode, 300-600 ns SDA hold, SCL low time-out (a device holding SCL
    // for 25-35 ms ends the transfer with LOWTOUT and BUSERR); smart mode
    // acknowledges on DATA reads.
    s->I2CM.CTRLA.reg = SERCOM_I2CM_CTRLA_MODE_I2C_MASTER | SERCOM_I2CM_CTRLA_SDAHOLD(2) | SERCOM_I2CM_CTRLA_LOWTOUTEN;
    s->I2CM.CTRLB.reg = SERCOM_I2CM_CTRLB_SMEN;
    sync(s);
    i2c->device.hw = hw;
    i2c->device.timeout = I2C_POLL_LIMIT;
    return ERR_NONE;
}

void i2c_m_sync_deinit(struct i2c_m_sync_desc *i2c)
{
    Sercom *s = hw_of(&i2c->device);
    s->I2CM.CTRLA.reg = SERCOM_I2CM_CTRLA_SWRST;
    while (s->I2CM.SYNCBUSY.reg & SERCOM_I2CM_SYNCBUSY_SWRST) {
    }
}

int32_t i2c_m_sync_enable(struct i2c_m_sync_desc *i2c)
{
    Sercom *s = hw_of(&i2c->device);
    s->I2CM.CTRLA.reg |= SERCOM_I2CM_CTRLA_ENABLE;
    while (s->I2CM.SYNCBUSY.reg & SERCOM_I2CM_SYNCBUSY_ENABLE) {
    }
    return ERR_NONE;
}

void i2c_m_sync_disable(struct i2c_m_sync_desc *i2c)
{
    Sercom *s = hw_of(&i2c->device);
    s->I2CM.CTRLA.reg &= ~SERCOM_I2CM_CTRLA_ENABLE;
    while (s->I2CM.SYNCBUSY.reg & SERCOM_I2CM_SYNCBUSY_ENABLE) {
    }
}

// f_SCL = f_core / (10 + BAUD + BAUDLOW + f_core * t_rise); BAUD is the high time.
int32_t i2c_m_sync_set_baudrate(struct i2c_m_sync_desc *i2c, uint32_t clkrate, uint32_t baudrate)
{
    (void)clkrate;
    Sercom *s = hw_of(&i2c->device);
    if (s->I2CM.CTRLA.reg & SERCOM_I2CM_CTRLA_ENABLE)
        return ERR_DENIED;
    if (baudrate == 0)
        return ERR_INVALID_ARG;
    uint32_t cycles = I2C_CORE_FREQUENCY / (baudrate * 1000u);
    uint32_t rise = (uint32_t)(((uint64_t)I2C_CORE_FREQUENCY * I2C_RISE_NS) / 1000000000u);
    if (cycles < 10 + rise + 2)
        return ERR_INVALID_ARG;
    uint32_t total = cycles - 10 - rise;
    uint32_t high = total / 2, low = total - high;
    if (high > 255 || low > 255)
        return ERR_INVALID_ARG;
    s->I2CM.BAUD.reg = SERCOM_I2CM_BAUD_BAUD(high) | SERCOM_I2CM_BAUD_BAUDLOW(low);
    return ERR_NONE;
}

static int32_t wait_flag(Sercom *s, uint32_t limit)
{
    while (!(s->I2CM.INTFLAG.reg & (SERCOM_I2CM_INTFLAG_MB | SERCOM_I2CM_INTFLAG_SB))) {
        if (s->I2CM.INTFLAG.reg & SERCOM_I2CM_INTFLAG_ERROR)
            return I2C_ERR_BUS;
        if (!limit--)
            return I2C_ERR_BUSY;
    }
    // A bus error or SCL low time-out still sets MB or SB (DS60001507F
    // 36.6.2.4.6): the transfer failed although a flag is up.
    if (s->I2CM.INTFLAG.reg & SERCOM_I2CM_INTFLAG_ERROR) {
        s->I2CM.STATUS.reg = SERCOM_I2CM_STATUS_BUSERR | SERCOM_I2CM_STATUS_LOWTOUT;
        s->I2CM.INTFLAG.reg = SERCOM_I2CM_INTFLAG_ERROR;
        return I2C_ERR_BUS;
    }
    return I2C_OK;
}

static void stop(Sercom *s)
{
    s->I2CM.CTRLB.reg = (s->I2CM.CTRLB.reg & ~SERCOM_I2CM_CTRLB_CMD_Msk) | SERCOM_I2CM_CTRLB_CMD(3);
    sync(s);
}

int32_t _i2c_m_sync_transfer(struct _i2c_m_sync_device *device, struct _i2c_m_msg *msg)
{
    Sercom *s = hw_of(device);
    if (msg->len < 0 || (msg->len && !msg->buffer))
        return I2C_ERR_BUSY;
    bool read = msg->flags & I2C_M_RD;
    // Smart-mode reads ACK each byte; the last one is NACKed by ACKACT before STOP.
    s->I2CM.CTRLB.reg &= ~SERCOM_I2CM_CTRLB_ACKACT;
    sync(s);
    s->I2CM.ADDR.reg = SERCOM_I2CM_ADDR_ADDR((uint32_t)(msg->addr << 1) | (read ? 1u : 0u));
    sync(s);
    int32_t status = wait_flag(s, device->timeout);
    if (status != I2C_OK) {
        stop(s);
        return status;
    }
    if (s->I2CM.STATUS.reg & SERCOM_I2CM_STATUS_ARBLOST) {
        s->I2CM.INTFLAG.reg = SERCOM_I2CM_INTFLAG_MB;
        return I2C_ERR_BUS;
    }
    if (s->I2CM.STATUS.reg & SERCOM_I2CM_STATUS_RXNACK) {
        stop(s);
        return I2C_ERR_BAD_ADDRESS;
    }
    for (int32_t i = 0; i < msg->len; i++) {
        if (read) {
            if (i == msg->len - 1) {
                s->I2CM.CTRLB.reg |= SERCOM_I2CM_CTRLB_ACKACT;
                sync(s);
                if (msg->flags & I2C_M_STOP)
                    stop(s);
            }
            msg->buffer[i] = (uint8_t)s->I2CM.DATA.reg;
            sync(s);
            if (i < msg->len - 1 && (status = wait_flag(s, device->timeout)) != I2C_OK)
                return status;
        } else {
            s->I2CM.DATA.reg = msg->buffer[i];
            sync(s);
            if ((status = wait_flag(s, device->timeout)) != I2C_OK) {
                stop(s);
                return status;
            }
            if (s->I2CM.STATUS.reg & SERCOM_I2CM_STATUS_RXNACK) {
                stop(s);
                return I2C_NACK;
            }
        }
    }
    if (!read && (msg->flags & I2C_M_STOP))
        stop(s);
    return I2C_OK;
}
