// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Authored GC/service caller, NOT a generated Arcade program or board image.
#include "pxtbase-storage.h"
#include "native-aligned-heap.h"
#include "Runtime.h"
#include "MessageBus.h"
#include "Timer.h"
#include "CodalFiber.h"
#include "CodalHeapAllocator.h"
#include "InterruptMask.h"

namespace pxt {
ThreadContext *pushThreadContext(void *sp, void *endSP);
void popThreadContext(ThreadContext *ctx);
// A bounded authored metadata fragment for this GC caller only. The real
// getConfig reads this table; it is not a generated pointers.cpp substitute.
alignas(4) static uint32_t gcHeader[16];
const uintptr_t functionsAndBytecode[] = {0x4210, uintptr_t(gcHeader)};
static int gcConfig[] = {CFG_SYSTEM_HEAP_BYTES, 32768, CFG_RAM_BYTES, 192 * 1024, CFG_SETTINGS_SIZE_DEFL, 32768, CFG_SETTINGS_SIZE, 0, 0, 0};
static TValue gcGlobals[2];
}
using namespace pxt;
using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
extern "C" {
volatile unsigned gc_probe_stage = 0, gc_probe_checks = 0;
volatile unsigned gc_worker_ready = 0, gc_worker_done = 0;
volatile unsigned gc_public_pointer = 0, gc_native_pointer = 0;
}
static void require(bool ok, unsigned reason) {
    if (!ok) target_panic(reason);
    ++gc_probe_checks;
}
static RefRefLocal *object(int value) {
    auto *r = new (gcAllocate(sizeof(RefRefLocal))) RefRefLocal();
    // All authored probe values are small integers; use the actual PXT literal
    // encoding rather than linking unrelated floating-point conversion code.
    r->v = TAG_NUMBER(value);
    return r;
}
static void pressure() {
    // Force collections and reuse without retaining the temporary objects.
    for (unsigned round = 0; round < 5; ++round) {
        for (unsigned i = 0; i < 128; ++i) object(int(i));
        gc(0);
    }
}
static void worker() {
    volatile TValue root = (TValue)object(1234);
    auto *slot = const_cast<TValue *>(&root);
    auto *ctx = pushThreadContext(slot + 1, slot);
    // This C++ caller explicitly publishes its actual stack span, as generated
    // callers must do. The retained scanner maps the real copied fiber stack.
    ctx->stack.top = slot;
    gc_worker_ready = 1;
    require(fiber_wait_for_event(72, 1) == DEVICE_OK, 0xa01);
    require(((RefRefLocal *)root)->v == TAG_NUMBER(1234), 0xa02);
    popThreadContext(ctx);
    gc_worker_done = 1;
    Event(73, 1);
}
#include "reset-caller.cpp.inc"
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    gcHeader[0] = 0x923b8e70;
    bytecode = reinterpret_cast<uint16_t *>(gcHeader);
    bytecode[16] = 2;
    bytecode[17] = sizeof(gcHeader) / 8;
    gcHeader[9] = uintptr_t(gcConfig);
    bytecode[21] = 0;
    globals = gcGlobals;
    require(getConfig(CFG_SYSTEM_HEAP_BYTES, -1) == 32768, 0xa03);
    // Initialize the actual raw heap before GC_GET_HEAP_SIZE queries it.
    void *raw = device_malloc(4);
    require(raw != nullptr, 0xa04);
    device_free(raw);
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    Mask::restore(0);
    (void)timer;
    gc_probe_stage = 1;

    auto *managedPublic = static_cast<unsigned char *>(malloc(23));
    auto *native = static_cast<unsigned char *>(bw_native_alloc(23));
    require(managedPublic && native && managedPublic != native, 0xa05);
    gc_public_pointer = uintptr_t(managedPublic);
    gc_native_pointer = uintptr_t(native);
    require((uintptr_t(native) & 7) == 0, 0xa06);
    for (unsigned i = 0; i < 23; ++i) managedPublic[i] = native[i] = i + 1;
    require(bw_native_realloc(native, ~size_t(0)) == nullptr, 0xa07);
    for (unsigned i = 0; i < 23; ++i) require(native[i] == i + 1, 0xa08);
    native = static_cast<unsigned char *>(bw_native_realloc(native, 51));
    require(native && (uintptr_t(native) & 7) == 0, 0xa09);
    managedPublic = static_cast<unsigned char *>(realloc(managedPublic, 51));
    require(managedPublic != nullptr, 0xa0a);
    for (unsigned i = 0; i < 23; ++i)
        require(managedPublic[i] == i + 1 && native[i] == i + 1, 0xa0b);
    free(managedPublic);
    bw_native_free(native);
    gc_probe_stage = 2;

    globals[0] = (TValue)object(987);
    volatile TValue root = (TValue)object(456);
    auto *slot = const_cast<TValue *>(&root);
    auto *ctx = pushThreadContext(slot + 1, slot);
    ctx->stack.top = slot;
    pressure();
    require(((RefRefLocal *)root)->v == TAG_NUMBER(456), 0xa0c);
    require(((RefRefLocal *)globals[0])->v == TAG_NUMBER(987), 0xa0d);
    require(create_fiber(worker) != nullptr, 0xa0e);
    while (!gc_worker_ready) schedule();
    gc_probe_stage = 3;
    pressure();
    require(((RefRefLocal *)root)->v == TAG_NUMBER(456), 0xa0f);
    Event(72, 1);
    while (!gc_worker_done) schedule();
    require(gc_worker_done == 1, 0xa10);
    popThreadContext(ctx);
    globals[0] = nullptr;
    gc(0);
    gc_probe_stage = 4;
    storage_run();
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
