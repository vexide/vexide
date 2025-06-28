/*
 * Portions of this linker script are derived from the picolibc project.
 * License: Copyright © 2019 Keith Packard (BSD-3-Clause)
 * <https://github.com/picolibc/picolibc/blob/main/picolibc.ld.in>
 */

OUTPUT_FORMAT("elf32-littlearm")

/*
 * _boot is an assembly routine defined in src/lib.rs that sets
 * up the stack pointer before jumping to _start (Rust entrypoint).
 */
ENTRY(_boot)

__program_ram_start = 0x03800000;
__program_ram_length = 42M; /* This is actually 0x04800000 (48M), but we don't want the stack colliding with the patch. */
__program_ram_end = __program_ram_start + __program_ram_length;

__patcher_ram_start = 0x07A00000;
__patcher_ram_length = 6M; /* 6mb total, 2mb for each subsection. */
__patcher_ram_end = __patcher_ram_start + __patcher_ram_length; /* End of CPU1 RWX memory block. */
__patcher_section_length = 2M;

__code_signature_length = 0x20;

__stack_length = 4M;
__heap_end = __program_ram_end - __stack_length;
__program_length = __heap_start - __program_ram_start;

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

        /* libc constructor code */
		KEEP(*(SORT_BY_NAME(.init) SORT_BY_NAME(.init.*)))

        /* Rest of the program. */
        *(.text .text.*)

        /* libc destructor code */
		KEEP(*(.fini .fini.*))

		/* Need to align the following function pointers correctly */
		. = ALIGN(4);

        /* lists of constructors and destructors */
		PROVIDE_HIDDEN(__preinit_array_start = .);
		KEEP(*(.preinit_array))
		PROVIDE_HIDDEN(__preinit_array_end = .);

		PROVIDE_HIDDEN(__init_array_start = .);
		KEEP(*(SORT_BY_INIT_PRIORITY(.init_array.*) SORT_BY_INIT_PRIORITY(.ctors.*)))
		KEEP(*(.init_array .ctors))
		PROVIDE_HIDDEN(__init_array_end = .);

		PROVIDE_HIDDEN(__fini_array_start = .);
		KEEP(*(SORT_BY_INIT_PRIORITY(.fini_array.*) SORT_BY_INIT_PRIORITY(.dtors.*)))
		KEEP(*(.fini_array .dtors))
		PROVIDE_HIDDEN(__fini_array_end = .);
    } > PROGRAM_RAM

    /*
     * Global/uninitialized/static/constant data sections.
     */
    .rodata : {
        *(.rodata .rodata1 .rodata.*)
		*(.srodata .srodata.*)

        /* Global Offset Table, used for resolving globals */
        *(.got*)
    } > PROGRAM_RAM

    /*
     * ARM Stack Unwinding Sections
     *
     * These sections are added by the compiler in some cases to facilitate stack unwinding.
     * __eh_frame_start and similar symbols are used by libunwind.
     */

    .except_ordered : {
        __extab_start = .;
		*(.gcc_except_table *.gcc_except_table.*)
        *(.ARM.extab*)
        __extab_end = .;
    } > PROGRAM_RAM

    .eh_frame_hdr : {
        /* see https://github.com/llvm/llvm-project/blob/main/libunwind/src/AddressSpace.hpp#L78 */
        PROVIDE(__eh_frame_hdr_start = .);
        KEEP(*(.eh_frame_hdr))
        PROVIDE(__eh_frame_hdr_end = .);
    } > PROGRAM_RAM

    .eh_frame : {
        PROVIDE(__eh_frame_start = .);
        KEEP(*(.eh_frame))
        PROVIDE(__eh_frame_end = .);
    } > PROGRAM_RAM

    .except_unordered : {
        PROVIDE(__exidx_start = .);
        *(.ARM.exidx*)
        PROVIDE(__exidx_end = .);
    } > PROGRAM_RAM

    /* -- Data intended to be mutable begins here. -- */

    .data : {
        *(.data .data1 .data.*)
		*(.sdata .sdata.* .sdata2.*)
    } > PROGRAM_RAM

    /*
     * Task-local data
     */
    .vexide_tdata : {
        __vexide_tdata_start = .;
        *(.vexide_tdata .vexide_tdata.*)
        __vexide_tdata_end = .;
    } > PROGRAM_RAM

    /*
     * .tdata must be at least as aligned as .tbss so memcpy-ing the entire
     * TLS block to an address with alignment `__tls_align` works as expected.
     *
     * See: https://github.com/picolibc/picolibc/commit/de46cd909e6cad136e4c97c45c74f47cb4b00426
     */

    .tdata : ALIGN(__tls_align) {
        __tls_base = .;
        __tdata_start = .;

        *(.tdata .tdata.*)

        __tdata_end = .;
    } > PROGRAM_RAM

    /* -- End of loadable sections - anything beyond this point shouldn't go in the BIN. -- */

    /* __bss_start includes .tbss because both of them need to be zeroed on startup */
    .tbss (NOLOAD) : {
        __bss_start = .;
        __tbss_start = .;

        *(.tbss .tbss.*)
        *(.tcommon)

        __tls_end = .;
        __tbss_end = .;
    } > PROGRAM_RAM

    __tls_align = MAX(ALIGNOF(.tdata), ALIGNOF(.tbss));
    __arm32_tls_tcb_offset = MAX(8, __tls_align);

    .bss (NOLOAD) : {
		*(.sbss*)
        *(.bss .bss.*)

		/* Align the heap */
		. = ALIGN(8);
		__bss_end = .;
    } > PROGRAM_RAM

    /*
     * Active memory sections for the stack/heap.
     *
     * Because these are (NOLOAD), they will not influence the final size of the binary.
     */
    .heap (NOLOAD) : {
        __heap_start = .;
        . = __heap_end;
    } > PROGRAM_RAM

    .stack (NOLOAD) : ALIGN(8) {
        __stack_bottom = .;
        . += __stack_length;
        __stack_top = .;
    } > PROGRAM_RAM

    /* Patcher Memory */
    .patcher_patch (NOLOAD) : {
        __patcher_patch_start = .;
        . += __patcher_section_length;
        __patcher_patch_end = .;
    } > PATCHER_RAM

    .patcher_base_copy (NOLOAD) : {
        __patcher_base_start = .;
        . += __patcher_section_length;
        __patcher_base_end = .;
    } > PATCHER_RAM

    .patcher_new (NOLOAD) : {
        __patcher_new_start = .;
        . += __patcher_section_length;
        __patcher_new_end = .;
    } > PATCHER_RAM

    /*
     * `.ARM.attributes` contains arch metadata for compatibility purposes, but we
     * only target one hardware configuration, meaning it'd just take up space.
     */
    /DISCARD/ : {
        *(.ARM.attributes*)
    }

    /*
     * These sections are useful for debugging but shouldn't end up in the final
     * binary, so they are only stored in the ELF and not loaded into memory.
     */

    /* ELF metadata sections */
    .symtab   0 : { *(.symtab) }
    .strtab   0 : { *(.strtab) }
    .shstrtab 0 : { *(.shstrtab) }
    .debug    0 : { *(.debug*) }
    .comment  0 : { *(.comment) }
}
