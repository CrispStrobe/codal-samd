// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "samd/pins.h"

const mcu_pin_obj_t *samd_peripherals_get_pin(uint32_t number)
{
    for (const mcu_pin_obj_t *p = samd_pins; p->number != PIN_NONE; p++)
        if (p->number == number)
            return p;
    return 0;
}
