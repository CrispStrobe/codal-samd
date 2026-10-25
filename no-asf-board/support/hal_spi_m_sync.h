// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored SERCOM SPI master setup. Transfers are DMA-driven by
// the selected ZSPI driver; this layer only owns mode/enable/baud state.
#pragma once
#include <stdint.h>
#include "sam.h"
#include "err_codes.h"

#ifdef __cplusplus
extern "C" {
#endif

struct _spi_m_sync_dev {
    void *prvt; // the SERCOM instance
    uint8_t char_size;
    uint16_t dummy_byte;
};
struct spi_m_sync_descriptor {
    struct _spi_m_sync_dev dev;
};

int32_t spi_m_sync_init(struct spi_m_sync_descriptor *spi, void *hw);
void spi_m_sync_deinit(struct spi_m_sync_descriptor *spi);
void spi_m_sync_enable(struct spi_m_sync_descriptor *spi);
void spi_m_sync_disable(struct spi_m_sync_descriptor *spi);
// BAUD is enable-protected: refuse while the master is enabled.
int32_t spi_m_sync_set_baudrate(struct spi_m_sync_descriptor *spi, uint32_t baud_reg_value);

#ifdef __cplusplus
}
#endif
