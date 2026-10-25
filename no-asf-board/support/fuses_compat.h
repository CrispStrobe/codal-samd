// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Calibration fuse locations under the names the selected drivers use, defined
// from the generated ATDF FUSES layout (NVM software calibration area SW0).
#pragma once
#define USB_FUSES_TRANSN_ADDR (SW0_ADDR + FUSES_SW0_WORD_1_OFFSET)
#define USB_FUSES_TRANSN_Pos FUSES_SW0_WORD_1_USB_TRANSN_Pos
#define USB_FUSES_TRANSN_Msk FUSES_SW0_WORD_1_USB_TRANSN_Msk
#define USB_FUSES_TRANSP_ADDR (SW0_ADDR + FUSES_SW0_WORD_1_OFFSET)
#define USB_FUSES_TRANSP_Pos FUSES_SW0_WORD_1_USB_TRANSP_Pos
#define USB_FUSES_TRANSP_Msk FUSES_SW0_WORD_1_USB_TRANSP_Msk
#define USB_FUSES_TRIM_ADDR (SW0_ADDR + FUSES_SW0_WORD_1_OFFSET)
#define USB_FUSES_TRIM_Pos FUSES_SW0_WORD_1_USB_TRIM_Pos
#define USB_FUSES_TRIM_Msk FUSES_SW0_WORD_1_USB_TRIM_Msk
