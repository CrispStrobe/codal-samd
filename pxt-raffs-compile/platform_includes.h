// SPDX-License-Identifier: MIT
#pragma once
#if !defined(BW_PXT_RAFFS_COMPILE) || !defined(SAMD51)
#error "Select the SAMD51 PXT RAFFS compile-only profile explicitly"
#endif
// Separate service-only compile profile, not the original USB/WebUSB board.
#include "../runtime-services/platform_includes.h"
