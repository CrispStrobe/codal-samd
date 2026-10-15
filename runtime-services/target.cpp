// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "Runtime.h"
#include "CodalFiber.h"
#include "CodalHeapAllocator.h"
#include "Timer.h"
#include "InterruptMask.h"
#include "PairedIrqState.h"

using Mask = brickwright_simulation::InterruptMask;
using Pairs = brickwright_simulation::PairedIrqState<>;
static Pairs pairs;
static unsigned bootContext;
extern "C" {
volatile unsigned runtime_status = 0, runtime_switches = 0, runtime_restores = 0;
volatile unsigned runtime_clock_irqs = 0;
extern unsigned _heap_start;
}
// Match CodalHeapAllocator.h's actual C++ linkage, not the linker symbol's C ABI.
PROCESSOR_WORD_TYPE codal_heap_start = reinterpret_cast<unsigned>(&_heap_start);
static unsigned exception() {
    unsigned value;
    __asm__ volatile("mrs %0, ipsr" : "=r"(value) : : "memory");
    return value;
}
static Pairs::Owner owner() {
    const unsigned ipsr = exception();
    return {ipsr, ipsr ? 0 : reinterpret_cast<unsigned>(
        codal::currentFiber ? static_cast<void *>(codal::currentFiber) : &bootContext)};
}
extern "C" void target_panic(int reason) {
    Mask::restore(1);
    runtime_status = 0xe0000000u | unsigned(reason);
    while (true) __asm__ volatile("nop" : : : "memory");
}
static void apply(Pairs::Transition transition) {
    if (!transition.ok()) target_panic(0xd00 + int(transition.fault));
    Mask::restore(transition.restoreMask);
}
extern "C" void target_disable_irq() {
    const unsigned saved = Mask::saveAndDisable();
    apply(pairs.enter(saved, owner()));
}
extern "C" void target_enable_irq() {
    const unsigned saved = Mask::saveAndDisable();
    apply(pairs.leave(saved, owner()));
}
extern "C" void bw_runtime_require_switch() {
    if (Mask::read() || exception() || !pairs.maySwitchContext()) target_panic(0xd10);
}
extern "C" void runtime_fault() { target_panic(0xf00d); }
extern "C" void __cxa_pure_virtual() { target_panic(0xd11); }

// Match the independently retained CortexContextSwitch.s offsets exactly.
struct Tcb { uint32_t r[13], sp, lr, stackBase; };
static_assert(sizeof(Tcb) == 64, "retained assembly TCB layout");
extern "C" void *tcb_allocate() {
    void *value = malloc(sizeof(Tcb));
    if (!value) target_panic(0xd12);
    return memset(value, 0, sizeof(Tcb));
}
extern "C" uint32_t fiber_initial_stack_base() { return DEVICE_STACK_BASE; }
extern "C" void tcb_configure_lr(void *tcb, uint32_t value) {
    static_cast<Tcb *>(tcb)->lr = value | 1;
}
extern "C" void tcb_configure_sp(void *tcb, uint32_t value) {
    static_cast<Tcb *>(tcb)->sp = value;
}
extern "C" void tcb_configure_stack_base(void *tcb, uint32_t value) {
    static_cast<Tcb *>(tcb)->stackBase = value;
}
extern "C" uint32_t tcb_get_stack_base(void *tcb) {
    return static_cast<Tcb *>(tcb)->stackBase;
}
extern "C" uint32_t tcb_get_sp(void *tcb) { return static_cast<Tcb *>(tcb)->sp; }
extern "C" uint32_t get_current_sp() {
    uint32_t value;
    __asm__ volatile("mov %0, sp" : "=r"(value) : : "memory");
    return value;
}
extern "C" void tcb_configure_args(void *tcb, uint32_t ep, uint32_t cp, uint32_t pm) {
    Tcb &context = *static_cast<Tcb *>(tcb);
    context.r[0] = ep;
    context.r[1] = cp;
    context.r[2] = pm;
}
static void *newMemory(size_t bytes) {
    // Even a zero-size new expression needs a live allocation. The actual raw
    // heap retains its panic-on-exhaustion policy; arithmetic/NULL refusals must
    // also stop construction, never pass NULL to a nonthrowing constructor.
    void *memory = malloc(bytes ? bytes : 1);
    if (!memory) target_panic(0xc73);
    return memory;
}
void *operator new(size_t bytes) { return newMemory(bytes); }
void *operator new[](size_t bytes) { return newMemory(bytes); }
void operator delete(void *memory) noexcept { free(memory); }
void operator delete[](void *memory) noexcept { free(memory); }
void operator delete(void *memory, size_t) noexcept { free(memory); }
void operator delete[](void *memory, size_t) noexcept { free(memory); }
extern "C" void *memset(void *memory, int value, size_t bytes) {
    unsigned char *p = static_cast<unsigned char *>(memory);
    for (size_t i = 0; i < bytes; ++i) p[i] = static_cast<unsigned char>(value);
    return memory;
}
extern "C" void *memcpy(void *destination, const void *source, size_t bytes) {
    unsigned char *to = static_cast<unsigned char *>(destination);
    const unsigned char *from = static_cast<const unsigned char *>(source);
    for (size_t i = 0; i < bytes; ++i) to[i] = from[i];
    return destination;
}
extern "C" void __aeabi_memcpy(void *d, const void *s, size_t n) { memcpy(d, s, n); }
extern "C" int memcmp(const void *left, const void *right, size_t bytes) {
    const unsigned char *a = static_cast<const unsigned char *>(left);
    const unsigned char *b = static_cast<const unsigned char *>(right);
    for (size_t i = 0; i < bytes; ++i)
        if (a[i] != b[i]) return int(a[i]) - int(b[i]);
    return 0;
}
extern "C" void __aeabi_memcpy4(void *d, const void *s, size_t n) { memcpy(d, s, n); }
extern "C" void __aeabi_memclr(void *d, size_t n) { memset(d, 0, n); }
extern "C" void __aeabi_memclr4(void *d, size_t n) { memset(d, 0, n); }
extern "C" void target_wait_us(uint32_t us) {
    if (!brickwright_runtime::clock || us > 10000000) target_panic(0xd13);
    const uint32_t start = brickwright_runtime::clock->captureCounter();
    while (brickwright_runtime::clock->captureCounter() - start < us)
        __asm__ volatile("nop" : : : "memory");
}
extern "C" void target_wait(uint32_t ms) {
    if (ms > 10000) target_panic(0xd14);
    target_wait_us(ms * 1000);
}
extern "C" void target_wait_for_event() {
    bw_runtime_require_switch();
    __asm__ volatile("wfi" : : : "memory");
}
extern "C" void target_deepsleep() { target_panic(0xd15); }
#if defined(BW_SIMULATION_CORE_RESET)
// Explicit simulation core-reset profile only. This requests AIRCR reset;
// it does not implement a physical SAMD peripheral-reset controller.
extern "C" void target_reset() {
    Mask::saveAndDisable();
    volatile uint32_t &aircr = *reinterpret_cast<volatile uint32_t *>(0xe000ed0c);
    const uint32_t grouping = aircr & 0x700u;
    __asm__ volatile("dsb sy" : : : "memory");
    aircr = 0x05fa0004u | grouping;
    __asm__ volatile("dsb sy" : : : "memory");
    while (true) __asm__ volatile("nop" : : : "memory");
}
#else
extern "C" void target_reset() { target_panic(0xd16); }
#endif
