// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Source-selected actual target_reset caller; not a board/peripheral reset.
#include "Runtime.h"
#include "codal_target_hal.h"

extern "C" {
struct ResetPersistent {
    unsigned magic, boots, mask, basepri, faultmask, control, ipsr, msp;
    unsigned xpsr, lr, invalid_returned, reset_returned;
};
__attribute__((section(".noinit"))) volatile ResetPersistent runtime_reset_persistent;
volatile unsigned runtime_reset_data = 0x11223344;
volatile unsigned runtime_reset_bss;
volatile unsigned runtime_reset_stage;
}
static volatile unsigned &reg(unsigned address) {
    return *reinterpret_cast<volatile unsigned *>(address);
}
static void require(bool condition, unsigned reason) {
    if (!condition) target_panic(reason);
}
extern "C" __attribute__((noinline)) void runtime_reset_invalid_key() {
    reg(0xe000ed0c) = 4; // SYSRESETREQ without VECTKEY must not reset.
    __asm__ volatile("dsb sy" : : : "memory");
    runtime_reset_persistent.invalid_returned = 1;
}
extern "C" void runtime_probe() {
    volatile ResetPersistent &p = runtime_reset_persistent;
    require(runtime_reset_data == 0x11223344 && runtime_reset_bss == 0, 0xc90);
    require(runtime_status == 0 && p.control == 0 && p.ipsr == 0, 0xc91);
    require(p.msp == 0x20030000, 0xc92);
    require(p.xpsr == 0x01000000 && p.lr == 0xffffffff, 0xc9c);
    if (p.boots == 1) {
        require(p.magic == 0 && p.mask == 0 && p.basepri == 0 && p.faultmask == 0,
                0xc93); // Fresh simulated RAM/core, not a physical POR promise.
        p.magic = 0x42575253;
        runtime_reset_data = 0xaabbccdd;
        runtime_reset_bss = 0x55667788;
        runtime_reset_stage = 1;
        runtime_reset_invalid_key();
        require(p.boots == 1 && p.invalid_returned == 1 &&
                runtime_reset_data == 0xaabbccdd && runtime_reset_bss == 0x55667788,
                0xc94);
        runtime_reset_stage = 2;
        // Privileged, masked core only; no active peripheral transfer is armed.
        reg(0xe000ed08) = 0x20000000; // A deliberately unusable retained VTOR.
        reg(0xe000ed10) = 0x10;
        reg(0xe000ed18) = 0x80808080;
        reg(0xe000ed1c) = 0x80808080;
        reg(0xe000ed20) = 0x80808080;
        reg(0xe000e100) = 1;
        reg(0xe000e200) = 1;
        reg(0xe000e014) = 31;
        reg(0xe000e018) = 0;
        reg(0xe000e010) = 5;
        unsigned priority = 0x80, fault = 1;
        __asm__ volatile("msr basepri, %0\nmsr faultmask, %1" : :
                         "r"(priority), "r"(fault) : "memory");
        runtime_reset_stage = 3;
        target_reset();
        p.reset_returned = 1;
        target_panic(0xc95);
    }
    require(p.boots == 2 && p.magic == 0x42575253 &&
            p.invalid_returned == 1 && p.reset_returned == 0, 0xc96);
    // These are captured by startup BEFORE its CPSID or data/BSS loops.
    require(p.mask == 0 && p.basepri == 0 && p.faultmask == 0, 0xc97);
    require(reg(0xe000ed08) == 0 && reg(0xe000ed10) == 0, 0xc98);
    require(reg(0xe000ed18) == 0 && reg(0xe000ed1c) == 0 &&
            reg(0xe000ed20) == 0, 0xc99);
    require(reg(0xe000e100) == 0 && reg(0xe000e200) == 0, 0xc9a);
    require((reg(0xe000e010) & 7) == 0, 0xc9b);
    runtime_reset_stage = 4;
    runtime_status = 0x600d;
    while (true) __asm__ volatile("nop" : : : "memory");
}
