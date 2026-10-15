// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Authored routing caller, not generated Arcade metadata or board firmware.
#include "pxtbase-storage.h"
#include "Runtime.h"
#include "CodalHeapAllocator.h"
#include "InterruptMask.h"

using namespace pxt;
using Mask = brickwright_simulation::InterruptMask;
extern "C" void authored_gc_allocating_scan(RefRefLocal *value);
namespace pxt {
extern uint8_t inGC; // Observe the real collector flag; never assign it.
alignas(4) static uint32_t routingHeader[16];
const uintptr_t functionsAndBytecode[] = {0x4210, uintptr_t(routingHeader)};
static int routingConfig[] = {CFG_SYSTEM_HEAP_BYTES, 32768, CFG_RAM_BYTES, 192 * 1024, 0, 0};
static TValue routingGlobals[2];
// Use the actual PXT descriptor macro/types and real size/print/destroy methods.
// Only this authored object's scan callback allocates; delegate the real scan.
// The descriptor must stay const in flash: GC tag bits exclude RAM vtables.
DEF_VTABLE(routing_vtable, RefRefLocal, ValType::Object,
           (void *)&RefRefLocal::destroy, (void *)&RefRefLocal::print,
           (void *)&authored_gc_allocating_scan, (void *)&RefRefLocal::gcsize)
}
extern "C" {
volatile unsigned routing_stage = 0, routing_checks = 0;
volatile unsigned routing_irq_count = 0, routing_irq_done = 0, routing_scan_count = 0;
volatile unsigned routing_thread_pointer = 0, routing_irq_pointer = 0, routing_scan_pointer = 0;
volatile unsigned routing_irq_exception = 0, routing_irq_mask = 0;
volatile unsigned routing_scan_in_gc = 0;
}
static void require(bool ok, unsigned reason) {
    if (!ok) target_panic(reason);
    ++routing_checks;
}
static volatile unsigned &reg(unsigned address) {
    return *reinterpret_cast<volatile unsigned *>(address);
}
static void barriers() { __asm__ volatile("dsb\n\tisb" : : : "memory"); }
static void payload(unsigned size, volatile unsigned &published, unsigned reason) {
    auto *memory = static_cast<unsigned char *>(malloc(size));
    require(memory != nullptr, reason);
    published = uintptr_t(memory);
    for (unsigned i = 0; i < size; ++i) memory[i] = static_cast<unsigned char>(i ^ size);
    bool valid = true;
    for (unsigned i = 0; i < size; ++i)
        valid = valid && memory[i] == static_cast<unsigned char>(i ^ size);
    require(valid, reason + 1);
    free(memory);
}
extern "C" void authored_gc_routing_irq0() {
    routing_irq_exception = bw_pxt_exception_number();
    routing_irq_mask = Mask::read();
    require(routing_irq_exception == 16 && routing_irq_mask == 0 && inGC == 0, 0xb10);
    ++routing_irq_count;
    payload(29, routing_irq_pointer, 0xb11);
    require(bw_pxt_exception_number() == 16 && Mask::read() == 0 && inGC == 0, 0xb13);
    routing_irq_done = 1;
    // Real NVIC software pending clears on entry; no invented peripheral ack.
}
extern "C" void authored_gc_allocating_scan(RefRefLocal *value) {
    routing_scan_in_gc = inGC;
    require(routing_scan_in_gc != 0 && bw_pxt_exception_number() == 0 && Mask::read() == 1, 0xb20);
    ++routing_scan_count;
    payload(37, routing_scan_pointer, 0xb21);
    RefRefLocal::scan(value);
    require(inGC != 0 && bw_pxt_exception_number() == 0, 0xb23);
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    routing_stage = 1;
    routingHeader[0] = 0x923b8e70;
    bytecode = reinterpret_cast<uint16_t *>(routingHeader);
    bytecode[16] = 2;
    bytecode[17] = sizeof(routingHeader) / 8;
    routingHeader[9] = uintptr_t(routingConfig);
    bytecode[21] = 0;
    globals = routingGlobals;
    require(getConfig(CFG_SYSTEM_HEAP_BYTES, -1) == 32768, 0xb01);
    void *raw = device_malloc(4);
    require(raw != nullptr, 0xb02);
    device_free(raw);
    require(inGC == 0 && Mask::read() == 1, 0xb03);
    payload(17, routing_thread_pointer, 0xb04);
    require((routing_thread_pointer & 3) == 0, 0xb06);

    routing_stage = 2;
    reg(0xe000e180) = 1; // NVIC ICER0
    reg(0xe000e280) = 1; // NVIC ICPR0
    reg(0xe000e100) = 1; // NVIC ISER0
    reg(0xe000e200) = 1; // NVIC ISPR0: actual pending IRQ, while masked
    barriers();
    require((reg(0xe000e200) & 1) != 0 && routing_irq_count == 0, 0xb07);
    Mask::restore(0);
    for (unsigned i = 0; i < 32; ++i) __asm__ volatile("nop" : : : "memory");
    require(routing_irq_count == 1 && routing_irq_done == 1, 0xb08);
    require(bw_pxt_exception_number() == 0 && Mask::read() == 0 &&
            (reg(0xe000e200) & 1) == 0, 0xb09);
    Mask::restore(1);
    reg(0xe000e180) = 1;
    reg(0xe000e280) = 1;
    barriers();
    require((reg(0xe000e100) & 1) == 0 && (reg(0xe000e200) & 1) == 0, 0xb0a);

    routing_stage = 3;
    auto *value = new (gcAllocate(sizeof(RefRefLocal))) RefRefLocal();
    require(value != nullptr, 0xb2f);
    value->v = TAG_NUMBER(777);
    value->setVT(uintptr_t(&routing_vtable));
    globals[0] = (TValue)value;
    gc(0); // Actual collector marks the global and invokes the scan descriptor.
    require(routing_scan_count == 1 && inGC == 0, 0xb30);
    require(globals[0] == (TValue)value && value->v == TAG_NUMBER(777), 0xb31);
    globals[0] = nullptr;
    gc(0);
    require(routing_scan_count == 1 && inGC == 0, 0xb32);

    routing_stage = 4;
    require((reg(0xe000e100) & 1) == 0 && (reg(0xe000e200) & 1) == 0 &&
            Mask::read() == 1 && bw_pxt_exception_number() == 0, 0xb40);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
