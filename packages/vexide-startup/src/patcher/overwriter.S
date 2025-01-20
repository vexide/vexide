.section .overwriter, "ax"

.global __patcher_overwrite

__patcher_overwrite:
    @ TODO(tropix126): It might be more sound to go through vexSystemBoot here instead.

    @ Execute memcpy to copy our patched program from the heap to active memory.
    mov r0, #0x03800000                 @ memcpy dest -> r0
    mov r1, #0x07E00000                 @ memcpy src -> r1
    ldr r2, =0x07A00010                 @ memcpy size -> r2
    ldr r2, [r2]

    bl __overwriter_aeabi_memcpy

    @ Disable all the things while we do cache maintenance.
    mrc p15, 0, r1, c1, c0, 0           @ SCTLR -> r1
    mov r0, r1                          @ Save initial SCTLR into r0
    bic r1, r1, #1                      @ Disable MMU
    bic r1, r1, #(1 << 12)              @ Disable icache
    bic r1, r1, #(1 << 2)               @ Disable dcache and L2C
    mcr p15, 0, r1, c1, c0, 0           @ r1 -> SCTLR

    @ Cache maintenance
    mov r1, #0
    mcr p15, 0, r1, c7, c5, 0           @ Invalidate icache
    mcr p15, 0, r1, c7, c5, 6           @ Invalidate branch predictor
    mcr p15, 0, r1, c8, c7, 0           @ Invalidate main TLB
    isb

    @ Restore initial SCTLR from r0.
    mcr p15, 0, r0, c1, c0, 0           @ r0 -> SCTLR

    @ Branch to _boot again with our now-patched program.
    b _boot
