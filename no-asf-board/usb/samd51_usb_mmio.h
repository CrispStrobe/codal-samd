// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored numeric/register slice from DS60001507F, chapters9,
//10,14,15,32,38. No vendor header or implementation is copied or included.
#pragma once
#if !defined(BW_NO_ASF_USB_LEAF) && !defined(BW_USB_LAYOUT_CONTROL)
#error "Select the no-ASF SAMD51 USB leaf or the pure layout control explicitly"
#endif
#if defined(BW_NO_ASF_USB_LEAF) && (!defined(__ARM_ARCH_7EM__) || !defined(SAMD51) || defined(SAMD21))
#error "The USB leaf requires the selected SAMD51 Cortex-M4 profile"
#endif
#if defined(BW_NO_ASF_USB_LEAF) && (DEVICE_USB != 1 || DEVICE_WEBUSB != 1)
#error "Preserve the generated enabled USB and WebUSB configuration"
#endif
#include <stdint.h>
#include <stddef.h>
struct BwUsbReg8 { volatile uint8_t reg; };
struct BwUsbReg16 { volatile uint16_t reg; };
struct BwUsbReg32 { volatile uint32_t reg; };
union BwUsbControl {
    volatile uint8_t reg;
    struct { volatile uint8_t SWRST:1, ENABLE:1, reserved:4, RUNSTDBY:1, MODE:1; } bit;
};
union BwUsbSync {
    volatile uint8_t reg;
    struct { volatile uint8_t SWRST:1, ENABLE:1, reserved:6; } bit;
};
union BwUsbDeviceControl {
    volatile uint16_t reg;
    struct { volatile uint16_t DETACH:1, UPRSM:1, SPDCONF:2, reserved:12; } bit;
};
union BwUsbPadCalibration {
    volatile uint16_t reg;
    struct { volatile uint16_t TRANSP:5, reserved0:1, TRANSN:5, reserved1:1, TRIM:3, reserved2:1; } bit;
};
union BwUsbPacketSize {
    volatile uint32_t reg;
    struct { volatile uint32_t BYTE_COUNT:14, MULTI_PACKET_SIZE:14, SIZE:3, AUTO_ZLP:1; } bit;
};
struct BwUsbBank {
    BwUsbReg32 ADDR;
    BwUsbPacketSize PCKSIZE;
    BwUsbReg16 EXTREG;
    BwUsbReg8 STATUS_BK;
    uint8_t reserved[5];
};
struct UsbDeviceDescriptor { BwUsbBank DeviceDescBank[2]; };
struct UsbDeviceEndpoint {
    BwUsbReg8 EPCFG;
    uint8_t reserved0[3];
    BwUsbReg8 EPSTATUSCLR, EPSTATUSSET, EPSTATUS, EPINTFLAG, EPINTENCLR, EPINTENSET;
    uint8_t reserved1[22];
};
struct BwUsbDevice {
    BwUsbControl CTRLA;
    uint8_t reserved0;
    BwUsbSync SYNCBUSY;
    uint8_t reserved1[5];
    BwUsbDeviceControl CTRLB;
    BwUsbReg8 DADD;
    uint8_t reserved2[9];
    BwUsbReg16 INTENCLR;
    uint8_t reserved3[2];
    BwUsbReg16 INTENSET;
    uint8_t reserved4[2];
    BwUsbReg16 INTFLAG;
    uint8_t reserved5[2];
    BwUsbReg16 EPINTSMRY;
    uint8_t reserved6[2];
    BwUsbReg32 DESCADD;
    BwUsbPadCalibration PADCAL;
    uint8_t reserved7[214];
    UsbDeviceEndpoint DeviceEndpoint[8];
};
struct BwUsbRegisters { union { BwUsbDevice DEVICE; BwUsbDevice HOST; }; };
union BwUsbPinConfig {
    volatile uint8_t reg;
    struct { volatile uint8_t PMUXEN:1, reserved:7; } bit;
};
struct BwUsbPortGroup {
    uint8_t reserved0[48];
    BwUsbReg8 PMUX[16];
    BwUsbPinConfig PINCFG[32];
    uint8_t reserved1[32];
};
struct BwUsbPort { BwUsbPortGroup Group[2]; };
union BwUsbGclkSync {
    volatile uint32_t reg;
    struct { volatile uint32_t reserved0:2, GENCTRL0:1, reserved1:29; } bit;
};
struct BwUsbGclk {
    uint8_t reserved0[4];
    BwUsbGclkSync SYNCBUSY;
    uint8_t reserved1[120];
    BwUsbReg32 PCHCTRL[48];
};
union BwUsbAhbMask {
    volatile uint32_t reg;
    struct { volatile uint32_t reserved0:10, USB_:1, reserved1:21; } bit;
};
union BwUsbApbbMask {
    volatile uint32_t reg;
    struct { volatile uint32_t USB_:1, reserved:31; } bit;
};
struct BwUsbMclk {
    uint8_t reserved0[16];
    BwUsbAhbMask AHBMASK;
    uint32_t reserved1;
    BwUsbApbbMask APBBMASK;
};
#define USB reinterpret_cast<volatile BwUsbRegisters *>(0x41000000u)
#define PORT reinterpret_cast<volatile BwUsbPort *>(0x41008000u)
#define GCLK reinterpret_cast<volatile BwUsbGclk *>(0x40001c00u)
#define MCLK reinterpret_cast<volatile BwUsbMclk *>(0x40000800u)
#define PIN_PA24H_USB_DM 24
#define PIN_PA25H_USB_DP 25
#define MUX_PA24H_USB_DM 7
#define MUX_PA25H_USB_DP 7
#define USB_GCLK_ID 10
#define GCLK_PCHCTRL_GEN_GCLK1_Val 1
#define GCLK_PCHCTRL_CHEN_Pos 6
#define USB_FUSES_TRANSN_ADDR 0x00800084u
#define USB_FUSES_TRANSN_Pos 0
#define USB_FUSES_TRANSN_Msk (31u << USB_FUSES_TRANSN_Pos)
#define USB_FUSES_TRANSP_ADDR 0x00800084u
#define USB_FUSES_TRANSP_Pos 5
#define USB_FUSES_TRANSP_Msk (31u << USB_FUSES_TRANSP_Pos)
#define USB_FUSES_TRIM_ADDR 0x00800084u
#define USB_FUSES_TRIM_Pos 10
#define USB_FUSES_TRIM_Msk (7u << USB_FUSES_TRIM_Pos)
#define USB_DEVICE_CTRLB_SPDCONF_FS_Val 0
#define USB_DEVICE_CTRLB_DETACH 1u
#define USB_DEVICE_DADD_ADDEN 128u
#define USB_DEVICE_INTFLAG_MASK 0x7ffu
#define USB_DEVICE_INTFLAG_EORST 8u
#define USB_DEVICE_INTENSET_EORST 8u
#define USB_DEVICE_EPINTFLAG_TRCPT0 1u
#define USB_DEVICE_EPINTFLAG_TRCPT1 2u
#define USB_DEVICE_EPINTFLAG_RXSTP 16u
#define USB_DEVICE_EPINTFLAG_STALL0 32u
#define USB_DEVICE_EPINTFLAG_STALL1 64u
#define USB_DEVICE_EPINTFLAG_MASK 127u
#define USB_DEVICE_EPINTENCLR_RXSTP 16u
#define USB_DEVICE_EPINTENSET_RXSTP 16u
#define USB_DEVICE_EPINTENCLR_TRCPT0 1u
#define USB_DEVICE_EPINTENSET_TRCPT0 1u
#define USB_DEVICE_EPSTATUSSET_DTGLOUT 1u
#define USB_DEVICE_EPSTATUSSET_DTGLIN 2u
#define USB_DEVICE_EPSTATUSCLR_STALLRQ0 16u
#define USB_DEVICE_EPSTATUSSET_STALLRQ0 16u
#define USB_DEVICE_EPSTATUSCLR_STALLRQ1 32u
#define USB_DEVICE_EPSTATUSSET_STALLRQ1 32u
#define USB_DEVICE_EPSTATUSCLR_BK0RDY 64u
#define USB_DEVICE_EPSTATUSSET_BK0RDY 64u
#define USB_DEVICE_EPSTATUSCLR_BK1RDY 128u
#define USB_DEVICE_EPSTATUSSET_BK1RDY 128u
#define USB_DEVICE_EPCFG_EPTYPE0_Msk 7u
#define USB_DEVICE_EPCFG_EPTYPE1_Msk 112u
#define USB_DEVICE_EPCFG_EPTYPE0(value) ((value) & 7u)
#define USB_DEVICE_EPCFG_EPTYPE1(value) (((value) & 7u) << 4)
enum { USB_0_IRQn=80, USB_1_IRQn=81, USB_2_IRQn=82, USB_3_IRQn=83 };
static inline void NVIC_EnableIRQ(unsigned irq) {
    *reinterpret_cast<volatile uint32_t *>(0xe000e100u + 4u*(irq/32u)) = 1u << (irq%32u);
}
static_assert(sizeof(BwUsbBank)==16, "descriptor bank stride");
static_assert(sizeof(UsbDeviceDescriptor)==32, "endpoint descriptor stride");
static_assert(sizeof(UsbDeviceEndpoint)==32, "endpoint register stride");
static_assert(offsetof(BwUsbDevice,CTRLB)==8, "device control offset");
static_assert(offsetof(BwUsbDevice,INTENCLR)==0x14, "interrupt clear offset");
static_assert(offsetof(BwUsbDevice,DESCADD)==0x24, "descriptor address offset");
static_assert(offsetof(BwUsbDevice,PADCAL)==0x28, "pad calibration offset");
static_assert(offsetof(BwUsbDevice,DeviceEndpoint)==0x100, "endpoint register offset");
static_assert(offsetof(BwUsbGclk,PCHCTRL)==0x80, "generic clock channel offset");
static_assert(sizeof(BwUsbPortGroup)==0x80, "pin group stride");
static_assert(offsetof(BwUsbPortGroup,PINCFG)==0x40, "pin configuration offset");
static_assert(offsetof(BwUsbMclk,APBBMASK)==0x18, "peripheral clock mask offset");
