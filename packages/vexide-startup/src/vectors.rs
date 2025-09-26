use std::sync::atomic::{AtomicBool, Ordering};
use core::arch::{asm, global_asm, naked_asm};

// Custom ARM vector table. Pointing the VBAR coprocessor. register at this
// will configure the CPU to jump to these functions on an exception.
global_asm!(
    r#"
.section .vectors, "ax"
.align 5
.global vectors
.arm

vectors:
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
            "ldr r0, =vectors",
            // Set VBAR; see <https://developer.arm.com/documentation/ddi0601/2025-06/AArch32-Registers/VBAR--Vector-Base-Address-Register>
            "mcr p15, 0, r0, c12, c0, 0",
            out("r0") _,
            options(nostack, preserves_flags)
        );
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum FaultType {
    UndefinedInstruction = 0,
    SupervisorCall = 1,
    PrefetchAbort = 2,
    DataAbort = 3,
}

#[repr(C)]
struct Fault {
    stack_pointer: u32,
    cause: FaultType,
    /// The address at which the fault occurred.
    ///
    /// This is calculated using the Link Register (`lr`), which is set to this address
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
    instruction_address: u32,
    /// Registers r0 through r12
    registers: [u32; 13],
}

#[expect(edition_2024_expr_fragment_specifier)]
macro_rules! abort_handler {
    (
        $name:ident:
        lr_offset = $lr_offset:expr,
        cause = $cause:expr$(,)?
    ) => {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        #[instruction_set(arm::a32)]
        unsafe extern "C" fn $name() -> ! {
            naked_asm!(
                "
        .arm
                dsb             @ Workaround for Cortex-A9 erratum (id 775420)

                sub lr, lr, #{lr_offset}    @ Apply an offset to link register so that it points at address
                                            @ of return instruction (see Fault::instruction_pointer docs).

                @ Create `Fault` struct in the stack:
                push {{r0-r12}}     @ Save general-purpose registers for debugging
                mov r0, {cause}     @ We will push this later.
                push {{r0, lr}}     @ Keep building up our struct; add `cause` and I.P.

                @ Storing original stack pointer needs two steps
                stmdb sp, {{sp}}^  @ Get the user/system mode's stack pointer
                sub sp, sp, #4     @ Move *our* stack pointer to the beginning of the struct

                @ Pass it to our handler using the C ABI:
                mov r0, sp                     @ set param 0
                blx {exception_handler}        @ Actually call the function now
                ",
                exception_handler = sym exception_handler,
                lr_offset = const $lr_offset,
                cause = const $cause as u32,
            );
        }
    };
}

abort_handler!(undefined_instruction: lr_offset = 4, cause = FaultType::UndefinedInstruction);
abort_handler!(prefetch_abort: lr_offset = 4, cause = FaultType::PrefetchAbort);
abort_handler!(data_abort: lr_offset = 8, cause = FaultType::DataAbort);

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub extern "C" fn svc() -> ! {
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
pub extern "C" fn fiq() -> ! {
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
pub extern "C" fn irq() -> ! {
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

unsafe extern "C" fn exception_handler(fault: *const Fault) -> ! {
    unsafe {
        // how and why
        core::arch::asm!("cpsie i");
    }

    unsafe {
        let Fault {
            cause,
            instruction_address: instruction_pointer,
            stack_pointer,
            registers,
        } = *fault;
        println!("\n\n*** {cause:?} EXCEPTION ***");
        println!("  at address 0x{instruction_pointer:x?}");

        match cause {
            FaultType::DataAbort => {
                let abort = data_abort_info();
                let verb = if abort.is_write() {
                    "writing to"
                } else {
                    "reading from"
                };

                println!("  while {verb} address 0x{:x?}", abort.address);
            }
            FaultType::UndefinedInstruction => {},
            FaultType::SupervisorCall => {},
            FaultType::PrefetchAbort => {},
        }

        println!("\nCPU registers at time of fault (r0-r12, in hexadecimal):\n{registers:x?}\n");
        println!("help: This indicates the misuse of `unsafe` code.");
        println!("      Use a symbolizer tool to determine the file and line of the crash.");

        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        println!("      (e.g. llvm-symbolizer -e ./target/armv7a-vex-v5/{profile}/program_name 0x{instruction_pointer:x?})");

        #[cfg(feature = "backtrace")]
        {
            let gprs = backtrace::GPRS {
                r: registers,
                lr: instruction_pointer,
                pc: instruction_pointer,
                sp: stack_pointer,
            };
            let context = backtrace::make_unwind_context(gprs);
            let result = backtrace::print_backtrace(&context);
            println!("backtrace result: {result:?}");
        }

        match (*fault).cause {
            FaultType::DataAbort => vex_sdk::vexSystemDataAbortInterrupt(),
            FaultType::PrefetchAbort => vex_sdk::vexSystemPrefetchAbortInterrupt(),
            FaultType::SupervisorCall => vex_sdk::vexSystemSWInterrupt(),
            FaultType::UndefinedInstruction => vex_sdk::vexSystemUndefinedException(),
        }

        loop {
            vex_sdk::vexTasksRun();
        }
    }
}

/// A representation of a data abort exception
#[derive(Debug, Clone, Copy)]
struct DataAbort {
    /// A bitfield with information about the fault.
    status: u32,
    /// The address at which the last fault occurred
    address: u32,
}

impl DataAbort {
    /// Returns whether the abort was caused by a write operation.
    pub const fn is_write(self) -> bool {
        const WRITE_NOT_READ_BIT: u32 = 1 << 11;
        self.status & WRITE_NOT_READ_BIT != 0
    }
}

fn data_abort_info() -> DataAbort {
    let address: u32;
    let status: u32;

    unsafe {
        core::arch::asm!(
            "mrc p15, 0, {fsr}, c5, c0, 0",
            "mrc p15, 0, {far}, c6, c0, 0",
            fsr = out(reg) status,
            far = out(reg) address,
        );
    }

    DataAbort { status, address }
}

#[cfg(feature = "backtrace")]
mod backtrace {
    use core::mem::transmute;

    use vex_libunwind::{registers::UNW_REG_IP, UnwindContext, UnwindCursor, UnwindError};
    use vex_libunwind_sys::{unw_context_t, unw_init_local, CONTEXT_SIZE};

    #[repr(C)]
    pub struct GPRS {
        pub r: [u32; 13],
        pub sp: u32,
        pub lr: u32,
        pub pc: u32,
    }

    const REGISTERS_DATA_SIZE: usize = size_of::<unw_context_t>() - size_of::<GPRS>();
    #[repr(C)]
    struct Registers_arm {
        gprs: GPRS,
        data: [u8; REGISTERS_DATA_SIZE],
    }

    /// Create an unwind context using custom registers instead of ones captured
    /// from the current processor state.
    ///
    /// This is based on the ARM implementation of __unw_getcontext:
    /// <https://github.com/llvm/llvm-project/blob/6fc3b40b2cfc33550dd489072c01ffab16535840/libunwind/src/UnwindRegistersSave.S#L834>
    pub fn make_unwind_context(gprs: GPRS) -> UnwindContext {
        let context = Registers_arm {
            gprs,
            // This matches the behavior of __unw_getcontext, which leaves
            // this data uninitialized.
            data: [0; REGISTERS_DATA_SIZE],
        };

        // SAFETY: `context` is a valid `unw_context_t` because it has its
        // general-purpose registers field set.
        UnwindContext::from(unsafe { transmute::<Registers_arm, unw_context_t>(context) })
    }

    pub fn print_backtrace(context: &UnwindContext) -> Result<(), UnwindError> {
        let mut cursor = UnwindCursor::new(context)?;

        println!("\nstack backtrace:");
        loop {
            println!("0x{:x?}", cursor.register(UNW_REG_IP)?);

            if !cursor.step()? {
                break;
            }
        }
        println!();

        Ok(())
    }
}
