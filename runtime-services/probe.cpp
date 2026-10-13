// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Actual services integration caller; no alternative Event/Heap/Fiber bodies.
#include "Runtime.h"
#include "MessageBus.h"
#include "CodalFiber.h"
#include "Timer.h"
#include "InterruptMask.h"
#include "CodalHeapAllocator.h"

using namespace codal;
using Mask = brickwright_simulation::InterruptMask;
extern "C" {
volatile unsigned runtime_listener_calls = 0, runtime_fiber_started = 0;
volatile unsigned runtime_fiber_woken = 0, runtime_main_woken = 0;
volatile unsigned runtime_worker_identity = 0, runtime_main_identity = 0;
volatile unsigned runtime_listener_exception = 0, runtime_checks = 0;
volatile unsigned runtime_mode = BW_RUNTIME_TEST_MODE;
volatile unsigned runtime_heap_reused = 0;
volatile unsigned runtime_clock_stage = 0, runtime_clock_pending_seen = 0;
volatile unsigned runtime_clock_before = 0, runtime_clock_after = 0;
volatile unsigned runtime_timer_before = 0, runtime_timer_after = 0;
volatile unsigned runtime_ms_before = 0, runtime_ms_after = 0;
volatile unsigned runtime_alignment_first = 0, runtime_alignment_second = 0;
volatile unsigned runtime_alignment_size = 0, runtime_alignment_stage = 0;
volatile unsigned runtime_public_heap_checks = 0, runtime_public_heap_stage = 0;
volatile unsigned runtime_public_heap_old = 0, runtime_public_heap_guard = 0;
}
static void require(bool condition, unsigned reason) {
    if (!condition) target_panic(reason);
}
static void timerListener(Event evt) {
    unsigned ipsr;
    __asm__ volatile("mrs %0, ipsr" : "=r"(ipsr) : : "memory");
    runtime_listener_exception = ipsr;
    require(evt.source == 42 && evt.value == 7 && ipsr == 15, 0xb01);
    ++runtime_listener_calls;
}
static void worker() {
    runtime_worker_identity = reinterpret_cast<unsigned>(currentFiber);
    runtime_fiber_started = 1;
    require(fiber_wait_for_event(42, 7) == DEVICE_OK, 0xb02);
    require(currentFiber && reinterpret_cast<unsigned>(currentFiber) == runtime_worker_identity,
            0xb03);
    runtime_fiber_woken = 1;
    Event(43, 9);
}
static void heap_check(bool condition) {
    require(condition, 0xc62);
    ++runtime_public_heap_checks;
}
static bool aligned_public(void *memory) {
    const uintptr_t address = reinterpret_cast<uintptr_t>(memory);
    return address >= 0x20000000 && address < 0x2002e000 && address % 8 == 0;
}
static void public_heap_controls() {
    const size_t maximum = ~size_t(0);
    heap_check(malloc(0) == NULL);
    heap_check(malloc(maximum) == NULL);
    heap_check(calloc(0, 8) == NULL && calloc(8, 0) == NULL);
    heap_check(calloc(maximum, 2) == NULL);
    free(NULL);
    heap_check(realloc(NULL, 0) == NULL);
    const unsigned sizes[] = {1, 2, 3, 7, 8, 9, 15, 16, 17, 31, 32, 33};
    void *blocks[12];
    for (unsigned round = 0; round < 3; ++round) {
        for (unsigned i = 0; i < 12; ++i) {
            blocks[i] = malloc(sizes[i]);
            heap_check(aligned_public(blocks[i]));
            memset(blocks[i], int(0x20 + i), sizes[i]);
        }
        for (unsigned i = 0; i < 12; ++i) {
            auto *bytes = static_cast<unsigned char *>(blocks[i]);
            for (unsigned j = 0; j < sizes[i]; ++j)
                heap_check(bytes[j] == 0x20 + i);
        }
        // Free alternate blocks before the others, exercising actual heap
        // split/coalescing with simultaneous differently sized allocations.
        for (unsigned i = 0; i < 12; i += 2) free(blocks[i]);
        for (unsigned i = 1; i < 12; i += 2) free(blocks[i]);
    }
    auto *old = static_cast<unsigned char *>(realloc(NULL, 13));
    heap_check(aligned_public(old));
    for (unsigned i = 0; i < 13; ++i) old[i] = static_cast<unsigned char>(i + 1);
    void *refused = realloc(old, maximum);
    heap_check(refused == NULL);
    for (unsigned i = 0; i < 13; ++i) heap_check(old[i] == i + 1);
    auto *grown = static_cast<unsigned char *>(realloc(old, 41));
    heap_check(aligned_public(grown));
    for (unsigned i = 0; i < 13; ++i) heap_check(grown[i] == i + 1);
    auto *shrunk = static_cast<unsigned char *>(realloc(grown, 5));
    heap_check(aligned_public(shrunk));
    for (unsigned i = 0; i < 5; ++i) heap_check(shrunk[i] == i + 1);
    heap_check(realloc(shrunk, 0) == NULL);
    auto *zero = static_cast<unsigned char *>(calloc(9, 3));
    heap_check(aligned_public(zero));
    for (unsigned i = 0; i < 27; ++i) heap_check(zero[i] == 0);
    free(zero);
    struct alignas(8) Words {
        volatile uint64_t first, second;
        Words() : first(0x123456789abcdef0ULL), second(0xfedcba9876543210ULL) {}
    };
    static_assert(alignof(Words) == 8, "actual fundamental object alignment");
    auto *words = new Words();
    heap_check(aligned_public(words));
    heap_check(words->first == 0x123456789abcdef0ULL && words->second == 0xfedcba9876543210ULL);
    words->first ^= words->second;
    heap_check(words->first == 0xece8ece0ece8ece0ULL);
    delete words;
    auto *array = new Words[3];
    heap_check(aligned_public(array));
    for (unsigned i = 0; i < 3; ++i) {
        array[i].second = uint64_t(i + 1) << 40;
        heap_check(array[i].first == 0x123456789abcdef0ULL && array[i].second == uint64_t(i + 1) << 40);
    }
    delete[] array;
    // Raw realloc stays in the raw domain, with rounded payload capacity.
    auto *raw = static_cast<unsigned char *>(device_malloc(5));
    heap_check(raw != NULL);
    memset(raw, 0x6b, 5);
    auto *rawGrown = static_cast<unsigned char *>(device_realloc(raw, 17));
    heap_check(rawGrown != NULL);
    for (unsigned i = 0; i < 5; ++i) heap_check(rawGrown[i] == 0x6b);
    device_free(rawGrown);
    runtime_public_heap_stage = 1;
    runtime_checks = 2;
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
static void public_heap_exhaustion() {
    // Keep the selected fatal OOM policy, and expose the old payload after
    // an actual failed realloc allocation. No artificial allocation failure.
    auto *old = static_cast<unsigned char *>(malloc(32));
    require(aligned_public(old), 0xc63);
    memset(old, 0x69, 32);
    runtime_public_heap_old = reinterpret_cast<unsigned>(old);
    void *guard = malloc(0x20000);
    require(aligned_public(guard), 0xc64);
    runtime_public_heap_guard = reinterpret_cast<unsigned>(guard);
    runtime_public_heap_stage = 2;
    (void)realloc(old, 0x20000);
    target_panic(0xc65); // The actual selected heap must refuse by DEVICE_OOM.
}
static void long_gap_adversary(brickwright_runtime::CoreClockTimer &low, Timer &timer) {
    Mask::restore(1);
    require(runtime_clock_irqs == 0 && runtime_listener_calls == 0, 0xc31);
    require(timer.eventAfterUs(20000, 42, 7) == DEVICE_OK, 0xc32);
    runtime_ms_before = uint32_t(timer.getTime());
    runtime_timer_before = uint32_t(timer.getTimeUs());
    runtime_clock_before = low.captureCounter();
    runtime_clock_stage = 1;
    // Poll only the actual low-level clock. Calling Timer::getTimeUs in this
    // loop would repeatedly sync sigma and hide a long-gap elapsed-time loss.
    while (low.captureCounter() - runtime_clock_before < 80000)
        __asm__ volatile("nop" : : : "memory");
    runtime_clock_after = low.captureCounter();
    runtime_clock_pending_seen =
        (*reinterpret_cast<volatile uint32_t *>(0xe000e010) >> 16) & 1;
    runtime_timer_after = uint32_t(timer.getTimeUs());
    runtime_ms_after = uint32_t(timer.getTime());
    runtime_clock_stage = 2;
    require(Mask::read() == 1 && runtime_clock_pending_seen == 1 &&
            runtime_clock_irqs == 0 && runtime_listener_calls == 0, 0xc33);
    const uint32_t elapsed = runtime_clock_after - runtime_clock_before;
    const uint32_t timerElapsed = runtime_timer_after - runtime_timer_before;
    require(elapsed >= 80000 && elapsed < 81000, 0xc34);
    // The prior unchanged Timer image lost approximately 65536us in the
    // retained original. This candidate must restore both timestamp APIs.
    // The margin bounds call overhead, not RTx or physical clock accuracy.
    require(timerElapsed >= 80000 && timerElapsed <= elapsed + 1000, 0xc35);
    const uint32_t msElapsed = runtime_ms_after - runtime_ms_before;
    require(msElapsed >= 80 && msElapsed <= 82 &&
            msElapsed * 1000 + 1000 >= timerElapsed &&
            msElapsed * 1000 <= timerElapsed + 1000, 0xc3a);
    runtime_clock_stage = 3;
    Mask::restore(0);
    const uint32_t resumed = low.captureCounter();
    while (runtime_listener_calls == 0 && low.captureCounter() - resumed < 2000)
        __asm__ volatile("nop" : : : "memory");
    require(runtime_listener_calls == 1 && runtime_listener_exception == 15, 0xc36);
    require(runtime_clock_irqs == 1 && runtime_switches == 0 && runtime_restores == 0, 0xc37);
    require(timer.cancel(42, 7) == DEVICE_INVALID_PARAMETER, 0xc38);
    runtime_clock_stage = 4;
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
static void clock_adversary(brickwright_runtime::CoreClockTimer &low, Timer &timer) {
    Mask::restore(1);
    require(runtime_clock_irqs == 0 && runtime_listener_calls == 0, 0xb21);
    require(timer.eventAfterUs(1000, 42, 7) == DEVICE_OK, 0xb22);
    runtime_clock_stage = 1;
    const uint32_t start = low.captureCounter();
    // Real DWT time crosses the armed SysTick deadline under actual PRIMASK.
    // No host pend, artificial clock advance or callback/event injection.
    while (low.captureCounter() - start < 2000)
        __asm__ volatile("nop" : : : "memory");
    runtime_clock_pending_seen =
        (*reinterpret_cast<volatile uint32_t *>(0xe000e010) >> 16) & 1;
    require(runtime_clock_pending_seen == 1 && runtime_clock_irqs == 0 &&
            runtime_listener_calls == 0 && Mask::read() == 1, 0xb23);
    require((runtime_mode == 2 ? low.disableIRQ() : low.disable()) == DEVICE_OK, 0xb24);
    runtime_clock_before = low.captureCounter();
    runtime_clock_stage = 2;
    Mask::restore(0); // The already-pending real exception may enter and return.
    for (unsigned i = 0; i < 256; ++i) __asm__ volatile("nop" : : : "memory");
    runtime_clock_after = low.captureCounter();
    runtime_clock_stage = 3;
    require(runtime_listener_calls == 0, 0xb20); // Exact stale-event failure.
    require(runtime_clock_irqs == 1, 0xb25);
    require(runtime_mode == 2 ? runtime_clock_after > runtime_clock_before :
                               runtime_clock_after == runtime_clock_before, 0xb26);
    runtime_clock_stage = 4;
    require((runtime_mode == 2 ? low.enableIRQ() : low.enable()) == DEVICE_OK, 0xb27);
    const uint32_t resumed = low.captureCounter();
    while (runtime_listener_calls == 0 && low.captureCounter() - resumed < 2000)
        __asm__ volatile("nop" : : : "memory");
    require(runtime_listener_calls == 1 && runtime_listener_exception == 15, 0xb28);
    require(runtime_clock_irqs == 2 && runtime_switches == 0 && runtime_restores == 0, 0xb29);
    require(timer.cancel(42, 7) == DEVICE_INVALID_PARAMETER, 0xb2a);
    runtime_clock_stage = 5;
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
extern "C" void runtime_probe() {
    runtime_status = 0x100;
    require(Mask::read() == 1, 0xb04);
    target_disable_irq();
    target_disable_irq();
    target_enable_irq();
    target_enable_irq();
    require(Mask::read() == 1, 0xb05);
    Mask::restore(0);
    void *first = malloc(64);
    require(first != NULL, 0xb06);
    memset(first, 0x5a, 64);
    free(first);
    void *second = malloc(64);
    require(second == first, 0xb07);
    runtime_heap_reused = 1;
    free(second);
    runtime_checks = 1;
    if (runtime_mode == 6 || runtime_mode == 7) {
        Mask::restore(1);
        if (runtime_mode == 6) public_heap_controls();
        else public_heap_exhaustion();
        target_panic(0xc67);
    }
    if (runtime_mode == 5) {
        // Retain both allocations. The unmodified raw heap uses four-byte
        // words and a one-word header; two eight-byte payloads expose its
        // twelve-byte allocation stride without substituting another heap.
        Mask::restore(1);
        void *a = device_malloc(8);
        void *b = device_malloc(8);
        require(a != NULL && b != NULL && a != b, 0xc61);
        runtime_alignment_first = reinterpret_cast<unsigned>(a);
        runtime_alignment_second = reinterpret_cast<unsigned>(b);
        runtime_alignment_size = 8;
        memset(a, 0x5a, 8);
        memset(b, 0xa5, 8);
        runtime_alignment_stage = 1;
        // An expected diagnostic refusal is not allocator admission. Preserve
        // the original guest result before adapting the public allocation ABI.
        require(((runtime_alignment_first | runtime_alignment_second) & 7) == 0, 0xc60);
        device_free(b);
        device_free(a);
        runtime_alignment_stage = 2;
        runtime_checks = 2;
        runtime_status = 0x600d;
        while (true) __asm__ volatile("nop" : : : "memory");
    }
    // CODAL fibers page their active stack. IRQ/global service pointers must
    // refer to stable heap objects, not objects on the shared paged stack.
    auto *low = new brickwright_runtime::CoreClockTimer();
    auto *timer = new Timer(*low);
    auto *bus = new MessageBus();
    scheduler_init(*bus);
    require(fiber_scheduler_running() && currentFiber, 0xb08);
    runtime_main_identity = reinterpret_cast<unsigned>(currentFiber);
    require(bus->listen(42, 7, timerListener, MESSAGE_BUS_LISTENER_IMMEDIATE) == DEVICE_OK,
            0xb09);
    if (runtime_mode == 4) {
        runtime_checks = 2;
        long_gap_adversary(*low, *timer);
        target_panic(0xc39);
    }
    if (runtime_mode == 2 || runtime_mode == 3) {
        runtime_checks = 2;
        clock_adversary(*low, *timer);
        target_panic(0xb2b);
    }
    require(create_fiber(worker) != NULL, 0xb0a);
    runtime_checks = 2;
    if (runtime_mode == 1) {
        target_disable_irq();
        schedule();
        target_panic(0xb0b);
    }
    require(runtime_mode == 0, 0xb0c);
    require(timer->eventAfterUs(20000, 42, 7) == DEVICE_OK, 0xb0d);
    require(fiber_wait_for_event(43, 9) == DEVICE_OK, 0xb0e);
    runtime_main_woken = 1;
    require(runtime_fiber_started == 1 && runtime_fiber_woken == 1 &&
            runtime_listener_calls == 1 && runtime_listener_exception == 15,
            0xb0f);
    require(runtime_worker_identity != runtime_main_identity &&
            reinterpret_cast<unsigned>(currentFiber) == runtime_main_identity,
            0xb10);
    require(runtime_switches >= 3 && runtime_clock_irqs && Mask::read() == 0,
            0xb11);
    require(timer->cancel(42, 7) == DEVICE_INVALID_PARAMETER, 0xb12);
    runtime_checks = 3;
    Mask::restore(1);
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
