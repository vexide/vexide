.section .overwriter, "ax"

.global __patcher_overwrite

__patcher_overwrite:
    cpsid i @ Mask interrupts

    @ Execute memcpy to copy our patched program at 0x07E00000 onto active memory.
    ldr r0, =__program_ram_start      @ memcpy dest -> r0
    ldr r1, =__patcher_base_start     @ memcpy src -> r1
    ldr r2, =__patcher_patch_start+16 @ New binary len is stored as metadata in the patch
    ldr r2, [r2]                      @ memcpy size -> r2
    bl __overwriter_aeabi_memcpy

    @ Clean L1 data cache in the user memory range
    @ NOTE: 0x03800000 is already aligned to cache line size (32), so no work there.
    ldr r1, =__patcher_ram_end @ End address of RWX user memory
dcache_clean_range:
    mcr p15, 0, r0, c7, c11, 1 @ Clean and invalidate D-cache line by MVA (DCCMVAU)
    add r0, #32                @ Move to the next cache line
    cmp r0, r1                 @ Check if we've reached the end
    blo dcache_clean_range     @ If not, continue cleaning
    dsb                        @ Wait until all cache operations complete

    @ Invalidate entire L1 instruction cache
    mov r0, #0
    mcr p15, 0, r0, c7, c1, 0 @ ICIALLUIS
    dsb                       @ Wait until all cache operations complete

    cpsie i @ Unmask interrupts

    @ Jump to modified code
    b _boot
