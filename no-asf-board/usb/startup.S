/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 CrispStrobe contributors
 * Authored USB leaf startup with relocated vectors and actual constructors.
 */
.syntax unified
.cpu cortex-m4
.thumb
.section .boot_vectors,"a",%progbits
.word _stack_top, reset_entry
.section .vectors,"a",%progbits
.balign 1024
.global runtime_vectors
runtime_vectors:
.word _stack_top, reset_entry
.rept 13
.word runtime_fault
.endr
.word runtime_systick
.rept 80
.word runtime_fault
.endr
.word USB_0_Handler, USB_1_Handler, USB_2_Handler, USB_3_Handler
.rept 156
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
    ldr r0, =0xe000ed08
    ldr r1, =runtime_vectors
    str r1, [r0]
    // The selected softfp ABI permits VFP operations; enable CP10/CP11.
    ldr r0, =0xe000ed88
    ldr r1, [r0]
    ldr r2, =0x00f00000
    orrs r1, r2
    str r1, [r0]
    dsb
    isb
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
.global usb_static_init_entry
usb_static_init_entry:
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
