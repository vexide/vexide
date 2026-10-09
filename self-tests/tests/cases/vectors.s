@ Custom ARM vector table used by the ISR self-tests.
@
@ This mirrors the table in vexide-startup (see packages/vexide-startup/src/vectors/vectors.s),
@ reusing its fault handlers verbatim. The only entry that differs is IRQ, which routes through
@ selftest_irq so tests can install a callback that runs on every interrupt.
.text
.arm

.align 5
.global selftest_vector_table
selftest_vector_table:
    b vexSystemBoot
    b vexide_undefined_instruction
    b vexide_supervisor_call
    b vexide_prefetch_abort
    b vexide_data_abort
    nop @ Placeholder, unused on this platform.
    b selftest_irq
    b vexide_fiq

.global selftest_irq
.type selftest_irq, %function
selftest_irq:
    stmdb sp!,{{r0-r3,r12,lr}}

    vpush {{d0-d7}}
    vpush {{d16-d31}}
    vmrs r1, FPSCR
    push {{r1}}
    vmrs r1, FPEXC
    push {{r1}}

    blx {irq_handler}
    bl vexSystemIRQInterrupt

    pop {{r1}}
    vmsr FPEXC, r1
    pop {{r1}}
    vmsr FPSCR, r1
    vpop {{d16-d31}}
    vpop {{d0-d7}}

    ldmia sp!,{{r0-r3,r12,lr}}
    subs pc, lr, #4
