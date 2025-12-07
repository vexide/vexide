/*
 * This file contains vexide-specific memory layout options.
 *
 * This file is designed to augment the builtin linkerscript for
 * the `armv7a-vex-v5` target in rustc, which may be found here:
 * <https://github.com/rust-lang/rust/blob/master/compiler/rustc_target/src/spec/targets/armv7a_vex_v5_linker_script.ld>
 */

/* 6MiB total, 2MiB for each subsection. (start = 0x07a00000) */
__patcher_section_length = 2M;
__linked_file_length = __patcher_section_length * 3;

OVERWRITE_SECTIONS {
    /* Allow users to specify their code signature from Rust source code */
    .code_signature : {
        KEEP(*(.code_signature))
        FILL(0)
        . = __user_ram_start + 0x20;
    } > USER_RAM

    /* Ensure .overwriter is placed after .boot but before .text */
    .text : {
        /* _boot routine (entry point from VEXos, must be at 0x03800020) */
        *(.vexide_boot)

        /* Patch overwriter routine. */
        *(.overwriter)

        /* The rest of the program. */
        *(.text .text.*)

        /* Init/finalizer arrays: lists of pointers to init functions. */

		/* Pre-init constructors. (run first) */
        /*
         * These array symbols need to be aligned to exactly the start of the array
         * to avoid manually skipping padding at runtime.
         */
		. = ALIGN(8);

		PROVIDE_HIDDEN( __preinit_array_start = . );
		PROVIDE_HIDDEN( __bothinit_array_start = . );

		KEEP( *(.preinit_array) )

		PROVIDE_HIDDEN( __preinit_array_end = . );

		/* Normal constructors. */
		PROVIDE_HIDDEN( __init_array_start = . );

		KEEP( *(SORT_BY_INIT_PRIORITY(.init_array.*) SORT_BY_INIT_PRIORITY(.ctors.*)) )
		KEEP( *(.init_array .ctors) )

		PROVIDE_HIDDEN( __init_array_end = . );
		PROVIDE_HIDDEN( __bothinit_array_end = . );

		/* Destructors. */
		PROVIDE_HIDDEN( __fini_array_start = . );

		KEEP( *(SORT_BY_INIT_PRIORITY(.fini_array.*) SORT_BY_INIT_PRIORITY(.dtors.*)) )
		KEEP( *(.fini_array .dtors) )
		KEEP( *(.fini_array*) )

		PROVIDE_HIDDEN( __fini_array_end = . );
    } > USER_RAM

    /DISCARD/ : {
        /* Strip out libstd's _boot routine (replaced with .vexide_boot section). */
        *(.boot)
    }
}

SECTIONS {
    /*
     * Global offset table, which holds addresses of symbols referenced by `.text`.
     *
     * This allows for a position-independent executable (which we aren't making),
     * but we want to support external code that uses this strategy anyways.
     * See: <https://maskray.me/blog/2021-08-29-all-about-global-offset-table>
     */
	.got : {
		*(.got.plt .got)
	} > USER_RAM
} INSERT BEFORE .data;

SECTIONS {
    /* Patcher Memory */
    .patcher_patch (NOLOAD) : {
        __patcher_patch_start = .;
        . += __patcher_section_length;
        __patcher_patch_end = .;
    } > USER_RAM

    .patcher_base_copy (NOLOAD) : {
        __patcher_base_start = .;
        . += __patcher_section_length;
        __patcher_base_end = .;
    } > USER_RAM

    .patcher_new (NOLOAD) : {
        __patcher_new_start = .;
        . += __patcher_section_length;
        __patcher_new_end = .;
    } > USER_RAM
} INSERT AFTER .stack;

/*
 * Compiler-aware thread-local storage support.
 * This is used to support the thread_local keyword in C, for e.g. errno.
 * .tdata and .tbss are a template for a new TLS block, but since there's
 * only one thread we just use them the same way as data. The
 * _set_tls(void* tls) function is used to set the pointer to the current
 * TLS block.
 */
SECTIONS {
    /*
     * .tdata must be at least as aligned as .tbss so that memcpy-ing the entire
     * TLS block to an address with alignment `__tls_align` works as expected.
     *
     * See: https://github.com/picolibc/picolibc/commit/de46cd909e6cad136e4c97c45c74f47cb4b00426
     */

    .tdata : ALIGN(__tls_align) {
        __tls_base = .;
        __tdata_start = .;

        *(.tdata .tdata.* .gnu.linkonce.td.*)

        __tdata_end = .;
    } > USER_RAM

    /*
     * `.tbss` must come directly before `.bss`, not any earlier.
     * This makes the NOLOAD attribute take effect since .bss is also NOLOAD.
     */

    .tbss (NOLOAD) : {
        __tbss_start = .;

        *(.tbss .tbss.*)
        *(.tcommon)

        __tls_end = .;
        __tbss_end = .;
    } > USER_RAM

    __tls_align = MAX(ALIGNOF(.tdata), ALIGNOF(.tbss));

    /*
     * Offset to account for thread control block and also align padding.
     * The thread control block is 8 bytes. Used internally in picolibc.
     */
    __arm32_tls_tcb_offset = MAX(8, __tls_align);
} INSERT AFTER .data;
