// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#pragma once
#include "pxtbase-storage.h"
#include "Runtime.h"
#include "CodalHeapAllocator.h"
using namespace pxt;
namespace pxt {
alignas(4) static uint32_t sizeHeader[16];
const uintptr_t functionsAndBytecode[] = {0x4210, uintptr_t(sizeHeader)};
static int sizeConfig[] = {CFG_SYSTEM_HEAP_BYTES, 32768, CFG_RAM_BYTES, 192 * 1024, 0, 0};
static TValue sizeGlobals[2];
}
extern "C" {
volatile unsigned public_size_stage = 0, public_size_request = 0;
volatile unsigned public_size_live = 0, public_size_pointer = 0;
volatile unsigned public_size_bytes = 0, public_size_result = 0;
__attribute__((noinline)) void authored_public_size_boundary() {
    __asm__ volatile("" : : : "memory");
}
}
static void require(bool ok, unsigned reason) {
    if (!ok) target_panic(reason);
}
static void *allocate(unsigned stage, unsigned bytes) {
    public_size_request = bytes;
    public_size_stage = stage;
    return malloc(bytes);
}
static void publish(void *pointer, unsigned bytes, unsigned fill) {
    require(pointer && !(uintptr_t(pointer) & 3), 0xd41);
    memset(pointer, fill, bytes);
    public_size_pointer = uintptr_t(pointer);
    public_size_bytes = bytes;
    authored_public_size_boundary();
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    sizeHeader[0] = 0x923b8e70;
    bytecode = reinterpret_cast<uint16_t *>(sizeHeader);
    bytecode[16] = 2;
    bytecode[17] = sizeof(sizeHeader) / 8;
    sizeHeader[9] = uintptr_t(sizeConfig);
    bytecode[21] = 0;
    globals = sizeGlobals;
    require(getConfig(CFG_SYSTEM_HEAP_BYTES, -1) == 32768 &&
            getConfig(CFG_RAM_BYTES, -1) == 192 * 1024, 0xd42);
    void *initial = device_malloc(4);
    require(initial != nullptr, 0xd43);
    device_free(initial);
    require(allocate(1, 0) == nullptr, 0xd44);
    void *live = allocate(2, 32);
    public_size_live = uintptr_t(live);
    publish(live, 32, 0x69);
    for (unsigned stage = 3; stage <= 5; ++stage) {
        const unsigned bytes = stage == 3 ? 1 : stage == 4 ? 131065 : 131068;
        void *memory = allocate(stage, bytes);
        publish(memory, bytes, 0x40 + stage);
        free(memory);
    }
    // The last request is observed at every actual allocator entry/return.
    // In particular, never fill or dereference a SIZE_MAX requested payload.
    void *result = allocate(6, BW_PUBLIC_SIZE_FINAL);
    public_size_result = uintptr_t(result);
    #if BW_PUBLIC_SIZE_FINAL == 17u
    publish(result, 17, 0x46);
    free(result);
#endif
    runtime_status = 0x600d; // Only the fresh ordinary recovery case may return.
}
