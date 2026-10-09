/* SPDX-License-Identifier: MIT
 * Copyright (c) 2026 CrispStrobe contributors
 * Generic Cortex-M4 diagnostic layout, not production PyBadge memory mapping.
 */
ENTRY(reset_entry)
MEMORY {
  FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 256K
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
    . = ALIGN(4); _bss_end = .;
  } > RAM
  /DISCARD/ : { *(.comment) *(.ARM.exidx*) *(.ARM.extab*) }
}
ASSERT(_bss_end <= _stack_top - 4096, "diagnostic stack reserve")
