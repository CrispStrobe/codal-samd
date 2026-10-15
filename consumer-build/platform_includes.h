// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#if !defined(BW_SIMULATION_NVM_CONSUMER_COMPILE)
#error "Select the NVM consumer compile-only profile explicitly"
#endif
#include "../runtime-services/platform_includes.h"

// Declarations only: this profile neither implements nor links libc.
extern "C" {
size_t strlen(const char *);
int strcmp(const char *, const char *);
}
