//! APIs for catching and handling CPU faults.

use core::arch::{asm, global_asm, naked_asm};

pub mod fault;
pub(crate) mod report;

use fault::{ExceptionContext, ExceptionType};
use vex_sdk::{V5_TouchEvent, V5_TouchStatus, vexTasksRun, vexTouchDataGet};

use crate::{abort_handler::fault::Fault, debug::bkpt::handle_breakpoint};

// Custom ARM vector table. Pointing the VBAR coprocessor register at this will configure the CPU to
// jump to these functions on an exception.
global_asm!(
    r#"
    .text
    .arm
    .align 5
    .global vector_table

    vector_table:
        b vexSystemBoot
        b {undefined_instruction}
        b {svc}
        b {prefetch_abort}
        b {data_abort}
        nop @ Placeholder for address exception vector
        b {irq}
    @ Place the FIQ exception vector directly on the last entry of the vector table to
    @ avoid an unnecessary branch.
    @
    @ See: https://developer.arm.com/documentation/dui0056/d/handling-processor-exceptions/interrupt-handlers?lang=en#:~:text=The%20FIQ%20vector,increasing%20execution%20speed.
    fiq:
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
    "#,
    undefined_instruction = sym undefined_instruction,
    svc = sym svc,
    prefetch_abort = sym prefetch_abort,
    data_abort = sym data_abort,
    irq = sym irq,
);

/// Enable vexide's CPU exception handling logic by installing its custom vector table.
///
/// This automatically called when vexide starts, so this function is probably only useful if you
/// aren't using vexide's startup routine.
pub fn install_vector_table() {
    unsafe {
        asm!(
            // If vector_table is placed too far away, we'll need 2 instructions to load it.
            "movw r0, #:lower16:vector_table",
            "movt r0, #:upper16:vector_table",

            // Set VBAR; see <https://developer.arm.com/documentation/ddi0601/2025-06/AArch32-Registers/VBAR--Vector-Base-Address-Register>
            "mcr p15, 0, r0, c12, c0, 0",
            out("r0") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Handles a supervisor call exception.
///
/// # Safety
///
/// This function must only be invoked as an exception handler. Do not manually call this function
/// from Rust code.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[instruction_set(arm::a32)]
pub unsafe extern "C" fn svc() {
    core::arch::naked_asm!(
        "
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
        ",
    )
}

/// Handles an interrupt request exception.
///
/// # Safety
///
/// This function must only be invoked as an exception handler. Do not manually call this function
/// from Rust code.
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[instruction_set(arm::a32)]
pub unsafe extern "C" fn irq() {
    core::arch::naked_asm!(
        "
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
        ",
    )
}

macro_rules! fault_exception_vector {
    (
        $(#[$attrs:meta])*
        $name:ident:
        lr_offset = $lr_offset:expr,
        exception = $exception:expr$(,)?
    ) => {
        $(#[$attrs])*
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        #[instruction_set(arm::a32)]
        pub unsafe extern "C" fn $name() {
            naked_asm!("
                dsb @ Workaround for Cortex-A9 erratum (id 775420)

                @ Apply an offset to link register so that it points at address of return
                @ instruction (see Abort::program_counter docs).
                sub lr, lr, #{lr_offset}

                @ -- Save --
                @ Create a ExceptionContext struct on the stack.
                @ 1. Save general purpose registers for debugging and exception return
                push {{r0-r12, lr}}

                @ 2. Add secondary details: exception type and other debug info
                mov r0, {exception}
                push {{r0}}

                @ Store original SYS-mode stack pointer and link register for debug
                stmdb sp, {{sp,lr}}^
                @ Adjust our sp, since we can't use writeback on stmdb SYS
                sub sp, sp, #8

                @ 3. Save the caller's prog. status reg. in case a recursive exception happens.
                @ Note that this also helps align the stack to 8.
                mrs r0, spsr
                push {{r0}}

                @ -- Handle exception --
                @ Pass it to our handler using the C ABI, fn(*mut ExceptionContext) -> ():
                mov r0, sp                     @ set param 0
                blx {exception_handler}        @ Actually call the function now

                @ -- Restore --
                @ Restore spsr in case it was changed.
                pop {{r0}}
                msr spsr, r0

                @ Our exception handler wouldn't have modified anything in SYS-mode,
                @ so we can just discard that stuff. Also discard the exception type.
                add sp, sp, #12

                @ And... perform an exception return by loading the old LR into PC.
                @ Note that we've already offset LR as required to point to the correct address.
                ldm sp!, {{r0-r12, pc}}^
                ",
                exception_handler = sym fault_exception_handler,
                lr_offset = const $lr_offset,
                exception = const $exception as u32,
            );
        }
    };
}

fault_exception_vector!(
    /// Handles an undefined instruction exception.
    ///
    /// # Safety
    ///
    /// This function must only be invoked as an exception handler. Do not manually call this function
    /// from Rust code.
    undefined_instruction:
        lr_offset = 4,
        exception = ExceptionType::UndefinedInstruction,
);

fault_exception_vector!(
    /// Handles an prefetch abort exception.
    ///
    /// # Safety
    ///
    /// This function must only be invoked as an exception handler. Do not manually call this function
    /// from Rust code.
    prefetch_abort:
        lr_offset = 4,
        exception = ExceptionType::PrefetchAbort,
);

fault_exception_vector!(
    /// Handles a data abort exception.
    ///
    /// # Safety
    ///
    /// This function must only be invoked as an exception handler. Do not manually call this function
    /// from Rust code.
    data_abort:
        lr_offset = 8,
        exception = ExceptionType::DataAbort,
);

unsafe extern "C" fn fault_exception_handler(fault: *mut ExceptionContext) {
    // Load the fault's details from the captured program state & by querying the CPU's fault status
    // registers.
    let mut fault = unsafe { Fault::from_ptr(fault) };

    unsafe {
        // Data abort and prefetch abort exceptions disable IRQs (source: ARMv7-A TRM page B1-1213).
        //
        // This has the side-effect of breaking vexTasksRun since it disables the private timer
        // interrupt. Without VEXos background processing services, we cannot flush the
        // serial buffer, so we re-enable IRQs here to make that working.
        core::arch::asm!("cpsie i", options(nomem, nostack, preserves_flags));
    }

    // Stop all motors
    for index in 0..=20 {
        unsafe {
            vex_sdk::vexDeviceMotorVoltageSet(vex_sdk::vexDeviceGetByIndex(index), 0);
        }
    }

    if fault.is_breakpoint() {
        unsafe {
            handle_breakpoint(&mut fault);
        }
        return;
    }

    report::report_fault(&fault);

    let mut prev_touch_event = V5_TouchEvent::kTouchEventRelease;
    loop {
        let mut status = V5_TouchStatus::default();
        unsafe {
            vexTouchDataGet(&raw mut status);
        }

        if status.lastEvent == V5_TouchEvent::kTouchEventRelease
            && prev_touch_event != V5_TouchEvent::kTouchEventRelease
        {
            report::report_fault(&fault);
        }

        prev_touch_event = status.lastEvent;

        unsafe {
            vexTasksRun();
        }
    }
}
