use core::arch::{asm, global_asm, naked_asm};
use std::fmt::{self, Write};

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

#[derive(Clone, Copy)]
#[repr(C)]
struct Fault {
    link_register: u32,
    stack_pointer: u32,
    exception: FaultException,
    /// The address at which the abort occurred.
    ///
    /// This is calculated using the Link Register (`lr`), which is set to this address
    /// plus an offset when an exception occurs.
    ///
    /// Offsets:
    ///
    /// * [plus 8 bytes][da-exception] for data aborts.
    /// * [plus 4 bytes][pf-exception] for prefetch aborts.
    /// * [plus the size of an instruction][svc-exception] for SVCs and
    ///   undefined instruction aborts (this is different in thumb mode).
    ///
    /// [da-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Data-Abort-exception
    /// [pf-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Prefetch-Abort-exception
    /// [svc-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Supervisor-Call--SVC--exception
    program_counter: u32,
    /// Registers r0 through r12
    registers: [u32; 13],
}

/// Type of exception causing a fault to be raised.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum FaultException {
    UndefinedInstruction = 0,
    PrefetchAbort = 1,
    DataAbort = 2,
}

pub enum FaultStatus {
    DataFault(u32),
    InstructionFault(u32),
}

impl FaultStatus {
    fn source(&self) -> u8 {
        let fsr = match self {
            FaultStatus::DataFault(dfsr) => dfsr,
            FaultStatus::InstructionFault(ifsr) => ifsr,
        };

        (fsr & 0b1111) as u8
    }

    fn is_write(&self) -> bool {
        match self {
            FaultStatus::DataFault(dfsr) => {
                const WRITE_NOT_READ_BIT: u32 = 1 << 11;

                dfsr & WRITE_NOT_READ_BIT != 0
            }
            FaultStatus::InstructionFault(_ifsr) => false,
        }
    }
}

impl Fault {
    fn status(&self) -> FaultStatus {
        let status: u32;

        // https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/Virtual-Memory-System-Architecture--VMSA-/CP15-registers-for-a-VMSA-implementation/CP15-c5--Fault-status-registers
        match self.exception {
            FaultException::DataAbort => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {dfsr}, c5, c0, 0",
                    dfsr = out(reg) status,
                );

                FaultStatus::DataFault(status)
            },
            FaultException::PrefetchAbort | FaultException::UndefinedInstruction => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {ifsr}, c5, c0, 1",
                    ifsr = out(reg) status,
                );

                FaultStatus::InstructionFault(status)
            },
        }
    }

    fn address(&self) -> u32 {
        let address: u32;

        match self.exception {
            FaultException::DataAbort => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {dfar}, c6, c0, 0",
                    dfar = out(reg) address,
                );
            },
            FaultException::PrefetchAbort | FaultException::UndefinedInstruction => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {ifar}, c6, c0, 1",
                    ifar = out(reg) address,
                );
            },
        }

        address
    }
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

pub struct AbortWriter(());

impl AbortWriter {
    pub const BUFFER_SIZE: usize = 2048;
    pub const fn new() -> Self {
        Self(())
    }

    fn flush(&mut self) {
        unsafe {
            while (vex_sdk::vexSerialWriteFree(1) as usize) != Self::BUFFER_SIZE {
                vex_sdk::vexTasksRun();
            }
        }
    }
}

impl Write for AbortWriter {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        let buf = s.as_bytes();

        for chunk in buf.chunks(Self::BUFFER_SIZE) {
            if unsafe { vex_sdk::vexSerialWriteFree(1) as usize } < chunk.len() {
                self.flush();
            }

            let count: usize =
                unsafe { vex_sdk::vexSerialWriteBuffer(1, chunk.as_ptr(), chunk.len() as u32) }
                    as _;

            if count != chunk.len() {
                break;
            }
        }

        Ok(())
    }
}

#[instruction_set(arm::a32)]
unsafe extern "aapcs" fn fault_exception_handler(fault: *const Fault) -> ! {
    unsafe {
        // how and why does this work
        core::arch::asm!("cpsie i");
    }

    let fault = unsafe { *fault };
    let fault_status = fault.status();
    let fault_source = fault_status.source();

    let mut writer = AbortWriter::new();
    writer.flush();

    _ = writeln!(
        &mut writer,
        "\n{} Exception: {}",
        match fault.exception {
            FaultException::DataAbort => "Data Abort",
            FaultException::UndefinedInstruction => "Undefined Instruction",
            FaultException::PrefetchAbort => "Prefetch Abort",
        },
        // https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/Virtual-Memory-System-Architecture--VMSA-/Fault-Status-and-Fault-Address-registers-in-a-VMSA-implementation/Fault-Status-Register-encodings-for-the-VMSA?lang=en#CBHHADIB
        match fault_status {
            FaultStatus::DataFault(_) => {
                match fault_source {
                0b00001 => "Alignment fault	(MMU)",
                0b00100 => "Instruction cache maintenance fault",
                0b01100 | 0b01110 => "Translation table walk synchronous external abort",
                0b11100 | 0b11110 => "Translation table walk synchronous parity error",
                0b00101 | 0b00111 => "Translation fault (MMU)",
                0b00011 | 0b00110 => "Access Flag fault (MMU)",
                0b01001 | 0b01011 => "Domain fault (MMU)",
                0b01101 | 0b01111 => "Permission fault (MMU)",
                0b00010 => "Debug event",
                0b01000 => "Synchronous external abort",
                0b10100 => "implementation defined (Lockdown)",
                0b11010 => "implementation defined (Coprocessor abort)",
                0b11001 => "Memory access synchronous parity error",
                0b10110 => "Asynchronous external abort",
                0b11000 => "Memory access asynchronous parity error (Including on translation table walk)",
                _ => "unknown",
            }
            }
            FaultStatus::InstructionFault(_) => {
                match fault_source {
                    0b01100 | 0b01110 => "Translation table walk synchronous external abort",
                    0b11100 | 0b11110 => "Translation table walk synchronous parity error",
                    0b00101 | 0b00111 => "Translation fault (MMU)",
                    0b00011 | 0b00110 => "Access Flag fault (MMU)",
                    0b01001 | 0b01011 => "Domain fault (MMU)",
                    0b01101 | 0b01111 => "Permission fault (MMU)",
                    0b00010 => "Debug event",
                    0b01000 => "Synchronous external abort",
                    0b10100 => "implementation defined (Lockdown)",
                    0b11010 => "implementation defined (Coprocessor abort)",
                    0b11001 => "Memory access synchronous parity error",
                    _ => "unknown",
                }
            }
        }
    );

    _ = writeln!(&mut writer, "    at address {:#x}", fault.program_counter);

    let action = match fault_status {
        FaultStatus::DataFault(_) => {
            if fault_status.is_write() {
                "writing to"
            } else {
                "reading from"
            }
        }
        FaultStatus::InstructionFault(_) => "fetching instruction at",
    };

    _ = writeln!(
        &mut writer,
        "    while {action} address {:#x}",
        fault.address()
    );

    _ = writeln!(&mut writer, "\nregisters at time of fault:");

    for (i, register) in fault.registers.iter().enumerate() {
        if i > 9 {
            _ = writeln!(writer, "r{i}: {:#x} ", register);
        } else {
            _ = writeln!(writer, " r{i}: {:#x} ", register);
        }
    }

    _ = writeln!(writer, " sp: {:#x} ", fault.stack_pointer);
    _ = writeln!(writer, " lr: {:#x} ", fault.link_register);
    _ = writeln!(writer, " pc: {:#x} ", fault.program_counter);

    _ = writeln!(
        &mut writer,
        "\nhelp: This indicates the misuse of `unsafe` code. Use a symbolizer tool to determine the location of the crash."
    );

    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    _ = writeln!(
        &mut writer,
        "      (e.g. llvm-symbolizer -e ./target/armv7a-vex-v5/{profile}/program_name {:#x})",
        fault.program_counter
    );

    writer.flush();

    // #[cfg(feature = "backtrace")]
    // {
    //     let context = backtrace::make_unwind_context(backtrace::CoreRegisters {
    //         r: fault.registers,
    //         lr: fault.link_register,
    //         pc: fault.program_counter,
    //         sp: fault.stack_pointer,
    //     });
    //     _ = backtrace::print_backtrace(&mut writer, &context);
    // }

    match fault.exception {
        FaultException::DataAbort => unsafe {
            vex_sdk::vexSystemDataAbortInterrupt();
        },
        FaultException::PrefetchAbort => unsafe {
            vex_sdk::vexSystemPrefetchAbortInterrupt();
        },
        FaultException::UndefinedInstruction => unsafe {
            vex_sdk::vexSystemUndefinedException();
        },
    }

    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}

#[cfg(feature = "backtrace")]
mod backtrace {
    use std::fmt::Write;

    use vex_libunwind::{registers::UNW_REG_IP, UnwindContext, UnwindCursor, UnwindError};
    use vex_libunwind_sys::unw_context_t;

    use super::AbortWriter;

    /// https://developer.arm.com/documentation/ddi0406/b/Application-Level-Architecture/Application-Level-Programmers--Model/ARM-core-registers?lang=en
    #[repr(C)]
    pub struct CoreRegisters {
        pub r: [u32; 13],
        pub sp: u32,
        pub lr: u32,
        pub pc: u32,
    }

    #[repr(C)]
    struct RawUnwindContext {
        /// Value of each general-purpose register in the order of r0-r12, sp, lr, pc.
        core_registers: CoreRegisters,

        /// Padding (unused on ARM).
        data: [u8; const { size_of::<unw_context_t>() - size_of::<CoreRegisters>() }],
    }

    /// Create an unwind context using custom registers instead of ones captured
    /// from the current processor state.
    ///
    /// This is based on the ARM implementation of __unw_getcontext:
    /// <https://github.com/llvm/llvm-project/blob/6fc3b40b2cfc33550dd489072c01ffab16535840/libunwind/src/UnwindRegistersSave.S#L834>
    pub fn make_unwind_context(core_registers: CoreRegisters) -> UnwindContext {
        // SAFETY: `context` is a valid `unw_context_t` because it has its
        // general-purpose registers field set.
        UnwindContext::from(unsafe {
            core::mem::transmute::<RawUnwindContext, unw_context_t>(RawUnwindContext {
                core_registers,
                // This matches the behavior of __unw_getcontext, which leaves
                // this data uninitialized.
                data: [0; _],
            })
        })
    }

    pub fn print_backtrace(
        writer: &mut AbortWriter,
        context: &UnwindContext,
    ) -> Result<(), UnwindError> {
        let mut cursor = UnwindCursor::new(context)?;

        _ = writeln!(writer, "\nstack backtrace:");
        loop {
            _ = writeln!(writer, "{:#x}", cursor.register(UNW_REG_IP)?);

            if !cursor.step()? {
                break;
            }
        }
        _ = writeln!(writer);

        Ok(())
    }
}
