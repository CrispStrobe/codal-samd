// SPDX-License-Identifier: MIT
#pragma once
#if !defined(BW_PXT_SETTINGS_COMPILE) || !defined(SAMD51)
#error "Select the SAMD51 PXT settings compile-only profile explicitly"
#endif
// Explicit service-only declarations, not the original USB/WebUSB board.
#include "../runtime-services/platform_includes.h"
