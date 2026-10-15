/*
 * Linker script for Xbox 360 PowerPC XEX payload
 */
OUTPUT_FORMAT("elf32-powerpc", "elf32-powerpc", "elf32-powerpc")
OUTPUT_ARCH(powerpc:common)
ENTRY(_start)

SECTIONS
{
    . = 0x82001000;

    .text : {
        *(.text.entry)
        *(.text .text.*)
        . = ALIGN(16);
        *(.text.thunks)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    . = ALIGN(4096);

    .data : {
        __import_table_start = .;
        *(.data.imports)
        __import_table_end = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    .bss : {
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        *(COMMON)
    }

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
    }
}
