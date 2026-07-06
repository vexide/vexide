@ Template for fault handlers in the exception vector table.
.arm
.set PROG_STATUS.T, 1 << 5

    dsb @ Workaround for Cortex-A9 erratum (id 775420)

    @ Reset stack to the base of our custom abort stack (this is valid because we never
    @ return from this function or access stack data from a previous abort handler).
    movw sp, #:lower16:__abort_stack_top
    movt sp, #:upper16:__abort_stack_top

    @ Create a Fault struct on the stack:
    push {{r0-r12}}     @ Save general purpose registers for debugging

    mov r0, {exception}
    mrs r1, spsr

    @ Apply an offset to link register so that it points at the address
    @ of the faulting instruction (see Fault::program_counter docs).
.if {lr_offset} == {LR_OFFSET_INSTR_SIZE}
    @ This is used by UndefinedInstruction which needs an offset of 4 on ARM and 2 on Thumb.
    tst r1, #PROG_STATUS.T  @ Does spsr.thumb == 0?
    subeq lr, #4
    subne lr, #2
.else
    sub lr, #{lr_offset}
.endif

    push {{r0, r1, lr}}     @ Keep building up our struct; add `cause`, PC, program status.

    @ Store original system/user-mode stack pointer
    stmdb sp, {{sp}}^   @ Get the system mode's stack pointer
    sub sp, sp, #4      @ Adjust our sp, since we can't use writeback on stmdb SYS

    @ Store original system/user-mode link register
    stmdb sp, {{lr}}^   @ Get the system mode's link register
    sub sp, sp, #4      @ Adjust our sp, since we can't use writeback on stmdb SYS

    @ Pass it to our handler using the C ABI:
    mov r0, sp                  @ set param 0
    blx {exception_handler}     @ Actually call the function now
