.text
.arm

@ Custom ARM vector table. Pointing the VBAR coprocessor register at this will configure the CPU to
@ jump to these functions on an exception.
.align 5
.global vector_table
vector_table:
    b vexSystemBoot
    b {undefined_instruction}
    b vexide_supervisor_call
    b {prefetch_abort}
    b {data_abort}
    nop @ Placeholder for address exception vector
    b vexide_irq
@ Place the FIQ exception vector directly on the last entry of the vector table to
@ avoid an unnecessary branch.
@
@ See: https://developer.arm.com/documentation/dui0056/d/handling-processor-exceptions/interrupt-handlers?lang=en#:~:text=The%20FIQ%20vector,increasing%20execution%20speed.
.global vexide_fiq
vexide_fiq:
    @ NOTE: In FIQ mode r12 is banked, so saving it to the stack here is unnecessary,
    @       but we still do it anyways to maintain stack alignment.
    stmdb sp!,{{r0-r3,r12,lr}}

    vpush {{d0-d7}}
    vpush {{d16-d31}}
    vmrs r1, FPSCR
    push {{r1}}
    vmrs r1, FPEXC
    push {{r1}}

    bl vexSystemFIQInterrupt

    pop {{r1}}
    vmsr FPEXC, r1
    pop {{r1}}
    vmsr FPSCR, r1
    vpop {{d16-d31}}
    vpop {{d0-d7}}

    ldmia sp!,{{r0-r3,r12,lr}}
    subs pc, lr, #4


.global vexide_supervisor_call
vexide_supervisor_call:
    @ Save state and SPSR so we can restore it later. Saving SPSR matches
    @ the behavior of the ARM sample SVC handler:
    @ https://developer.arm.com/documentation/dui0203/j/handling-processor-exceptions/armv6-and-earlier--armv7-a-and-armv7-r-profiles/svc-handlers?lang=en
    @ This is intended to prevent modification inside vexSystemSWInterrupt from
    @ corrupting the register.

    push {{r0-r3,r12,lr}}
    mrs r0, spsr
    push {{r0,r3}} @ r3 is a random register to maintain alignment

    @ Extract the SVC immediate number from the instruction and place it into r0.
    @ This is intended to match the behavior of Xilinx's embeddedsw SVC handler.
    @
    @ The way we do this depends on whether or not user code was running in ARM
    @ or Thumb mode at the time of this exception, so we check the T-bit in SPSR
    @ to determine this.

    @ T-bit check (spsr was placed into r0 a few lines above)
    tst	r0, #0x20

    @ Thumb mode
    ldrhne r0, [lr,#-2]
    bicne r0, r0, #0xff00

    @ ARM mode
    ldreq r0, [lr,#-4]
    biceq r0, r0, #0xff000000

    @ Call VEXos interrupt handler as fn(svc_comment) -> ()
    bl vexSystemSWInterrupt

    @ Restore spsr, other registers. Then return.
    pop {{r0,r3}}
        @ Only the cxsf groups are restored because writing to the entire thing could cause issues.
        @ These bits restore things like interrupt state, condition flags, etc.
    msr spsr_cxsf, r0
    ldmia sp!, {{r0-r3,r12,pc}}^


.global vexide_irq
vexide_irq:
    stmdb sp!,{{r0-r3,r12,lr}}

    vpush {{d0-d7}}
    vpush {{d16-d31}}
    vmrs r1, FPSCR
    push {{r1}}
    vmrs r1, FPEXC
    push {{r1}}

    bl vexSystemIRQInterrupt

    pop {{r1}}
    vmsr FPEXC, r1
    pop {{r1}}
    vmsr FPSCR, r1
    vpop {{d16-d31}}
    vpop {{d0-d7}}

    ldmia sp!,{{r0-r3,r12,lr}}
    subs pc, lr, #4
