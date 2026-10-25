// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Independently authored PORT access with the GPIO call surface used by the
// selected CODAL drivers. Register facts come from the generated ATDF header.
#pragma once
#include <stdbool.h>
#include <stdint.h>
#include "sam.h"

#ifdef __cplusplus
extern "C" {
#endif

enum gpio_direction { GPIO_DIRECTION_OFF, GPIO_DIRECTION_IN, GPIO_DIRECTION_OUT };
enum gpio_pull_mode { GPIO_PULL_OFF, GPIO_PULL_UP, GPIO_PULL_DOWN };

#define GPIO_PORT(pin) ((pin) >> 5)
#define GPIO_PIN(pin) ((pin) & 0x1f)
#define GPIO_PIN_FUNCTION_OFF 0xffffffff
#define GPIO(port, pin) ((((port) & 0x7u) << 5) + ((pin) & 0x1fu))
#ifndef PINMUX
#define PINMUX(pin, mux) ((((uint32_t)(pin)) << 16) | (uint32_t)(mux))
#endif

static inline PortGroup *gpio_group(uint32_t pin) { return &PORT->Group[GPIO_PORT(pin)]; }

// A function value is a peripheral multiplexer index in its low half-word, or
// OFF. The pin is always the first argument, matching existing callers.
static inline void gpio_set_pin_function(uint32_t pin, uint32_t function)
{
    PortGroup *group = gpio_group(pin);
    uint8_t index = GPIO_PIN(pin);
    if (function == GPIO_PIN_FUNCTION_OFF) {
        group->PINCFG[index].reg &= (uint8_t)~PORT_PINCFG_PMUXEN;
        return;
    }
    uint8_t mux = (uint8_t)(function & 0xf);
    uint8_t value = group->PMUX[index >> 1].reg;
    if (index & 1)
        value = (uint8_t)((value & ~PORT_PMUX_PMUXO_Msk) | PORT_PMUX_PMUXO(mux));
    else
        value = (uint8_t)((value & ~PORT_PMUX_PMUXE_Msk) | PORT_PMUX_PMUXE(mux));
    group->PMUX[index >> 1].reg = value;
    group->PINCFG[index].reg |= PORT_PINCFG_PMUXEN;
}

static inline void gpio_set_pin_direction(uint32_t pin, enum gpio_direction direction)
{
    PortGroup *group = gpio_group(pin);
    uint32_t mask = 1u << GPIO_PIN(pin);
    uint8_t index = GPIO_PIN(pin);
    switch (direction) {
    case GPIO_DIRECTION_OUT:
        group->DIRSET.reg = mask;
        group->PINCFG[index].reg &= (uint8_t)~PORT_PINCFG_INEN;
        break;
    case GPIO_DIRECTION_IN:
        group->DIRCLR.reg = mask;
        group->PINCFG[index].reg |= PORT_PINCFG_INEN;
        break;
    default:
        group->DIRCLR.reg = mask;
        group->PINCFG[index].reg &= (uint8_t)~PORT_PINCFG_INEN;
        break;
    }
}

static inline void gpio_set_pin_level(uint32_t pin, bool level)
{
    if (level)
        gpio_group(pin)->OUTSET.reg = 1u << GPIO_PIN(pin);
    else
        gpio_group(pin)->OUTCLR.reg = 1u << GPIO_PIN(pin);
}

static inline void gpio_toggle_pin_level(uint32_t pin)
{
    gpio_group(pin)->OUTTGL.reg = 1u << GPIO_PIN(pin);
}

// Pull resistors act only on inputs; OUT selects pull-up (1) or pull-down (0).
static inline void gpio_set_pin_pull_mode(uint32_t pin, enum gpio_pull_mode mode)
{
    PortGroup *group = gpio_group(pin);
    uint32_t mask = 1u << GPIO_PIN(pin);
    uint8_t index = GPIO_PIN(pin);
    if (mode == GPIO_PULL_OFF) {
        group->PINCFG[index].reg &= (uint8_t)~PORT_PINCFG_PULLEN;
        return;
    }
    group->DIRCLR.reg = mask;
    if (mode == GPIO_PULL_UP)
        group->OUTSET.reg = mask;
    else
        group->OUTCLR.reg = mask;
    group->PINCFG[index].reg |= PORT_PINCFG_PULLEN;
}

// An output reads back its driven level; an input reads its sampled level.
static inline bool gpio_get_pin_level(uint32_t pin)
{
    PortGroup *group = gpio_group(pin);
    uint32_t mask = 1u << GPIO_PIN(pin);
    uint32_t dir = group->DIR.reg;
    return ((group->IN.reg & ~dir) | (group->OUT.reg & dir)) & mask;
}

#ifdef __cplusplus
}
#endif
