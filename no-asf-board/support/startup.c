// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Reset entry for the no-ASF board. The vector table is generated from the
// ATDF (samd51j19a_vectors.c); section symbols come from the board linker script.
#include "sam.h"

extern uint32_t _sfixed, _etext, _srelocate, _erelocate, _szero, _ezero;
int main(void);
void cpu_init(void);
void __libc_init_array(void);

void Reset_Handler(void)
{
    const uint32_t *source = &_etext;
    for (uint32_t *dest = &_srelocate; dest < &_erelocate;)
        *dest++ = *source++;
    for (uint32_t *dest = &_szero; dest < &_ezero;)
        *dest++ = 0;
    SCB->VTOR = (uint32_t)&_sfixed & SCB_VTOR_TBLOFF_Msk;
    // CP10/CP11 full access before any floating-point instruction.
    SCB->CPACR |= 0xfu << 20;
    __DSB();
    __ISB();
    cpu_init();
    __libc_init_array();
    main();
    for (;;) {
    }
}
