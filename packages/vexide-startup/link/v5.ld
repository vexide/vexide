OUTPUT_FORMAT("elf32-littlearm")

ENTRY(_boot)

__program_ram_start = 0x03800000;
__program_ram_length = 0x04200000; /* This is actually 0x04800000, but we don't want the stack colliding with the patch. */
__program_ram_end = __program_ram_start + __program_ram_length;

__patcher_ram_start = 0x07A00000;
__patcher_ram_length = 0x600000; /* 6mb, 2mb for each segment */
__patcher_ram_end = __patcher_ram_start + __patcher_ram_length; /* End of CPU1 RWX memory block */

__code_signature_length = 0x20;

__stack_length = 0x400000;
__heap_end = __program_ram_end - __stack_length;

/* see https://github.com/llvm/llvm-project/blob/main/libunwind/src/AddressSpace.hpp#L78 */
__eh_frame_hdr_start = SIZEOF(.eh_frame_hdr) > 0 ? ADDR(.eh_frame_hdr) : 0;
__eh_frame_hdr_end = SIZEOF(.eh_frame_hdr) > 0 ? . : 0;

MEMORY {
    PROGRAM_RAM : ORIGIN = __program_ram_start, LENGTH = __program_ram_length
    PATCHER_RAM : ORIGIN = __patcher_ram_start, LENGTH = __patcher_ram_length
}

SECTIONS {
    /*
     * VEXos expects program binaries to have a 32-byte header called a "code signature",
     * at their start, which tells the OS that we are a valid program and configures some
     * miscellaneous startup behavior.
     */
    .code_signature : {
        KEEP(*(.code_signature))
        . = __program_ram_start + __code_signature_length;
    } > PROGRAM_RAM

    .text : {
        *(.boot)
        *(.overwriter)
        *(.text .text.*)
    } > PROGRAM_RAM

    /* Global/uninitialized/static/constant data sections. */
    .rodata : {
        *(.rodata .rodata.*)
    } > PROGRAM_RAM

    .data : {
        *(.data .data.*)
    } > PROGRAM_RAM

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        __bss_end = .;
    } > PROGRAM_RAM

    /*
     * These sections are added by the compiler in some cases to facilitate stack unwinding.
     * __eh_frame_start and similar symbols are used by libunwind.
     */
    .eh_frame_hdr : {
        KEEP(*(.eh_frame_hdr))
    } > PROGRAM_RAM

    .eh_frame : {
        __eh_frame_start = .;
       KEEP(*(.eh_frame))
        __eh_frame_end = .;
    } > PROGRAM_RAM

    .ARM.exidx : {
        __exidx_start = .;
        *(.ARM.exidx*)
        __exidx_end = .;
    } > PROGRAM_RAM

    .ARM.extab : {
        __extab_start = .;
        *(.ARM.extab*)
        __extab_end = .;
    } > PROGRAM_RAM

    /* Active memory regions for the stack/heap. */
    .heap (NOLOAD) : ALIGN(4) {
        __heap_start = .;
        . = __heap_end;
    } > PROGRAM_RAM

    .stack (NOLOAD) : ALIGN(8) {
        __stack_bottom = .;
        . += __stack_length;
        __stack_top = .;
    } > PROGRAM_RAM

    /*
     * `.ARM.attributes` contains arch metadata for compatibility purposes, but we
     * only target one hardware configuration, meaning it'd just take up space.
     */
    /DISCARD/ : {
        *(.ARM.attributes*)
    }
}
