// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Separate expected-fatal public GC exhaustion, NOT native OOM or size overflow.
#include "pxtbase-storage.h"
#include "Runtime.h"
#include "CodalHeapAllocator.h"
using namespace pxt;
namespace pxt {
alignas(4) static uint32_t publicOomHeader[16];
const uintptr_t functionsAndBytecode[] = {0x4210, uintptr_t(publicOomHeader)};
static int publicOomConfig[] = {CFG_SYSTEM_HEAP_BYTES, 32768, CFG_RAM_BYTES, 192 * 1024, 0, 0};
static TValue publicOomGlobals[2];
}
extern "C" {
volatile unsigned public_oom_attempt = 0, public_oom_completed = 0;
volatile unsigned public_oom_request = 0, public_oom_checks = 0;
volatile unsigned public_oom_heap_bytes = 0, public_oom_live[32] = {};
__attribute__((noinline)) void authored_public_oom_boundary() {
    __asm__ volatile("" : : : "memory");
}
}
static void require(bool ok, unsigned reason) {
    if (!ok) target_panic(reason);
    ++public_oom_checks;
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    publicOomHeader[0] = 0x923b8e70;
    bytecode = reinterpret_cast<uint16_t *>(publicOomHeader);
    bytecode[16] = 2;
    bytecode[17] = sizeof(publicOomHeader) / 8;
    publicOomHeader[9] = uintptr_t(publicOomConfig);
    bytecode[21] = 0;
    globals = publicOomGlobals;
    require(getConfig(CFG_SYSTEM_HEAP_BYTES, -1) == 32768 &&
            getConfig(CFG_RAM_BYTES, -1) == 192 * 1024, 0xd31);
    // Initialize the real raw heap before the real GC queries its total size.
    void *initial = device_malloc(4);
    require(initial != nullptr, 0xd32);
    device_free(initial);
    public_oom_heap_bytes = device_heap_size(0);
    require(public_oom_heap_bytes > 32768 && public_oom_heap_bytes < 192 * 1024, 0xd33);
    for (unsigned i = 0; i < 32; ++i) {
        public_oom_request = 8192;
        // Publish the attempt only after its request is observable. The
        // instruction-by-instruction observer may read between these stores.
        public_oom_attempt = i + 1;
        // All allocations remain permanent/live. 8192 + its array header is
        // below the selected pxtcore.h cap (128 KiB); no cap/policy override.
        void *memory = malloc(8192);
        require(memory && !(uintptr_t(memory) & 3), 0xd34);
        public_oom_live[i] = uintptr_t(memory);
        memset(memory, 0x30 + i, 8192);
        ++public_oom_completed;
        authored_public_oom_boundary();
    }
    target_panic(0xd35); // Unexpected finite survival is NOT an OOM success.
}
