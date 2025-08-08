/* Applying this linker script will reserve space at the end of user memory for differential uploading. */

__patcher_section_length = 2M;
/* 6MiB total, 2MiB for each subsection. (start = 0x07a00000) */
__linked_file_start = __linked_file_end - (__patcher_section_length * 3);

OVERWRITE_SECTIONS {
    /* Ensure .overwriter is placed after .boot but before .text */

    .text : {
        /* _boot routine (entry point from VEXos, must be at 0x03800020) */
        *(.boot)

        /* Stage 2 patcher. */
        *(.overwriter)

        /* The rest of the program. */
        *(.text .text.*)
    } > USER_RAM
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
