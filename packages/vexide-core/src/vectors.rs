use core::arch::{asm, global_asm, naked_asm};

use crate::println;

// Custom ARM vector table for improved exception handling
global_asm!(
    r#"
.section .vectors, "ax"
.global vectors
.arm

vectors:
    b vexSystemBoot_wrapper @ Defined by SDK
    nop @ TODO (b undefined_instruction_handler) - use data abort for now
    nop @ b svc - use data abort for now
    nop @ b prefetch_abort - use data abort for now
    b data_abort
    nop @ Placeholder for address exception vector
    b . @ b irq
    b . @ b fiq
"#
);

/// Enable vexide's CPU exception handling logic by installing
/// its custom vector table.
pub unsafe fn install_vector_table() {
    unsafe {
        asm!(
            "ldr r0, =vectors",
            // Set VBAR; see <https://developer.arm.com/documentation/ddi0601/2025-06/AArch32-Registers/VBAR--Vector-Base-Address-Register>
            "mcr p15, 0, r0, c12, c0, 0",
            out("r0") _,
        );
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn vexSystemBoot_wrapper() -> ! {
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
        ldr r2, =exception_handler + 1 @ Set up switching to thumb mode: +1 = call in thumb
        bx r2                          @ Actually call the function now
        ",
    );
}

#[unsafe(no_mangle)]
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
