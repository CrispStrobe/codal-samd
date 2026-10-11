/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 CrispStrobe contributors
 * Minimal owned diagnostic startup; no CODAL/vendor/CMSIS startup included.
 */
.syntax unified
.thumb
.section .vectors,"a",%progbits
.word _stack_top
.word reset_entry
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
4:  bl transfer_state_probe
5:  wfi
    b 5b
.size reset_entry, .-reset_entry
