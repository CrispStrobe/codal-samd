// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#if !defined(__ARM_ARCH_7EM__) || !defined(BW_NO_ASF_USB_LEAF)
#error "Select the no-ASF SAMD51 USB leaf explicitly"
#endif
// Configuration comes from the byte-bound original forced header. This leaf
// supplies permitted language/library declarations, never USB0 overrides.
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#define CODAL_LOW_LEVEL_VALIDATION 1
extern "C" {
void bw_runtime_require_switch();
void target_panic(int);
extern volatile unsigned runtime_switches, runtime_restores;
}
