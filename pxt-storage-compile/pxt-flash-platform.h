// SPDX-License-Identifier: MIT
#pragma once
#include "platform_includes.h"
#include "codal_target_hal.h"

// Authored narrow register views from DS60001507F, chapters 7, 11 and 25.
// No ASF/CMSIS/vendor source header is selected or copied. These declarations
// are not an emulator, flash backing implementation, or hardware qualification.
#if __BYTE_ORDER__ != __ORDER_LITTLE_ENDIAN__
#error "Register views require the reviewed little-endian ARM ABI"
#endif
namespace bw_pxt_flash {
union ControlA {
    uint16_t reg;
    struct {
        uint16_t reserved0:4;
        uint16_t WMODE:2;
        uint16_t reserved1:8;
        uint16_t CACHEDIS0:1;
        uint16_t CACHEDIS1:1;
    } bit;
};
union Parameter {
    uint32_t reg;
    struct { uint32_t NVMP:16; uint32_t PSZ:3; uint32_t reserved:13; } bit;
};
union Status {
    uint16_t reg;
    struct { uint16_t READY:1; uint16_t reserved:15; } bit;
};
struct Command { uint16_t reg; };
struct Address { uint32_t reg; };
struct NvmRegisters {
    ControlA CTRLA;                    // 0x00, 16-bit
    uint16_t reserved0;
    Command CTRLB;                     // 0x04, 16-bit
    uint16_t reserved1;
    Parameter PARAM;                   // 0x08, 32-bit
    uint8_t reserved2[6];
    Status STATUS;                     // 0x12, 16-bit
    Address ADDR;                      // 0x14, 32-bit
};
union CacheControl {
    uint32_t reg;
    struct { uint32_t CEN:1; uint32_t reserved:31; } bit;
};
union CacheStatus {
    uint32_t reg;
    struct { uint32_t CSTS:1; uint32_t reserved:31; } bit;
};
union CacheMaintenance {
    uint32_t reg;
    struct { uint32_t INVALL:1; uint32_t reserved:31; } bit;
};
struct CacheRegisters {
    uint32_t reserved0[2];
    CacheControl CTRL;                 // 0x08, 32-bit
    CacheStatus SR;                    // 0x0c, 32-bit
    uint32_t reserved1[4];
    CacheMaintenance MAINT0;           // 0x20, 32-bit
};
#define BW_FIELD_OFFSET(type, field, offset) \
    static_assert(__builtin_offsetof(type, field) == offset, "register offset")
BW_FIELD_OFFSET(NvmRegisters, CTRLA, 0x00);
BW_FIELD_OFFSET(NvmRegisters, CTRLB, 0x04);
BW_FIELD_OFFSET(NvmRegisters, PARAM, 0x08);
BW_FIELD_OFFSET(NvmRegisters, STATUS, 0x12);
BW_FIELD_OFFSET(NvmRegisters, ADDR, 0x14);
BW_FIELD_OFFSET(CacheRegisters, CTRL, 0x08);
BW_FIELD_OFFSET(CacheRegisters, SR, 0x0c);
BW_FIELD_OFFSET(CacheRegisters, MAINT0, 0x20);
static_assert(sizeof(ControlA) == 2 && sizeof(Command) == 2 && sizeof(Status) == 2,
              "16-bit NVM register widths");
static_assert(sizeof(Parameter) == 4 && sizeof(Address) == 4 &&
              sizeof(CacheControl) == 4 && sizeof(CacheStatus) == 4 &&
              sizeof(CacheMaintenance) == 4, "32-bit register widths");
#undef BW_FIELD_OFFSET
}

#define NVMCTRL (reinterpret_cast<volatile bw_pxt_flash::NvmRegisters *>(0x41004000u))
#define CMCC (reinterpret_cast<volatile bw_pxt_flash::CacheRegisters *>(0x41006000u))
#define NVMCTRL_BLOCK_SIZE 8192u
#define NVMCTRL_CTRLA_WMODE_MAN_Val 0u
#define NVMCTRL_CTRLB_CMDEX_KEY 0xa500u
#define NVMCTRL_CTRLB_CMD_EB 0x01u
#define NVMCTRL_CTRLB_CMD_WQW 0x04u
#define NVMCTRL_CTRLB_CMD_PBC 0x15u
#define NOLOG(...) ((void)0)
static inline void __DMB() { __asm__ volatile("dmb 0xf" ::: "memory"); }
