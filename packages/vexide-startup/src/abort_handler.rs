use core::arch::{asm, global_asm, naked_asm};

use self::fault::FaultException;

#[cfg(feature = "backtrace")]
mod backtrace;
mod fault;
mod report;

/// Enable vexide's CPU exception handling logic by installing
/// its custom vector table. (Temporary internal API)
pub fn install_vector_table() {
    unsafe {
        asm!(
            "ldr r0, =vector_table",
            // Set VBAR; see <https://developer.arm.com/documentation/ddi0601/2025-06/AArch32-Registers/VBAR--Vector-Base-Address-Register>
            "mcr p15, 0, r0, c12, c0, 0",
            out("r0") _,
            options(nostack, preserves_flags)
        );
    }
}

// Custom ARM vector table. Pointing the VBAR coprocessor. register at this
// will configure the CPU to jump to these functions on an exception.
global_asm!(
    r#"
.section .vector_table, "ax"
.align 5
.global vector_table
.arm

vector_table:
    b vexSystemBoot
    b {undefined_instruction}
    b {svc}
    b {prefetch_abort}
    b {data_abort}
    nop @ Placeholder for address exception vector
    b {irq}
    b {fiq}
    "#,
    undefined_instruction = sym undefined_instruction,
    svc = sym svc,
    prefetch_abort = sym prefetch_abort,
    data_abort = sym data_abort,
    irq = sym irq,
    fiq = sym fiq,
);

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[instruction_set(arm::a32)]
pub extern "aapcs" fn svc() -> ! {
    core::arch::naked_asm!(
        "
        stmdb sp!,{{r0-r3,r12,lr}}

        tst	r0, #0x20
        ldreq r0, [lr,#-4]
        biceq r0, r0, #0xff000000
        bl vexSystemSWInterrupt

        ldmia sp!,{{r0-r3,r12,lr}}
        movs pc, lr
        ",
    )
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[instruction_set(arm::a32)]
pub extern "aapcs" fn fiq() -> ! {
    core::arch::naked_asm!(
        "
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
        ",
    )
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[instruction_set(arm::a32)]
pub extern "aapcs" fn irq() -> ! {
    core::arch::naked_asm!(
        "
            stmdb sp!,{{r0-r3,r12,lr}}

            vpush {{d0-d7}}
            vpush {{d16-d31}}
            vmrs r1, FPSCR
            push {{r1}}
            vmrs r1, FPEXC
            push {{r1}}

            bl vexSystemIQRQnterrupt

            pop {{r1}}
            vmsr FPEXC, r1
            pop {{r1}}
            vmsr FPSCR, r1
            vpop {{d16-d31}}
            vpop {{d0-d7}}

            ldmia sp!,{{r0-r3,r12,lr}}
            subs pc, lr, #4
        ",
    )
}



#[expect(edition_2024_expr_fragment_specifier)]
macro_rules! fault_exception_vector {
    (
        $name:ident:
        lr_offset = $lr_offset:expr,
        exception = $exception:expr$(,)?
    ) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        #[instruction_set(arm::a32)]
        unsafe extern "C" fn $name() -> ! {
            naked_asm!(
                "
                dsb @ Workaround for Cortex-A9 erratum (id 775420)

                sub lr, lr, #{lr_offset}    @ Apply an offset to link register so that it points at address
                                            @ of return instruction (see Abort::instruction_pointer docs).

                @ Create a Fault struct on the stack:
                push {{r0-r12}} @ Save general purpose registers for debugging
                mov r0, {exception} @ We will push this later.
                push {{r0, lr}} @ Keep building up our struct; add `cause` and I.P.

                @ Store original SYS-mode stack pointer
                stmdb sp, {{sp}}^ @ Get the system mode's stack pointer
                sub sp, sp, #4 @ Adjust our sp, since we can't use writeback on stmdb SYS

                @ Store original SYS-mode link register
                stmdb sp, {{lr}}^ @ Get the system mode's link register
                sub sp, sp, #4 @ Adjust our sp, since we can't use writeback on stmdb SYS

                @ Pass it to our handler using the C ABI:
                mov r0, sp                     @ set param 0
                blx {exception_handler}        @ Actually call the function now
                ",
                exception_handler = sym report::fault_exception_handler,
                lr_offset = const $lr_offset,
                exception = const $exception as u32,
            );
        }
    };
}

fault_exception_vector!(undefined_instruction: lr_offset = 4, exception = FaultException::UndefinedInstruction);
fault_exception_vector!(prefetch_abort: lr_offset = 4, exception = FaultException::PrefetchAbort);
fault_exception_vector!(data_abort: lr_offset = 8, exception = FaultException::DataAbort);
