// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Separate finite authored caller: NOT real OOM, allocated DMA or native Arcade.
#include "pxtbase-storage.h"
#include "Runtime.h"
#include "native-aligned-heap.h"
#include "InterruptMask.h"
using namespace pxt;
using Mask = brickwright_simulation::InterruptMask;
namespace pxt {
alignas(4) static uint32_t churnHeader[16];
const uintptr_t functionsAndBytecode[] = {0x4210, uintptr_t(churnHeader)};
static int churnConfig[] = {CFG_SYSTEM_HEAP_BYTES, 32768, CFG_RAM_BYTES, 192 * 1024, 0, 0};
static TValue churnGlobals[2];
}
struct Live { unsigned pointer, size, seed; };
extern "C" {
volatile Live churn_live[8] = {};
volatile unsigned churn_checks = 0, churn_round = 0, churn_sequence = 0;
volatile unsigned churn_operation = 0, churn_slot = 0, churn_request = 0;
volatile unsigned churn_old = 0, churn_prior_size = 0, churn_return = 0;
volatile unsigned churn_boundaries = 0;
__attribute__((noinline)) void authored_churn_boundary() {
    ++churn_boundaries;
    __asm__ volatile("" : : : "memory");
}
}
static void require(bool ok, unsigned reason) {
    if (!ok) target_panic(reason);
    ++churn_checks;
}
static unsigned char pattern(unsigned seed, unsigned offset) {
    return static_cast<unsigned char>(seed + offset * 13);
}
static bool content(unsigned pointer, unsigned size, unsigned seed) {
    auto *bytes = reinterpret_cast<volatile unsigned char *>(pointer);
    bool ok = true;
    for (unsigned i = 0; i < size; ++i) ok = ok && bytes[i] == pattern(seed, i);
    return ok;
}
static void fill(unsigned slot) {
    auto &s = churn_live[slot];
    auto *bytes = reinterpret_cast<volatile unsigned char *>(s.pointer);
    for (unsigned i = 0; i < s.size; ++i) bytes[i] = pattern(s.seed, i);
}
static void checkAll() {
    bool ok = true;
    for (unsigned i = 0; i < 8; ++i) {
        auto &a = churn_live[i];
        if (!a.pointer) { ok = ok && a.size == 0; continue; }
        ok = ok && content(a.pointer, a.size, a.seed);
        for (unsigned j = 0; j < i; ++j) {
            auto &b = churn_live[j];
            if (b.pointer) ok = ok && (a.pointer + a.size <= b.pointer || b.pointer + b.size <= a.pointer);
        }
    }
    require(ok, 0xd10);
}
static void issue(unsigned operation, unsigned slot, unsigned size) {
    churn_operation = operation;
    churn_slot = slot;
    churn_request = size;
    churn_old = slot < 8 ? churn_live[slot].pointer : 0;
    churn_prior_size = slot < 8 ? churn_live[slot].size : 0;
    churn_return = 0;
    ++churn_sequence;
}
static void allocate(unsigned slot, unsigned size) {
    issue(1, slot, size);
    void *p = slot < 4 ? malloc(size) : bw_native_alloc(size);
    churn_return = uintptr_t(p);
    require(p && (uintptr_t(p) & (slot < 4 ? 3 : 7)) == 0, 0xd11);
    churn_live[slot].pointer = uintptr_t(p);
    churn_live[slot].size = size;
    churn_live[slot].seed = 0x20 + slot * 19 + churn_round * 7;
    fill(slot);
    checkAll();
    authored_churn_boundary();
}
static void resize(unsigned slot, unsigned size) {
    issue(2, slot, size);
    void *old = reinterpret_cast<void *>(churn_old);
    void *p = slot < 4 ? realloc(old, size) : bw_native_realloc(old, size);
    churn_return = uintptr_t(p);
    require(p && (uintptr_t(p) & (slot < 4 ? 3 : 7)) == 0, 0xd12);
    require(content(uintptr_t(p), churn_prior_size < size ? churn_prior_size : size,
                    churn_live[slot].seed), 0xd13);
    churn_live[slot].pointer = uintptr_t(p);
    churn_live[slot].size = size;
    fill(slot);
    checkAll();
    authored_churn_boundary();
}
static void refuse(unsigned slot) {
    issue(4, slot, ~size_t(0));
    churn_return = uintptr_t(bw_native_realloc(reinterpret_cast<void *>(churn_old), ~size_t(0)));
    require(churn_return == 0, 0xd14);
    checkAll(); // All original payload bytes remain live, not just one sentinel.
    authored_churn_boundary();
}
static void release(unsigned slot) {
    issue(3, slot, 0);
    void *p = reinterpret_cast<void *>(churn_old);
    if (slot < 4) free(p); else bw_native_free(p);
    churn_live[slot].pointer = 0;
    churn_live[slot].size = 0;
    churn_live[slot].seed = 0;
    checkAll();
    authored_churn_boundary();
}
static void collect() {
    issue(5, 8, 0);
    gc(0); // Actual collection with permanent public allocations still live.
    checkAll();
    authored_churn_boundary();
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    churnHeader[0] = 0x923b8e70;
    bytecode = reinterpret_cast<uint16_t *>(churnHeader);
    bytecode[16] = 2;
    bytecode[17] = sizeof(churnHeader) / 8;
    churnHeader[9] = uintptr_t(churnConfig);
    bytecode[21] = 0;
    globals = churnGlobals;
    require(getConfig(CFG_SYSTEM_HEAP_BYTES, -1) == 32768, 0xd01);
    // Arithmetic refusal is deliberately NOT claimed as physical/raw OOM.
    for (unsigned op = 6; op <= 9; ++op) {
        unsigned size = (op == 6 || op == 8) ? 0 : ~size_t(0);
        issue(op, 8, size);
        churn_return = uintptr_t(op < 8 ? bw_native_alloc(size) : bw_native_realloc(nullptr, size));
        require(churn_return == 0, 0xd02);
        authored_churn_boundary();
    }
    const unsigned sizes[4] = {1, 7, 17, 33};
    for (unsigned round = 0; round < 8; ++round) {
        churn_round = round;
        for (unsigned slot = 0; slot < 8; ++slot) allocate(slot, sizes[slot % 4] + round);
        for (unsigned slot = 4; slot < 8; ++slot) refuse(slot);
        for (unsigned slot = 0; slot < 8; ++slot) resize(slot, churn_live[slot].size + 9);
        for (unsigned slot = 0; slot < 8; ++slot) resize(slot, (churn_live[slot].size + 1) / 2);
        collect();
        for (unsigned i = 0; i < 4; ++i) { release(i); release(i + 4); }
        collect();
    }
    require(churn_checks == 661 && churn_sequence == 308 && churn_boundaries == 308 &&
            Mask::read() == 1 && bw_pxt_exception_number() == 0, 0xd03);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
