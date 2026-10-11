/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 CrispStrobe contributors
 * Authored Cortex-M4 diagnostic vectors; not a SAMD production startup.
 */
.syntax unified
.thumb
.section .vectors,"a",%progbits
.balign 1024
.global authored_vectors
authored_vectors:
.word _stack_top, reset_entry
.word unexpected_exception /* NMI */
.word unexpected_exception /* HardFault */
.word unexpected_exception /* MemManage */
.word unexpected_exception /* BusFault */
.word unexpected_exception /* UsageFault */
.word 0, 0, 0, 0 /* reserved ARMv7-M slots */
.word unexpected_exception /* SVCall */
.word unexpected_exception /* DebugMonitor */
.word 0 /* reserved */
.word unexpected_exception /* PendSV */
.word unexpected_exception /* SysTick */
.word authored_irq0
.rept 239
.word unexpected_exception
.endr
.global authored_vectors_end
authored_vectors_end:

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
4:  bl irq_mask_probe
5:  wfi
    b 5b
.size reset_entry, .-reset_entry
