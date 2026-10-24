// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// The register accessors the selected CODAL drivers call by name, authored over
// the generated ATDF register layout. Only these helpers exist; others refuse.
#pragma once
#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// DMAC: the pending channel and its per-channel interrupt flags.
static inline uint16_t hri_dmac_get_INTPEND_reg(const void *hw, uint16_t mask)
{
    return ((const Dmac *)hw)->INTPEND.reg & mask;
}
static inline bool hri_dmac_get_CHINTFLAG_TERR_bit(const void *hw, uint8_t channel)
{
    return ((const Dmac *)hw)->Channel[channel].CHINTFLAG.reg & DMAC_CHINTFLAG_TERR;
}
static inline bool hri_dmac_get_CHINTFLAG_TCMPL_bit(const void *hw, uint8_t channel)
{
    return ((const Dmac *)hw)->Channel[channel].CHINTFLAG.reg & DMAC_CHINTFLAG_TCMPL;
}
static inline void hri_dmac_clear_CHINTFLAG_TERR_bit(void *hw, uint8_t channel)
{
    ((Dmac *)hw)->Channel[channel].CHINTFLAG.reg = DMAC_CHINTFLAG_TERR;
}
static inline void hri_dmac_clear_CHINTFLAG_TCMPL_bit(void *hw, uint8_t channel)
{
    ((Dmac *)hw)->Channel[channel].CHINTFLAG.reg = DMAC_CHINTFLAG_TCMPL;
}
// Write-one-to-clear: clearing with 0 writes nothing, as the original call does.
static inline void hri_dmac_clear_CHINTFLAG_reg(void *hw, uint8_t channel, uint8_t mask)
{
    ((Dmac *)hw)->Channel[channel].CHINTFLAG.reg = mask;
}

// SERCOM SPI fields written while the peripheral is disabled.
static inline void hri_sercomspi_wait_for_sync(const void *hw, uint32_t mask)
{
    while (((const Sercom *)hw)->SPI.SYNCBUSY.reg & mask) {
    }
}
static inline void hri_sercomspi_write_CTRLA_DOPO_bf(void *hw, uint32_t value)
{
    Sercom *s = (Sercom *)hw;
    s->SPI.CTRLA.reg = (s->SPI.CTRLA.reg & ~SERCOM_SPI_CTRLA_DOPO_Msk) | SERCOM_SPI_CTRLA_DOPO(value);
}
static inline void hri_sercomspi_write_CTRLA_DIPO_bf(void *hw, uint32_t value)
{
    Sercom *s = (Sercom *)hw;
    s->SPI.CTRLA.reg = (s->SPI.CTRLA.reg & ~SERCOM_SPI_CTRLA_DIPO_Msk) | SERCOM_SPI_CTRLA_DIPO(value);
}
static inline void hri_sercomspi_write_CTRLA_CPHA_bit(void *hw, bool value)
{
    Sercom *s = (Sercom *)hw;
    s->SPI.CTRLA.reg = (s->SPI.CTRLA.reg & ~SERCOM_SPI_CTRLA_CPHA) | (value ? SERCOM_SPI_CTRLA_CPHA : 0);
}
static inline void hri_sercomspi_write_CTRLA_CPOL_bit(void *hw, bool value)
{
    Sercom *s = (Sercom *)hw;
    s->SPI.CTRLA.reg = (s->SPI.CTRLA.reg & ~SERCOM_SPI_CTRLA_CPOL) | (value ? SERCOM_SPI_CTRLA_CPOL : 0);
}
static inline void hri_sercomspi_set_CTRLA_RUNSTDBY_bit(void *hw)
{
    ((Sercom *)hw)->SPI.CTRLA.reg |= SERCOM_SPI_CTRLA_RUNSTDBY;
}
static inline void hri_sercomspi_write_CTRLB_CHSIZE_bf(void *hw, uint32_t value)
{
    Sercom *s = (Sercom *)hw;
    s->SPI.CTRLB.reg = (s->SPI.CTRLB.reg & ~SERCOM_SPI_CTRLB_CHSIZE_Msk) | SERCOM_SPI_CTRLB_CHSIZE(value);
    hri_sercomspi_wait_for_sync(hw, SERCOM_SPI_SYNCBUSY_CTRLB);
}
static inline void hri_sercomspi_write_BAUD_BAUD_bf(void *hw, uint8_t value)
{
    ((Sercom *)hw)->SPI.BAUD.reg = value;
}

static inline void hri_mclk_set_APBDMASK_DAC_bit(void *hw)
{
    ((Mclk *)hw)->APBDMASK.reg |= MCLK_APBDMASK_DAC;
}

// CMCC and NVMCTRL cache controls used around timing-critical NeoPixel output.
static inline void hri_cmcc_set_CFG_reg(void *hw, uint32_t mask) { ((Cmcc *)hw)->CFG.reg |= mask; }
static inline void hri_cmcc_clear_CFG_reg(void *hw, uint32_t mask) { ((Cmcc *)hw)->CFG.reg &= ~mask; }
static inline void hri_cmcc_write_MAINT0_reg(void *hw, uint32_t value) { ((Cmcc *)hw)->MAINT0.reg = value; }
static inline void hri_nvmctrl_set_CTRLA_CACHEDIS0_bit(void *hw) { ((Nvmctrl *)hw)->CTRLA.reg |= NVMCTRL_CTRLA_CACHEDIS0; }
static inline void hri_nvmctrl_set_CTRLA_CACHEDIS1_bit(void *hw) { ((Nvmctrl *)hw)->CTRLA.reg |= NVMCTRL_CTRLA_CACHEDIS1; }
static inline void hri_nvmctrl_clear_CTRLA_CACHEDIS0_bit(void *hw) { ((Nvmctrl *)hw)->CTRLA.reg &= ~NVMCTRL_CTRLA_CACHEDIS0; }
static inline void hri_nvmctrl_clear_CTRLA_CACHEDIS1_bit(void *hw) { ((Nvmctrl *)hw)->CTRLA.reg &= ~NVMCTRL_CTRLA_CACHEDIS1; }

#ifdef __cplusplus
}
#endif
