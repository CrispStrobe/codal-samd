// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// The C++ runtime entry points the selected sources need; no libstdc++ is linked.
// Allocation goes to CODAL's heap (malloc aliases device_malloc), which panics on
// exhaustion under DEVICE_PANIC_HEAP_FULL; exceptions are disabled.
#include <stddef.h>
#include <stdlib.h>

void *operator new(size_t size) { return malloc(size); }
void *operator new[](size_t size) { return malloc(size); }
void operator delete(void *memory) noexcept { free(memory); }
void operator delete[](void *memory) noexcept { free(memory); }
void operator delete(void *memory, size_t) noexcept { free(memory); }
void operator delete[](void *memory, size_t) noexcept { free(memory); }

// Static objects are never destroyed: firmware does not exit.
extern "C" int __aeabi_atexit(void *, void (*)(void *), void *) { return 0; }
