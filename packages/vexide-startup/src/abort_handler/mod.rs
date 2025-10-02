use core::arch::{asm, global_asm, naked_asm};

mod fault;
mod report;

use fault::{Fault, FaultException};
use vex_sdk::{V5_TouchEvent, V5_TouchStatus, vexTasksRun, vexTouchDataGet};

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

/// Enable vexide's CPU exception handling logic by installing
/// its custom vector table. (Temporary internal API)
pub fn install_vector_table() {
    unsafe {
        asm!(
            "mrs r0, cpsr",

            // abort
            "bic r0, r0, #0b11111",
            "orr r0, r0, #0b10111",
            "msr cpsr_c, r0",
            "ldr sp, =__abort_stack_top",

            // undefined
            "bic r0, r0, #0b11111",
            "orr r0, r0, #0b11011",
            "msr cpsr_c, r0",
            "ldr sp, =__undefined_stack_top",

            // back to sys
            "orr r0, r0, #0b11111",
            "msr cpsr_c, r0",

            "ldr r0, =vector_table",
            // Set VBAR; see <https://developer.arm.com/documentation/ddi0601/2025-06/AArch32-Registers/VBAR--Vector-Base-Address-Register>
            "mcr p15, 0, r0, c12, c0, 0",
            out("r0") _,
            options(nostack, preserves_flags)
        );
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
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

                sub sp, sp, #4 @ Ensure stack is aligned to 8 after we're done here.

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
                exception_handler = sym fault_exception_handler,
                lr_offset = const $lr_offset,
                exception = const $exception as u32,
            );
        }
    };
}

fault_exception_vector!(undefined_instruction: lr_offset = 4, exception = FaultException::UndefinedInstruction);
fault_exception_vector!(prefetch_abort: lr_offset = 4, exception = FaultException::PrefetchAbort);
fault_exception_vector!(data_abort: lr_offset = 8, exception = FaultException::DataAbort);

pub unsafe extern "C" fn fault_exception_handler(fault: *const Fault) -> ! {
    unsafe {
        // For some reason IRQ interrupts get disabled on abort. It's unclear why,
        // there's not a ton of info online about this.
        // These are required for serial flushing to work - turn them back on.
        core::arch::asm!("cpsie i", options(nomem, nostack, preserves_flags));
    }

    let fault = unsafe { *fault };

    report::report_fault(&fault);

    let mut prev_touch_event = V5_TouchEvent::kTouchEventPress;
    loop {
        let mut status = V5_TouchStatus::default();
        unsafe {
            vexTouchDataGet(&raw mut status);
        }

        if status.lastEvent == V5_TouchEvent::kTouchEventRelease && status.lastEvent != prev_touch_event {
            report::report_fault(&fault);
        }

        prev_touch_event = status.lastEvent;

        unsafe {
            vexTasksRun();
        }
    }
}
