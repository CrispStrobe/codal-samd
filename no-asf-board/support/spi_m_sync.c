// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "hal_spi_m_sync.h"

static Sercom *hw_of(struct spi_m_sync_descriptor *spi) { return (Sercom *)spi->dev.prvt; }

static void sync(Sercom *s)
{
    while (s->SPI.SYNCBUSY.reg) {
    }
}

int32_t spi_m_sync_init(struct spi_m_sync_descriptor *spi, void *hw)
{
    if (!spi || !hw)
        return ERR_INVALID_ARG;
    Sercom *s = (Sercom *)hw;
    if (s->SPI.CTRLA.reg & SERCOM_SPI_CTRLA_ENABLE)
        return ERR_DENIED;
    s->SPI.CTRLA.reg = SERCOM_SPI_CTRLA_SWRST;
    while (s->SPI.SYNCBUSY.reg & SERCOM_SPI_SYNCBUSY_SWRST) {
    }
    // Master mode, MSB first, mode 0; DOPO/DIPO are selected by the caller.
    s->SPI.CTRLA.reg = SERCOM_SPI_CTRLA_MODE_SPI_MASTER;
    s->SPI.CTRLB.reg = SERCOM_SPI_CTRLB_RXEN | SERCOM_SPI_CTRLB_CHSIZE(0);
    sync(s);
    spi->dev.prvt = hw;
    spi->dev.char_size = 1;
    spi->dev.dummy_byte = 0;
    return ERR_NONE;
}

void spi_m_sync_deinit(struct spi_m_sync_descriptor *spi)
{
    Sercom *s = hw_of(spi);
    s->SPI.CTRLA.reg = SERCOM_SPI_CTRLA_SWRST;
    while (s->SPI.SYNCBUSY.reg & SERCOM_SPI_SYNCBUSY_SWRST) {
    }
}

void spi_m_sync_enable(struct spi_m_sync_descriptor *spi)
{
    Sercom *s = hw_of(spi);
    s->SPI.CTRLA.reg |= SERCOM_SPI_CTRLA_ENABLE;
    while (s->SPI.SYNCBUSY.reg & SERCOM_SPI_SYNCBUSY_ENABLE) {
    }
}

void spi_m_sync_disable(struct spi_m_sync_descriptor *spi)
{
    Sercom *s = hw_of(spi);
    s->SPI.CTRLA.reg &= ~SERCOM_SPI_CTRLA_ENABLE;
    while (s->SPI.SYNCBUSY.reg & SERCOM_SPI_SYNCBUSY_ENABLE) {
    }
}

int32_t spi_m_sync_set_baudrate(struct spi_m_sync_descriptor *spi, uint32_t value)
{
    Sercom *s = hw_of(spi);
    if (s->SPI.CTRLA.reg & SERCOM_SPI_CTRLA_ENABLE)
        return ERR_BUSY;
    if (value > 255)
        return ERR_INVALID_ARG;
    s->SPI.BAUD.reg = (uint8_t)value;
    return ERR_NONE;
}
