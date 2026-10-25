// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "adc.h"
#include "samd/clocks.h"

static Adc *hw_of(struct adc_sync_descriptor *d) { return (Adc *)d->device.hw; }

static void sync(Adc *adc)
{
    while (adc->SYNCBUSY.reg) {
    }
}

int32_t adc_sync_init(struct adc_sync_descriptor *descr, void *hw, void *func)
{
    (void)func;
    if (!descr || !hw)
        return ERR_INVALID_ARG;
    Adc *adc = (Adc *)hw;
    adc->CTRLA.reg = ADC_CTRLA_SWRST;
    while (adc->SYNCBUSY.reg & ADC_SYNCBUSY_SWRST) {
    }
    // 8 MHz generic clock / 4 keeps the ADC clock within its 16 MHz limit.
    adc->CTRLA.reg = ADC_CTRLA_PRESCALER_DIV4;
    adc->CTRLB.reg = ADC_CTRLB_RESSEL_12BIT;
    adc->SAMPCTRL.reg = ADC_SAMPCTRL_SAMPLEN(3);
    sync(adc);
    descr->device.hw = hw;
    return ERR_NONE;
}

int32_t adc_sync_deinit(struct adc_sync_descriptor *descr)
{
    Adc *adc = hw_of(descr);
    adc->CTRLA.reg &= ~ADC_CTRLA_ENABLE;
    sync(adc);
    adc->CTRLA.reg = ADC_CTRLA_SWRST;
    while (adc->SYNCBUSY.reg & ADC_SYNCBUSY_SWRST) {
    }
    return ERR_NONE;
}

int32_t adc_sync_enable_channel(struct adc_sync_descriptor *descr, uint8_t channel)
{
    (void)channel;
    Adc *adc = hw_of(descr);
    adc->CTRLA.reg |= ADC_CTRLA_ENABLE;
    sync(adc);
    return ERR_NONE;
}

int32_t adc_sync_set_reference(struct adc_sync_descriptor *descr, uint8_t reference)
{
    Adc *adc = hw_of(descr);
    adc->REFCTRL.reg = ADC_REFCTRL_REFSEL(reference);
    sync(adc);
    return ERR_NONE;
}

int32_t adc_sync_set_resolution(struct adc_sync_descriptor *descr, uint8_t resolution)
{
    Adc *adc = hw_of(descr);
    adc->CTRLB.reg = (adc->CTRLB.reg & ~ADC_CTRLB_RESSEL_Msk) | ADC_CTRLB_RESSEL(resolution);
    sync(adc);
    return ERR_NONE;
}

int32_t adc_sync_set_inputs(struct adc_sync_descriptor *descr, uint8_t pos_input, uint8_t neg_input, uint8_t channel)
{
    (void)channel;
    Adc *adc = hw_of(descr);
    adc->INPUTCTRL.reg = ADC_INPUTCTRL_MUXPOS(pos_input) | ADC_INPUTCTRL_MUXNEG(neg_input);
    sync(adc);
    return ERR_NONE;
}

// SAMD51 has no per-channel gain stage; only unity gain is accepted.
int32_t adc_sync_set_channel_gain(struct adc_sync_descriptor *descr, uint8_t channel, uint8_t gain)
{
    (void)descr;
    (void)channel;
    return gain ? ERR_INVALID_ARG : ERR_NONE;
}

int32_t adc_sync_read_channel(struct adc_sync_descriptor *descr, uint8_t channel, uint8_t *buffer, uint16_t length)
{
    (void)channel;
    Adc *adc = hw_of(descr);
    if (!buffer || length == 0)
        return ERR_INVALID_ARG;
    bool eight_bit = ((adc->CTRLB.reg & ADC_CTRLB_RESSEL_Msk) >> ADC_CTRLB_RESSEL_Pos) == ADC_CTRLB_RESSEL_8BIT_Val;
    if (!eight_bit && length < 2)
        return ERR_INVALID_ARG;
    adc->INTFLAG.reg = ADC_INTFLAG_RESRDY;
    adc->SWTRIG.reg = ADC_SWTRIG_START;
    while (!(adc->INTFLAG.reg & ADC_INTFLAG_RESRDY)) {
    }
    uint16_t result = adc->RESULT.reg;
    buffer[0] = (uint8_t)result;
    if (eight_bit)
        return 1;
    buffer[1] = (uint8_t)(result >> 8);
    return 2;
}

void samd_peripherals_adc_setup(struct adc_sync_descriptor *descr, Adc *adc)
{
    uint32_t word = *(const volatile uint32_t *)(SW0_ADDR + FUSES_SW0_WORD_0_OFFSET);
    uint32_t biascomp, biasrefbuf, biasr2r;
    if (adc == ADC0) {
        MCLK->APBDMASK.reg |= MCLK_APBDMASK_ADC0;
        connect_gclk_to_peripheral(CLK_GEN_8MHZ, ADC0_GCLK_ID);
        biascomp = (word & FUSES_SW0_WORD_0_ADC0_BIASCOMP_Msk) >> FUSES_SW0_WORD_0_ADC0_BIASCOMP_Pos;
        biasrefbuf = (word & FUSES_SW0_WORD_0_ADC0_BIASREFBUF_Msk) >> FUSES_SW0_WORD_0_ADC0_BIASREFBUF_Pos;
        biasr2r = (word & FUSES_SW0_WORD_0_ADC0_BIASR2R_Msk) >> FUSES_SW0_WORD_0_ADC0_BIASR2R_Pos;
    } else {
        MCLK->APBDMASK.reg |= MCLK_APBDMASK_ADC1;
        connect_gclk_to_peripheral(CLK_GEN_8MHZ, ADC1_GCLK_ID);
        biascomp = (word & FUSES_SW0_WORD_0_ADC1_BIASCOMP_Msk) >> FUSES_SW0_WORD_0_ADC1_BIASCOMP_Pos;
        biasrefbuf = (word & FUSES_SW0_WORD_0_ADC1_BIASREFBUF_Msk) >> FUSES_SW0_WORD_0_ADC1_BIASREFBUF_Pos;
        biasr2r = (word & FUSES_SW0_WORD_0_ADC1_BIASR2R_Msk) >> FUSES_SW0_WORD_0_ADC1_BIASR2R_Pos;
    }
    adc_sync_init(descr, adc, 0);
    adc->CALIB.reg = ADC_CALIB_BIASCOMP(biascomp) | ADC_CALIB_BIASREFBUF(biasrefbuf) | ADC_CALIB_BIASR2R(biasr2r);
}
