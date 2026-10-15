// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "platform_includes.h"
extern "C" void *memmove(void *destination, const void *source, size_t bytes) {
    auto *to = static_cast<unsigned char *>(destination);
    auto *from = static_cast<const unsigned char *>(source);
    const uintptr_t dst = reinterpret_cast<uintptr_t>(to);
    const uintptr_t src = reinterpret_cast<uintptr_t>(from);
    if (dst > src && dst - src < bytes) {
        while (bytes) { --bytes; to[bytes] = from[bytes]; }
    } else {
        for (size_t i = 0; i < bytes; ++i) to[i] = from[i];
    }
    return destination;
}
