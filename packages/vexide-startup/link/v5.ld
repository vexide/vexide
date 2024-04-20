OUTPUT_FORMAT("elf32-littlearm")

ENTRY(_entry)

MEMORY {
    COLD : ORIGIN = 0x03800000, LENGTH = 0x04800000
}

__heap_end = 0x7800000;
__stack_length = 0x204000;

SECTIONS {
    .text : {
        KEEP(*(.cold_magic))
        /* Size of cold header */
        . += 0x20;
        *(.boot)
        *(.text .text.*)
    } > COLD
    .rodata : {
		__rodata_start = .;
		*(.rodata1 .rodata1.*)
		__rodata_end = .;
	} > COLD
    .data : {
        *(.data .data.*)
        *(.data1 .data1.*)
    } > COLD
    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    } > COLD
    .heap (NOLOAD) : ALIGN(4) {
        __heap_start = .;
        . = __heap_end;
    } > COLD
    .stack (NOLOAD) : ALIGN(8) {
        __stack_end = .;
        . += __stack_length;
        __stack_start = .;
    } > COLD
}
