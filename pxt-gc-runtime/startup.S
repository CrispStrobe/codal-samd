/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 CrispStrobe contributors
 * Separately selected modeled PXT GC startup with actual static constructors.
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
4:
.global gc_static_init_entry
gc_static_init_entry:
    ldr r4, =_init_array_start
    ldr r5, =_init_array_end
6:  cmp r4, r5
    bhs 7f
    ldr r3, [r4], #4
    blx r3
    b 6b
7:  bl runtime_probe
5:  wfi
    b 5b
.size reset_entry, .-reset_entry
