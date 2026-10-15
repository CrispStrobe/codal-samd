// SPDX-License-Identifier: MIT
#include "pxtbase-storage.h"

// Compile the actual public class definitions, never duplicated layouts.
static_assert(PXT_UTF8 == 1, "selected real UTF8 string representation");
static_assert(sizeof(void *) == 4 && sizeof(pxt::TValue) == 4, "32-bit target ABI");
static_assert(sizeof(pxt::RefObject) == 4, "actual object header");
static_assert(sizeof(pxt::Segment) == 8, "actual 16-bit length/capacity segment");
static_assert(sizeof(pxt::RefCollection) == 12, "actual collection size");
static_assert(sizeof(pxt::BoxedBuffer) == 8, "actual buffer header");
static_assert(__is_same(pxt::String, pxt::BoxedString *), "actual string alias");
static_assert(__is_same(pxt::Buffer, pxt::BoxedBuffer *), "actual buffer alias");
static_assert(__is_base_of(pxt::RefObject, pxt::BoxedString), "actual string inheritance");
static_assert(__is_base_of(pxt::RefObject, pxt::BoxedBuffer), "actual buffer inheritance");
static_assert(__is_base_of(pxt::RefObject, pxt::RefCollection), "actual collection inheritance");
