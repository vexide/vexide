use core::arch::{asm, global_asm, naked_asm};

use crate::println;

// Custom ARM vector table for improved exception handling
global_asm!(
    r#"
.section .vectors, "ax"
.global vectors
.arm

vectors:
    b {boot} @ Defined by SDK
    nop @ TODO (b undefined_instruction_handler) - use data abort for now
    nop @ b svc - use data abort for now
    nop @ b prefetch_abort - use data abort for now
    b {data_abort}
    nop @ Placeholder for address exception vector
    b {irq}
    b . @ b fiq
    "#,
    boot = sym boot,
    data_abort = sym data_abort,
    irq = sym irq,
);

/// Enable vexide's CPU exception handling logic by installing
/// its custom vector table.
pub unsafe fn install_vector_table() {
    unsafe {
        asm!(
            "ldr r0, =vectors",
            // Set VBAR; see <https://developer.arm.com/documentation/ddi0601/2025-06/AArch32-Registers/VBAR--Vector-Base-Address-Register>
            "mcr p15, 0, r0, c12, c0, 0",
            // Ensure the write takes effect before any exception occurs
            // in the pipeline.
            "dsb",
            "isb",
            out("r0") _,
            options(nostack, preserves_flags)
        );
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn boot() -> ! {
    unsafe {
        vex_sdk::vexSystemBoot();

        loop {
            vex_sdk::vexTasksRun();
        }
    }
}

#[derive(Debug)]
#[repr(u32)]
enum FaultType {
    UndefinedInstruction = 0,
    SupervisorCall = 1,
    PrefetchAbort = 2,
    DataAbort = 3,
}

#[repr(C)]
struct Fault {
    cause: FaultType,
    /// The address at which the fault occurred.
    ///
    /// This is calculated using the link register, which is set to this address
    /// plus an offset when an exception occurs.
    ///
    /// Offsets:
    ///
    /// * [plus 8 bytes][da-exception] for data aborts
    /// * [plus 4 bytes][pf-exception] for prefetch aborts
    /// * [plus the size of an instruction][svc-exception] for SVCs and
    ///   undefined instruction aborts (this is different in thumb mode)
    ///
    /// [da-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Data-Abort-exception
    /// [pf-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Prefetch-Abort-exception
    /// [svc-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Supervisor-Call--SVC--exception
    instruction_pointer: u32,
    /// Registers r0 through r12
    registers: [u32; 13],
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn data_abort() -> ! {
    naked_asm!(
        "
.arm
        dsb             @ Workaround for Cortex A9 erratum (id 775420)
                        @ It's possible VEX's boards aren't susceptible to this,
                        @ but it's included out of an abundance of caution.

        @ Create `Fault` struct in the stack:
        sub sp, sp, #4    @ Align the stack: 4 extra bytes at end of struct means 64 bytes total,
                          @ which will keep the stack aligned to 8 bytes after we add all our data.
        push {{r0-r12}}   @ Save general-purpose registers for debugging
        mov r0, 3         @ Cause is `FaultType::DataAbort`. We will push this later.
        sub r1, lr, #8    @ Get instruction pointer by undoing offset (LR_abt - 8)
                          @ (see `Fault::instruction_pointer` for details on the offsets)
        push {{r0, r1}}   @ Keep building up our struct; add `cause` and I.P.

        @ Pass it to our handler using the C ABI:
        mov r0, sp                     @ set param 0
        ldr r2, ={exception_handler} + 1 @ Set up switching to thumb mode: +1 = call in thumb
        bx r2                          @ Actually call the function now
        ",
        exception_handler = sym exception_handler,
    );
}

mod interrupt_controller {
    use core::ptr;

    pub const BASE_ADDRESS: usize = 0xF8F0_1000;

    const CPU_INTERFACE_OFFSET: isize = -0xf00;
    pub const CPU_INTERFACE_ADDRESS: usize = BASE_ADDRESS.wrapping_add_signed(CPU_INTERFACE_OFFSET);

    const INTERRUPT_ACKNOWLEDGE_OFFSET: isize = 0x0C;
    pub const INTERRUPT_ACKNOWLEDGE_REGISTER_ADDRESS: usize =
        CPU_INTERFACE_ADDRESS.wrapping_add_signed(INTERRUPT_ACKNOWLEDGE_OFFSET);
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
unsafe extern "C" fn irq() {
    naked_asm!(
        "
.arm
        @ Requirements for reentrant interrupt handlers:
        @ <https://developer.arm.com/documentation/dui0203/j/handling-processor-exceptions/armv6-and-earlier--armv7-a-and-armv7-r-profiles/interrupt-handlers>

        sub lr, lr, #4  @ Apply an offset to link register so that it points at address
                        @ of return instruction (see Fault::instruction_pointer docs).
        srsdb sp!, #19  @ Save LR & SPSR to supervisor mode stack

        @ Switch to supervisor mode
        @ Note: traditionally you would switch to system mode here,
        @ but PROS uses supervisor mode when it calls the IRQ handler
        @ (and it works) so we're going to use that behavior. This is
        @ presumably because user programs run in system mode by default.
        cps #19

        @ These registers aren't preserved by the APPCS calling convention
        @ or our own assembly code so we save them on the stack to make sure
        @ they're the same when we return from the IRQ.
        push {{r0-r3,r12}}

        @ Align the stack to 8 bytes by subtracting 4 bytes if neccesary
        @ r1 holds the adjustment so we can undo it with an `add` later
        and r1, sp, #0b0100   @ Is the `4` bit set? If so, set r1 to 4
        sub sp, sp, r1          @ Align stack by giving extra space if needed

        @ Pull interrupt id out of the the interrupt controller's ICCIAR register
        @ (Reading it automatically awknowledges the IRQ.)
        @ r0 is used as first param to function call
        ldr r0, ={icciar}   @ Store constant, then dereference it
        ldr r0, [r0]

        @ Call interrupt handler
        push {{r1,lr}}              @ AAPCS callee-saved registers (only the ones we're using)
        ldr r2, ={irq_handler} + 1  @ Set up switching to thumb mode: +1 = call in thumb
        bx r2                       @ Actually call the function now
        pop {{r1,lr}}               @ Restore callee-saved registers

        @ Disable IRQs again in case our handler enabled them.
        @ We're not able to handle reentrancy right now since
        @ our code is restoring the state from before *this* IRQ.
        cpsid i   @ Turn off the IRQ flag.
        dsb       @ Ensure the change is propogated before
        isb       @ we continue executing.

        @ Undo the changes we made at the beginning to prevent side effects
        add sp, sp, r1      @ Revert stack alignment
        pop {{r0-r3,r12}}   @ Restore general purpose registers
        rfeia sp!           @ Return from exception: restore SPSR (into CPSR register)
                            @ and LR from the stack, then jump to LR.
                            @ This is the counterpart to the `srsdb` call
        ",
        icciar = const interrupt_controller::INTERRUPT_ACKNOWLEDGE_REGISTER_ADDRESS,
        irq_handler = sym irq_handler,
    );
}

extern "C" fn exception_handler(fault: &mut Fault) -> ! {
    println!(">>>> Abort! Code: {:?} <<<<", fault.cause);
    unsafe {
        vex_sdk::vexDisplayForegroundColor(0xFF_00_00);
        vex_sdk::vexDisplayRectFill(0, 0, 500, 500);
        vex_sdk::vexSystemUndefinedException();
    }

    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}

/// Processes an Interrupt Request (IRQ) with the given ID.
///
/// The ID should be obtained from one of the Interrupt Acknowledge Registers (IARs).
unsafe extern "C" fn irq_handler(interrupt_id: u32) {
    unsafe {
        vex_sdk::vexSystemApplicationIRQHandler(interrupt_id);
    }
}
