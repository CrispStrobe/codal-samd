// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Simulation-only explicitly named native allocation domain for PXT coexistence.
// PXT owns malloc/free/realloc; these pointers must use only bw_native_* APIs.
#include "native-aligned-heap.h"
#include "CodalHeapAllocator.h"

namespace {
struct Header {
    uintptr_t raw;
    size_t requested;
};
static_assert(sizeof(Header) == 8, "selected Cortex-M allocation metadata ABI");
static_assert(sizeof(uintptr_t) == 4 && sizeof(size_t) == 4,
              "selected Cortex-M allocation address/size ABI");
static_assert(DEVICE_MAXIMUM_HEAPS == 1, "selected simulation domain owns one actual heap");
constexpr size_t overhead = sizeof(Header) + 7;
constexpr uintptr_t heapEnd = DEVICE_STACK_BASE - DEVICE_STACK_SIZE;

Header &header(void *memory) {
    const uintptr_t address = reinterpret_cast<uintptr_t>(memory);
    // Check the selected address/layout before inspecting metadata. These are
    // sanity checks, not authentication of arbitrary/invalid caller pointers.
    if ((address & 7) || address < uintptr_t(codal_heap_start) + 12 || address >= heapEnd)
        target_panic(0xc70);
    auto &metadata = *reinterpret_cast<Header *>(address - sizeof(Header));
    if (!metadata.requested || metadata.requested > heapEnd - address ||
        metadata.raw < uintptr_t(codal_heap_start) + sizeof(PROCESSOR_WORD_TYPE) ||
        metadata.raw > address - sizeof(Header) || (metadata.raw & 3) ||
        address - metadata.raw > overhead)
        target_panic(0xc71);
    return metadata;
}
}

extern "C" void *bw_native_alloc(size_t requested) {
    if (!requested || requested > ~size_t(0) - overhead) return nullptr;
    // DEVICE_PANIC_HEAP_FULL remains enabled: actual raw heap exhaustion may
    // panic rather than return NULL. Arithmetic refusal does not change policy.
    void *raw = device_malloc(requested + overhead);
    if (!raw) return nullptr;
    const uintptr_t start = reinterpret_cast<uintptr_t>(raw);
    const uintptr_t aligned = (start + overhead) & ~uintptr_t(7);
    auto *metadata = reinterpret_cast<Header *>(aligned - sizeof(Header));
    metadata->raw = start;
    metadata->requested = requested;
    return reinterpret_cast<void *>(aligned);
}

extern "C" void bw_native_free(void *memory) {
    if (!memory) return;
    const uintptr_t raw = header(memory).raw;
    device_free(reinterpret_cast<void *>(raw));
}

extern "C" void *bw_native_realloc(void *memory, size_t requested) {
    if (!memory) return bw_native_alloc(requested);
    if (!requested) { bw_native_free(memory); return nullptr; }
    const size_t previous = header(memory).requested;
    void *replacement = bw_native_alloc(requested);
    if (!replacement) return nullptr; // The old allocation stays live.
    memcpy(replacement, memory, previous < requested ? previous : requested);
    bw_native_free(memory);
    return replacement;
}
