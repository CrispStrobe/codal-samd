// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Separate expected-fatal native case. NOT public GC OOM or native Arcade.
#include "pxtbase-storage.h"
#include "Runtime.h"
#include "native-aligned-heap.h"
#include "InterruptMask.h"
using namespace pxt;
namespace pxt {
alignas(4) static uint32_t oomHeader[16];
const uintptr_t functionsAndBytecode[] = {0x4210, uintptr_t(oomHeader)};
static int oomConfig[] = {CFG_SYSTEM_HEAP_BYTES, 32768, CFG_RAM_BYTES, 192 * 1024, 0, 0};
static TValue oomGlobals[2];
}
extern "C" {
volatile unsigned oom_stage = 0, oom_old = 0, oom_guard = 0;
volatile unsigned oom_request = 0, oom_checks = 0;
__attribute__((noinline)) void authored_oom_boundary() {
    __asm__ volatile("" : : : "memory");
}
}
static void require(bool ok, unsigned reason) {
    if (!ok) target_panic(reason);
    ++oom_checks;
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    oomHeader[0] = 0x923b8e70;
    bytecode = reinterpret_cast<uint16_t *>(oomHeader);
    bytecode[16] = 2;
    bytecode[17] = sizeof(oomHeader) / 8;
    oomHeader[9] = uintptr_t(oomConfig);
    bytecode[21] = 0;
    globals = oomGlobals;
    require(getConfig(CFG_SYSTEM_HEAP_BYTES, -1) == 32768, 0xd21);
    oom_stage = 1;
    oom_request = 32;
    auto *old = static_cast<unsigned char *>(bw_native_alloc(32));
    require(old && !(uintptr_t(old) & 7), 0xd22);
    oom_old = uintptr_t(old);
    memset(old, 0x69, 32);
    authored_oom_boundary();
    oom_stage = 2;
    oom_request = 0x20000;
    auto *guard = static_cast<unsigned char *>(bw_native_alloc(0x20000));
    require(guard && !(uintptr_t(guard) & 7), 0xd23);
    oom_guard = uintptr_t(guard);
    memset(guard, 0x5a, 0x20000);
    authored_oom_boundary();
    // Both prior allocations remain owned/live. No freed/invalid metadata use,
    // allocation-cap substitution, NULL injection or fatal-policy override.
    oom_stage = 3;
    bw_native_realloc(old, 0x20000);
    target_panic(0xd24); // A return is NOT expected-fatal-OOM success.
}
