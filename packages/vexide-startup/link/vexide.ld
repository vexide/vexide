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

        *(.vector_table)

        /* The rest of the program. */
        *(.text .text.*)
    } > USER_RAM

    /DISCARD/ : {
        /* Strip out libstd's _boot routine (replaced with .vexide_boot section). */
        *(.boot)
    }
}

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
