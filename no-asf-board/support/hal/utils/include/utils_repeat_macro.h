// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// REPEAT_MACRO(m, 0, count) expands m(~, 0) ... m(~, count-1) with literal indices.
#pragma once
#define BW_REPEAT_0(m)
#define BW_REPEAT_1(m) m(~, 0)
#define BW_REPEAT_2(m) BW_REPEAT_1(m) m(~, 1)
#define BW_REPEAT_3(m) BW_REPEAT_2(m) m(~, 2)
#define BW_REPEAT_4(m) BW_REPEAT_3(m) m(~, 3)
#define BW_REPEAT_5(m) BW_REPEAT_4(m) m(~, 4)
#define BW_REPEAT_CAT(a, b) a##b
#define BW_REPEAT_X(count) BW_REPEAT_CAT(BW_REPEAT_, count)
#define BW_REPEAT_FIRST_0(m, count) BW_REPEAT_X(count)(m)
#define BW_REPEAT_CAT3(a, b) BW_REPEAT_FIRST_##b
#define REPEAT_MACRO(m, first, count) BW_REPEAT_CAT3(~, first)(m, count)
