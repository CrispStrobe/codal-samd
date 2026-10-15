/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 CrispStrobe contributors
 * Explicit GC/service caller image, not resident UF2/CF2/native Arcade.
 */
ENTRY(reset_entry)
MEMORY {
  FLASH (rx) : ORIGIN = 0, LENGTH = 512K
  RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 192K
}
_stack_top = ORIGIN(RAM) + LENGTH(RAM);
SECTIONS {
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
ASSERT(_heap_start < _stack_top - 8192 - 32768, "service heap and stack reserve")
