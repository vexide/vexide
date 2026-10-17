/* see the comment in build.rs to understand these shenanigans */
INCLUDE ../link/vexide.ld

/*
 * When talc is used as the allocator, we need a separate section from .heap,
 * since there are rare edge-cases where libstd's System allocator (using dlmalloc)
 * will get linked in despite us setting the #[global_allocator].
 *
 * See <https://github.com/vexide/vexide/issues/469> for an example of this crashing and burning.
 *
 * So, what we're going to do here is shrink down the .heap section to something small (128K) as a
 * contingency for the rare cases that the System allocator is used, then talc gets the rest of the
 * memory.
 */

INCLUDE ../link/vexide.ld

PROVIDE(__dlmalloc_heap_length = 128K);

SECTIONS {
    .talc_heap (NOLOAD) : ALIGN(8) {
        __talc_heap_start = .;
        . = __stack_bottom - __dlmalloc_heap_length;
        __talc_heap_end = .;
    } > USER_RAM
} INSERT BEFORE .heap;
