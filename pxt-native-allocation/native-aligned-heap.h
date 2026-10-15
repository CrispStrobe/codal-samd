// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#if !defined(BW_PXT_NATIVE_ALLOCATION)
#error "Select the distinct PXT native allocation domain explicitly"
#endif
#include "platform_includes.h"

// Never pass PXT managed/public allocations or raw device_* pointers here.
// Buffers returned here stay native-owned, are not GC roots, and must not be
// passed to public free/realloc. Retain GC references separately when needed.
extern "C" {
void *bw_native_alloc(size_t requested);
void bw_native_free(void *memory);
void *bw_native_realloc(void *memory, size_t requested);
}
