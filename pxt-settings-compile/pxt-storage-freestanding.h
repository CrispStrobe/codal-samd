// SPDX-License-Identifier: MIT
#pragma once
#include "platform_includes.h"
#include "codal_target_hal.h"

// Only compiler-native types/macros and C/C++ ABI declarations. No PXT type,
// GC, allocation, libc, math or device implementation is substituted here.
typedef __INTPTR_TYPE__ intptr_t;
typedef __PTRDIFF_TYPE__ ptrdiff_t;
#define offsetof(type, field) __builtin_offsetof(type, field)
extern "C" {
size_t strlen(const char *);
int strcmp(const char *, const char *);
}
void *operator new(size_t, void *) noexcept;
void *operator new[](size_t, void *) noexcept;
