// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#include "SAMDNVM.h"

template <typename A, typename B> struct Same { static const bool value = false; };
template <typename A> struct Same<A, A> { static const bool value = true; };
using Controller = codal::SAMDNVM;
static_assert(__is_base_of(codal::NVMController, Controller), "actual NVM base");
static_assert(!__is_abstract(Controller), "all actual virtual declarations present");
static_assert(Same<decltype(&Controller::read),
              int (Controller::*)(uint32_t *, uint32_t, uint32_t)>::value, "read ABI");
static_assert(Same<decltype(&Controller::write),
              int (Controller::*)(uint32_t, uint32_t *, uint32_t)>::value, "write ABI");
static_assert(Same<decltype(&Controller::erase),
              int (Controller::*)(uint32_t)>::value, "erase ABI");
static_assert(Same<decltype(&Controller::getFlashEnd),
              uint32_t (Controller::*)()>::value, "address ABI");
// No controller instance, fake peripheral implementation or link-success stub.
