.section .overwriter, "ax"

.global __patcher_overwrite

__patcher_overwrite:
    @ Execute memcpy to copy our patched program from the heap to active memory.
    bl __overwriter_aeabi_memcpy

    @ vexSystemBoot
    ldr r0, =0x037fc910
    bx r0
