// SPDX-License-Identifier: MIT
#pragma once
#if !defined(BW_PXT_FLASH_COMPILE) || !defined(SAMD51)
#error "Select the SAMD51 PXT flash compile-only profile explicitly"
#endif
// This separate compile-only selection is NOT the historical USB/WebUSB board
// configuration. No actual PXT header is shadowed by this platform header.
#include "../runtime-services/platform_includes.h"
