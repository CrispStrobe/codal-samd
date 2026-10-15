/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 CrispStrobe contributors
 * Captures actual reset-entry registers before CPSID or C initialization.
 */
.syntax unified
.cpu cortex-m4
.thumb
.section .vectors,"a",%progbits
.balign 1024
.global runtime_vectors
runtime_vectors:
.word _stack_top, reset_entry
.rept 13
.word runtime_fault
.endr
.word runtime_systick
.rept 240
.word runtime_fault
.endr
.global runtime_vectors_end
runtime_vectors_end:
.section .text.reset_entry,"ax",%progbits
.global reset_entry
.type reset_entry,%function
.thumb_func
reset_entry:
    ldr r0, =runtime_reset_persistent
    mrs r1, primask
    str r1, [r0, #8]
    mrs r1, basepri
    str r1, [r0, #12]
    mrs r1, faultmask
    str r1, [r0, #16]
    mrs r1, control
    str r1, [r0, #20]
    mrs r1, ipsr
    str r1, [r0, #24]
    mrs r1, msp
    str r1, [r0, #28]
    mrs r1, xpsr
    str r1, [r0, #32]
    mov r1, lr
    str r1, [r0, #36]
    ldr r1, [r0, #4]
    adds r1, r1, #1
    str r1, [r0, #4]
    cpsid i
    ldr r0, =_data_load
    ldr r1, =_data_start
    ldr r2, =_data_end
1:  cmp r1, r2
    bhs 2f
    ldr r3, [r0], #4
    str r3, [r1], #4
    b 1b
2:  ldr r1, =_bss_start
    ldr r2, =_bss_end
    movs r3, #0
3:  cmp r1, r2
    bhs 4f
    str r3, [r1], #4
    b 3b
4:  bl runtime_probe
5:  wfi
    b 5b
.size reset_entry, .-reset_entry

.section .text.runtime_core_reset_aircr_write,"ax",%progbits
.global runtime_core_reset_aircr_write
.type runtime_core_reset_aircr_write,%function
.thumb_func
runtime_core_reset_aircr_write:
    ldr r1, =0xe000ed0c
.global runtime_core_reset_aircr_store
runtime_core_reset_aircr_store:
    str r0, [r1]
    bx lr
.size runtime_core_reset_aircr_write, .-runtime_core_reset_aircr_write
