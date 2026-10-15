// SPDX-License-Identifier: MIT
#pragma once
#include "platform_includes.h"

// Compiler-native freestanding declarations, no linked libc implementation.
typedef __INTPTR_TYPE__ intptr_t;
extern "C" { size_t strlen(const char *); }
#define offsetof(type, field) __builtin_offsetof(type, field)

// This generated header contains the exact declaration extracted from the
// admitted public pxtbase.h, not a replacement implementation or a fake PXT type.
#include "pxt-hash-declaration.h"
