/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 CrispStrobe contributors
 * Explicit relocated USB leaf; authored reset forwarding, no vendor bootloader.
 */
ENTRY(reset_entry)
MEMORY {
  BOOT (rx) : ORIGIN = 0, LENGTH = 16K
  FLASH (rx) : ORIGIN = 0x4000, LENGTH = 200K
  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 192K
}
_stack_top = ORIGIN(RAM) + LENGTH(RAM);
SECTIONS {
  .boot_vectors : { KEEP(*(.boot_vectors)) } > BOOT
  .vectors : { KEEP(*(.vectors)) } > FLASH
  .text : { *(.text*) *(.rodata*) } > FLASH
  .init_array : {
    . = ALIGN(4); _init_array_start = .;
    KEEP(*(SORT_BY_INIT_PRIORITY(.init_array.*))) KEEP(*(.init_array))
    _init_array_end = .;
  } > FLASH
  .data : {
    . = ALIGN(4); _data_start = .;
    *(.data*)
    . = ALIGN(4); _data_end = .;
  } > RAM AT > FLASH
  _data_load = LOADADDR(.data);
  .bss (NOLOAD) : {
    . = ALIGN(4); _bss_start = .;
    *(.bss*) *(COMMON)
    . = ALIGN(8); _bss_end = .;
  } > RAM
  _heap_start = _bss_end;
  /DISCARD/ : { *(.comment) *(.ARM.exidx*) *(.ARM.extab*) }
}
ASSERT(_heap_start < _stack_top - 10240 - 32768, "service heap and stack reserve")

ASSERT(SIZEOF(.vectors) == 1024, "full relocated vector table")
ASSERT(ADDR(.vectors) == 0x4000, "generated application origin")
ASSERT(SIZEOF(.boot_vectors) == 8, "authored two-word reset forwarding only")
