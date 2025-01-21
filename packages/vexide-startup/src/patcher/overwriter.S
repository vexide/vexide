.section .overwriter, "ax"

.global __patcher_overwrite

__patcher_overwrite:
    cpsid i                             @ Interrupts OFF

    @ Execute memcpy to copy our patched program at 0x07E00000 onto active memory.
    mov r0, #0x03800000                 @ memcpy dest -> r0
    mov r1, #0x07E00000                 @ memcpy src -> r1
    ldr r2, =0x07A00010                 @ memcpy size -> r2
    ldr r2, [r2]

    bl __overwriter_aeabi_memcpy

    @ Ensure coherency between L1 cache and main memory
    dsb
    mov r0, #0
    mcr p15, 0, r0, c7, c5, 0           @ Invalidate instruction cache (ICIALLU)
    mcr p15, 0, r0, c7, c5, 6           @ Invalidate branch predictor array (BPIALL)
    dsb
    isb

    cpsie i                             @ Interrupts ON

    @ Branch to _boot again with our now-patched program.
    b _boot
