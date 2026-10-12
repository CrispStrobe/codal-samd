// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Explicit ARM-only diagnostic platform, NOT the original board configuration.
#pragma once
#if !defined(__arm__) || !defined(BW_SIMULATION_TIMER_CALLER)
#error "Select the ARM simulation-only Timer caller fixture explicitly"
#endif
typedef __UINT8_TYPE__ uint8_t;
typedef __UINT16_TYPE__ uint16_t;
typedef __UINT32_TYPE__ uint32_t;
typedef __UINT64_TYPE__ uint64_t;
typedef __SIZE_TYPE__ size_t;
#define PROCESSOR_WORD_TYPE uint32_t
#define NULL __null
#define DEVICE_DMESG_BUFFER_SIZE 0
#define DEVICE_USB 0
#define DEVICE_WEBUSB 0
#define DEVICE_HEAP_ALLOCATOR 0
#define SCHEDULER_TICK_PERIOD_US 4000
extern "C" void *malloc(size_t);
extern "C" void *memset(void *, int, size_t);
