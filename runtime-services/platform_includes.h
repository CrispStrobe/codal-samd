// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#if !defined(__ARM_ARCH_7EM__) || !defined(BW_SIMULATION_CODAL_SERVICES)
#error "Select the Cortex-M4 simulation-only real CODAL service profile explicitly"
#endif
typedef __UINT8_TYPE__ uint8_t;
typedef __UINT16_TYPE__ uint16_t;
typedef __UINT32_TYPE__ uint32_t;
typedef __UINT64_TYPE__ uint64_t;
typedef __INT8_TYPE__ int8_t;
typedef __INT16_TYPE__ int16_t;
typedef __INT32_TYPE__ int32_t;
typedef __INT64_TYPE__ int64_t;
typedef __UINTPTR_TYPE__ uintptr_t;
typedef __SIZE_TYPE__ size_t;
#define PROCESSOR_WORD_TYPE uint32_t
#define NULL __null
#define DEVICE_HEAP_ALLOCATOR 1
#define CODAL_LOW_LEVEL_VALIDATION 1
#define DEVICE_PANIC_HEAP_FULL 1
#define DEVICE_DMESG_BUFFER_SIZE 0
#define DEVICE_STACK_BASE 0x20030000u
#define DEVICE_STACK_SIZE 8192u
#define SCHEDULER_TICK_PERIOD_US 4000
// This service-only image does not select a board, USB or WebUSB. Their later
// required native Arcade build closure is separate; no success stub is supplied.
#define DEVICE_USB 0
#define DEVICE_WEBUSB 0
extern "C" {
void *malloc(size_t);
void free(void *);
void *realloc(void *, size_t);
void *memset(void *, int, size_t);
void *memcpy(void *, const void *, size_t);
void bw_runtime_require_switch();
void target_panic(int);
extern volatile unsigned runtime_switches, runtime_restores;
}
