INCLUDE ../link/vexide.ld

PROVIDE(__dlmalloc_heap_length = 128K);

SECTIONS {
    .talc_heap (NOLOAD) : {
        __talc_heap_start = .;
        . = __stack_bottom - __dlmalloc_heap_length;
        __talc_heap_end = .;
    } > USER_RAM
} INSERT BEFORE .heap;
