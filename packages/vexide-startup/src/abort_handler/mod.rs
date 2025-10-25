use core::arch::{asm, global_asm, naked_asm};

mod fault;
mod report;

use fault::{Fault, FaultException};
use vex_sdk::{V5_TouchEvent, V5_TouchStatus, vexTasksRun, vexTouchDataGet};

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

#[unsafe(naked)]
#[unsafe(no_mangle)]
#[instruction_set(arm::a32)]
pub extern "aapcs" fn svc() -> ! {
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
                                            @ of return instruction (see Abort::program_counter docs).

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

    let fault = unsafe { *fault };

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
