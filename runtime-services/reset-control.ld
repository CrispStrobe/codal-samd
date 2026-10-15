/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 CrispStrobe contributors
 * Simulation core-reset fixture. NOLOAD witness survives only modeled reset.
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
  .noinit (NOLOAD) : {
    . = ALIGN(8); _noinit_start = .;
    KEEP(*(.noinit))
    . = ALIGN(8); _noinit_end = .;
  } > RAM
  _heap_start = _noinit_end;
  /DISCARD/ : { *(.comment) *(.ARM.exidx*) *(.ARM.extab*) }
}
ASSERT(_noinit_end - _noinit_start == 48, "closed reset witness")
ASSERT(_heap_start < _stack_top - 8192 - 32768, "service heap and stack reserve")
