/* This is a safety net against attempting to link big-endian */
OUTPUT_FORMAT("elf32-littlearm", "elf32-littlearm", "elf32-littlearm")

/* cold memory contains the OS, C library, etc */
__cold_start = 0x03800000;
__cold_length = 0x04800000; /* 72 MiB */

__heap_start = 0x04a00000;
__heap_length = 0x02c00a60; /* 44 MiB */

__user_stack_length = 278528; /* 2.125 MiB */
