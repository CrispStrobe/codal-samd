// SPDX-License-Identifier: MIT
#pragma once
#include "../pxt-gc-runtime/pxt-gc-freestanding.h"
// C ABI declarations and compiler-native limits; implementations are separately
// linked and source-bound. These declarations supply no successful operation.
#define INT_MAX __INT_MAX__
#define INT_MIN (-INT_MAX - 1)
#define UINT_MAX (2u * __INT_MAX__ + 1u)
#define NAN __builtin_nan("")
#define HUGE_VAL __builtin_huge_val()
#define INFINITY __builtin_inf()
#define isnan __builtin_isnan
#define isinf __builtin_isinf
#define isnormal __builtin_isnormal
extern "C" {
long strtol(const char *, char **, int);
int abs(int);
double log10(double);
double fmod(double, double);
double trunc(double);
double floor(double);
double ceil(double);
double log(double);
double exp(double);
double pow(double, double);
}
