.section .overwriter, "ax"

.global __patcher_overwrite

__patcher_overwrite:
1:  @ Execute memcpy to copy our patched program from the heap to active memory.
    bl __overwriter_aeabi_memcpy

2:  @ Disable all the things while we do cache maintenance.
    cpsid i                             @ Interrupts OFF
    mrc p15, 0, r1, c1, c0, 0           @ SCTLR -> r1
    mov r0, r1                          @ Save initial SCTLR into r0
    bic r1, r1, #1                      @ Disable MMU
    bic r1, r1, #(1 << 12)              @ Disable icache
    bic r1, r1, #(1 << 2)               @ Disable dcache and L2C
    mcr p15, 0, r1, c1, c0, 0           @ r1 -> SCTLR

3:  @ Cache maintenance
    mov r1, #0
    mcr p15, 0, r1, c7, c5, 0           @ Invalidate icache
    mcr p15, 0, r1, c7, c5, 6           @ Invalidate branch predictor
    mcr p15, 0, r1, c8, c7, 0           @ Invalidate main TLB
    isb

4:  @ Restore initial SCTLR from r0.
    mcr p15, 0, r0, c1, c0, 0           @ r0 -> SCTLR
    cpsie i                             @ Interrupts ON

5:  @ Branch to _boot again with our now-patched program.
    b _boot
