OUTPUT_FORMAT("elf32-littlearm")

/*
 * _boot is an assembly routine defined in src/lib.rs that sets
 * up the stack pointer before jumping to _start (Rust entrypoint).
 */
ENTRY(_boot)

__program_ram_start = 0x03800000;
__program_ram_length = 0x04200000; /* This is actually 0x04800000, but we don't want the stack colliding with the patch. */
__program_ram_end = __program_ram_start + __program_ram_length;

__patcher_ram_start = 0x07A00000;
__patcher_ram_length = 0x600000; /* 6mb total, 2mb for each subsection. */
__patcher_ram_end = __patcher_ram_start + __patcher_ram_length; /* End of CPU1 RWX memory block. */

__code_signature_length = 0x20;

__stack_length = 0x400000;
__heap_end = __program_ram_end - __stack_length;
__program_length = __heap_start - __program_ram_start;

/* see https://github.com/llvm/llvm-project/blob/main/libunwind/src/AddressSpace.hpp#L78 */
__eh_frame_hdr_start = SIZEOF(.eh_frame_hdr) > 0 ? ADDR(.eh_frame_hdr) : 0;
__eh_frame_hdr_end = SIZEOF(.eh_frame_hdr) > 0 ? . : 0;

MEMORY {
    /*
     * Memory available for (initial) use by the user program.
     *
     * This is technically all the way from 0x03800000-0x8000000, but we reserve
     * the last 6mb for the patcher, which is later reclaimed as heap space.
     */
    PROGRAM_RAM (RWX) : ORIGIN = __program_ram_start, LENGTH = __program_ram_length

    /*
     * Memory reserved for the patcher.
     *
     * We reserve last 6mb of the CPU1 RWX region for storing data relating to patch
     * uploading. This region is further split into three 2mb "subsections":
     *
     * - The first 2mb are where the actual patch itself is loaded to.
     * - The next 2mb is a buffer where the running user program is copied to, which
     *   will serve as a reference for building the patched binary. We can't use active
     *   memory to do this, since statics and unwind tables will be written to at runtime,
     *   messing with the patch.
     * - The final 2mb is where the new patched binary will be constructed and eventually
     *   memcpy'd back to __program_ram_start.
     *
     * Following the patch process (or if we don't need to patch), this entire region is
     * claimed as heap space.
     */
    PATCHER_RAM (RWX) : ORIGIN = __patcher_ram_start, LENGTH = __patcher_ram_length
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

    /*
     * Executable program instructions.
     */
    .text : {
        /* _boot routine (always runs first, must be at 0x03800020) */
        *(.boot)
        /* Stage 2 patcher. */
        *(.overwriter)
        /* Rest of the program. */
        *(.text .text.*)
    } > PROGRAM_RAM

    /*
     * Global/uninitialized/static/constant data sections.
     */
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
     * ARM Stack Unwinding Sections
     *
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

    /* -- End of loadable sections - anything beyond this point shouldn't go in the BIN. -- */

    /*
     * Active memory sections for the stack/heap.
     *
     * Because these are (NOLOAD), they will not influence the final size of the binary.
     */
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
