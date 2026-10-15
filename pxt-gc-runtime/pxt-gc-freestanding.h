// SPDX-License-Identifier: MIT
#pragma once
#include "platform_includes.h"
#include "codal_target_hal.h"
#include "CodalCompat.h"
typedef __INTPTR_TYPE__ intptr_t;
typedef __PTRDIFF_TYPE__ ptrdiff_t;
#define offsetof(type, field) __builtin_offsetof(type, field)
extern "C" {
size_t strlen(const char *);
int strcmp(const char *, const char *);
void *memmove(void *, const void *, size_t);
}
inline void *operator new(size_t, void *p) noexcept { return p; }
inline void *operator new[](size_t, void *p) noexcept { return p; }

static inline unsigned bw_pxt_exception_number() {
    unsigned number;
    __asm__ volatile("mrs %0, ipsr" : "=r"(number) : : "memory");
    return number & 0x1ffu;
}
#define PXT_IN_ISR() bw_pxt_exception_number()
