OUTPUT_FORMAT("elf32-littlearm")

ENTRY(_boot)

__user_ram_start = 0x03800000;
__user_ram_length = 0x04800000;
__user_ram_end = __user_ram_start + __user_ram_length;

__code_signature_length = 0x20;

MEMORY {
    USER_RAM : ORIGIN = __user_ram_start, LENGTH = __user_ram_length
}

__stack_length = 0x400000;
__heap_end = __user_ram_end - __stack_length;

SECTIONS {
    .code_signature : {
        KEEP(*(.code_signature))
        . = __user_ram_start + __code_signature_length;
    } > USER_RAM

    .text : {
        *(.boot)
        *(.text .text.*)
    } > USER_RAM

    .rodata : {
        *(.rodata .rodata.*)
    } > USER_RAM

    .data : {
        *(.data .data.*)
    } > USER_RAM

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    } > USER_RAM

    /* The unwind tables enabled by "default-uwtables" in the target file live here. */
    /* __eh_frame_start and similar symbols are used by libunwind. */
    .eh_frame : {
        __eh_frame_start = .;
       *(.eh_frame)
        __eh_frame_end = .;
    } > USER_RAM

    .eh_framehdr : {
       __eh_frame_hdr_start = .;
       *(.eh_framehdr)
       __eh_frame_hdr_end = .;
    } > USER_RAM

    .ARM.extab : {
        __extab_start = .;
		*(.ARM.extab*)
        __extab_end = .;
	} > USER_RAM

    .ARM.exidx : {
        __exidx_start = .;
        *(.ARM.exidx*)
        *(.gnu.linkonce.armexidix.*.*)
        __exidx_end = .;
    } > USER_RAM

    .heap (NOLOAD) : ALIGN(4) {
        __heap_start = .;
        . = __heap_end;
    } > USER_RAM

    .stack (NOLOAD) : ALIGN(8) {
        __stack_bottom = .;
        . += __stack_length;
        __stack_top = .;
    } > USER_RAM
}
