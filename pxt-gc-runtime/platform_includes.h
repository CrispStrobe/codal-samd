// SPDX-License-Identifier: MIT
#pragma once
#if !defined(BW_PXT_GC_LINK) || !defined(SAMD51)
#error "Select the simulation-only linked PXT GC profile explicitly"
#endif
// Explicit modeled service profile, not the original USB/WebUSB board.
#include "../runtime-services/platform_includes.h"
