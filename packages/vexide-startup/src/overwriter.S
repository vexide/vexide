.section .overwriter, "ax"

.global __patcher_overwrite

__patcher_overwrite:
    @ TODO(tropix126): It might be more sound to go through vexSystemBoot here instead.

    @ Execute memcpy to copy our patched program from the heap to active memory.
    mov r0, #0x03800000                 @ memcpy dest -> r0
    mov r1, #0x07E00000                 @ memcpy src -> r1
                                        @ r2 is loaded from Rust
    bl __overwriter_aeabi_memcpy

    @ vexSystemBoot
    ldr r0, =0x037fc910
    bx r0
