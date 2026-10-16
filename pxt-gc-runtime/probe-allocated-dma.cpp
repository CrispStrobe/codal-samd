// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Initial allocated-buffer completion caller. Actual guest qualification and
// the remaining terminal/lifetime cases are still pending.
#include "pxtbase-storage.h"
#include "native-aligned-heap.h"
#include "Sercom4TxSPI.h"
#include "CodalFiber.h"
#include "CodalHeapAllocator.h"
#include "InterruptMask.h"

namespace pxt {
alignas(4) static uint32_t gcHeader[16];
const uintptr_t functionsAndBytecode[] = {0x4210, uintptr_t(gcHeader)};
static int gcConfig[] = {CFG_SYSTEM_HEAP_BYTES, 32768, CFG_RAM_BYTES, 192 * 1024, 0, 0};
static TValue gcGlobals[2];
}
using namespace pxt;
using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
using Spi = brickwright_runtime::Sercom4TxSPI;
static constexpr unsigned BUFFER_BYTES = 4096;
static constexpr unsigned GCLK = 0x40001c00u + 0x80 + 34 * 4;
static Spi *spi;
static unsigned char *buffer, *retained;
extern "C" {
volatile unsigned gc_dma_stage = 0, gc_dma_checks = 0;
volatile unsigned gc_dma_native = 0, gc_dma_public = 0, gc_dma_replacement = 0;
volatile unsigned gc_dma_callbacks = 0, gc_dma_callback_ipsr = 0;
volatile unsigned gc_dma_main_fiber = 0, gc_dma_pending_before = 0, gc_dma_pending_after = 0;
volatile unsigned gc_dma_callback_body_finished = 0, gc_dma_freed = 0;
}
static void require(bool ok, unsigned reason) {
    if (!ok) target_panic(reason);
    ++gc_dma_checks;
}
static volatile uint32_t &word(unsigned address) {
    return *reinterpret_cast<volatile uint32_t *>(address);
}
static unsigned exception() {
    unsigned value;
    __asm__ volatile("mrs %0, ipsr" : "=r"(value) : : "memory");
    return value;
}
static unsigned pattern(unsigned i) { return (i * 37 + 11) & 255; }
static unsigned remaining() {
    const unsigned address = word(0x4100a038);
    require(address >= 0x20000000 && address <= 0x2002dff0 && address % 16 == 0, 0xf01);
    return *reinterpret_cast<volatile uint16_t *>(address + 2);
}
static void payloads() {
    for (unsigned i = 0; i < BUFFER_BYTES; ++i) require(buffer[i] == pattern(i), 0xf02);
    for (unsigned i = 0; i < 32; ++i) require(retained[i] == i + 1, 0xf03);
}
extern "C" __attribute__((noinline)) void gc_dma_failed(void *, Spi::State::Result) {
    target_panic(0xf04);
}
extern "C" __attribute__((noinline)) void gc_dma_done(void *argument) {
    gc_dma_callback_ipsr = exception();
    require(argument == spi && exception() == 79 && gc_dma_callbacks == 0 &&
            ((word(0x41008090) >> 7) & 1) == 1, 0xf05);
    ++gc_dma_callbacks;
    // The observer must follow the ORIGINAL callback/finish return. This flag
    // records only the end of the body; it does not prove return or retirement.
    Event(74, 1);
    gc_dma_callback_body_finished = 1;
}
extern "C" __attribute__((noinline)) void *gc_dma_allocate_native() {
    void *pointer = bw_native_alloc(BUFFER_BYTES);
    __asm__ volatile("" : "+r"(pointer) : : "memory");
    return pointer;
}
extern "C" __attribute__((noinline)) void gc_dma_release_native(void *pointer) {
    bw_native_free(pointer);
    __asm__ volatile("" : : : "memory");
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    gcHeader[0] = 0x923b8e70;
    bytecode = reinterpret_cast<uint16_t *>(gcHeader);
    bytecode[16] = 2;
    bytecode[17] = sizeof(gcHeader) / 8;
    gcHeader[9] = uintptr_t(gcConfig);
    bytecode[21] = 0;
    globals = gcGlobals;
    void *raw = device_malloc(4);
    require(raw != nullptr && Mask::read() == 1, 0xf06);
    device_free(raw);
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    Mask::restore(0);
    require(fiber_scheduler_running() && currentFiber && exception() == 0, 0xf07);
    gc_dma_main_fiber = uintptr_t(currentFiber);
    spi = new Spi(*timer, *bus);
    require(spi->setFailureHandler(gc_dma_failed) == DEVICE_OK &&
            spi->setFrequency(1000000) == DEVICE_OK, 0xf08);
    buffer = static_cast<unsigned char *>(gc_dma_allocate_native());
    retained = static_cast<unsigned char *>(malloc(32));
    require(buffer && retained && buffer != retained && uintptr_t(buffer) % 8 == 0, 0xf09);
    gc_dma_native = uintptr_t(buffer);
    gc_dma_public = uintptr_t(retained);
    for (unsigned i = 0; i < BUFFER_BYTES; ++i) buffer[i] = pattern(i);
    for (unsigned i = 0; i < 32; ++i) retained[i] = i + 1;
    gc_dma_stage = 1;
    require(fiber_wake_on_event(74, 1) == DEVICE_OK, 0xf0a);
    require(spi->startTransfer(buffer, BUFFER_BYTES, nullptr, 0, gc_dma_done, spi) == DEVICE_OK, 0xf0b);
    // Explicit real modeled clock gating, not interrupt masking or a synthetic
    // callback: retain a genuinely unfinished transfer while GC runs.
    const unsigned clock = word(GCLK);
    word(GCLK) = clock & ~(1u << 6);
    gc_dma_pending_before = remaining();
    require(gc_dma_pending_before > 0 && gc_dma_pending_before < BUFFER_BYTES, 0xf0c);
    gc_dma_stage = 2;
    for (unsigned round = 0; round < 2; ++round) {
        for (unsigned i = 0; i < 128; ++i) {
            auto *value = new (gcAllocate(sizeof(RefRefLocal))) RefRefLocal();
            value->v = TAG_NUMBER(i);
        }
        gc(0);
        payloads();
    }
    gc_dma_pending_after = remaining();
    require(gc_dma_pending_after == gc_dma_pending_before && gc_dma_callbacks == 0, 0xf0d);
    gc_dma_stage = 3;
    word(GCLK) = clock;
    schedule();
    require(gc_dma_callbacks == 1 && gc_dma_callback_body_finished == 1 &&
            uintptr_t(currentFiber) == gc_dma_main_fiber && exception() == 0 &&
            Mask::read() == 0 && remaining() == 0, 0xf0e);
    payloads();
    gc_dma_stage = 4;
    gc_dma_release_native(buffer);
    buffer = nullptr;
    gc_dma_freed = 1;
    // Only the actual returned new allocation may be filled after release.
    auto *replacement = static_cast<unsigned char *>(gc_dma_allocate_native());
    require(replacement && uintptr_t(replacement) % 8 == 0, 0xf0f);
    gc_dma_replacement = uintptr_t(replacement);
    for (unsigned i = 0; i < BUFFER_BYTES; ++i) replacement[i] = 0xa5;
    for (unsigned i = 0; i < BUFFER_BYTES; ++i) require(replacement[i] == 0xa5, 0xf10);
    gc_dma_release_native(replacement);
    free(retained);
    gc_dma_stage = 5;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
