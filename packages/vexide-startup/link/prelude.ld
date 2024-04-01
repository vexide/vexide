/* This is a safety net against attempting to link big-endian */
OUTPUT_FORMAT("elf32-littlearm", "elf32-littlearm", "elf32-littlearm")

/* cold memory contains the OS, C library, etc */
__cold_start = 0x03800000;
__cold_length = 0x04800000; /* 72 MiB */

__heap_start = 0x04a00000;
__heap_length = 0x02e00000; /* 46 MiB */

/* hot memory contains the actual user code */
__hot_start = 0x07800000;
__hot_length = 0x00800000; /* 8 MiB */

__kernel_stack_length = 8192; /* 8 KiB */
__user_stack_length = 16384; /* 16 KiB */
