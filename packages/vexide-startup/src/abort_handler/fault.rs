use std::fmt::{Display, Formatter};

#[cfg(all(target_os = "vexos", feature = "backtrace"))]
use vex_libunwind::UnwindContext;
#[cfg(all(target_os = "vexos", feature = "backtrace"))]
use vex_libunwind_sys::unw_context_t;

pub struct Fault {
    pub ctx: ExceptionContext,
    pub target: u32,
    pub status: FaultStatus,
}

impl Fault {
    /// Load a fault's details from an exception context pointer.
    ///
    /// This function is intended to be called early in the handling of an exception.
    ///
    /// # Safety
    ///
    /// This function accesses CPU state that's set post-exception. The caller must ensure that this
    /// state has not been invalidated.
    pub unsafe fn from_ptr(ctx: *const ExceptionContext) -> Self {
        let ctx = unsafe { *ctx };

        Self {
            target: unsafe { ctx.target() },
            status: unsafe { ctx.status() },
            ctx,
        }
    }

    pub fn is_breakpoint(&self) -> bool {
        self.ctx.exception == ExceptionType::PrefetchAbort
            && self.status.details == FaultDetails::DebugEvent
    }
}

impl Display for Fault {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.ctx.exception {
            ExceptionType::DataAbort => {
                let details = self.status.details;
                let addr = self.target;
                let action = if self.status.write_not_read {
                    "writing to"
                } else {
                    "reading from"
                };

                write!(f, "{details} while {action} 0x{addr:x}")?;
            }
            ExceptionType::PrefetchAbort => {
                let details = self.status.details;
                let addr = self.target;

                write!(f, "{details} while fetching address 0x{addr:x}")?;
            }
            ExceptionType::UndefinedInstruction => {
                let undefd_instr =
                    unsafe { core::ptr::read_volatile(self.ctx.program_counter as *const u32) };

                write!(f, "Failed to decode instruction 0x{undefd_instr:x}")?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ExceptionContext {
    pub link_register: u32,
    pub stack_pointer: u32,
    pub exception: ExceptionType,
    /// The address at which the abort occurred.
    ///
    /// This is calculated using the Link Register (`lr`), which is set to this address plus an
    /// offset when an exception occurs.
    ///
    /// Offsets:
    ///
    /// * [plus 8 bytes][da-exception] for data aborts.
    /// * [plus 4 bytes][pf-exception] for prefetch aborts.
    /// * [plus the size of an instruction][svc-exception] for SVCs and undefined instruction
    ///   aborts (this is different in thumb mode).
    ///
    /// [da-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Data-Abort-exception
    /// [pf-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Prefetch-Abort-exception
    /// [svc-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Supervisor-Call--SVC--exception
    pub program_counter: u32,
    /// Registers r0 through r12
    pub registers: [u32; 13],
}

impl ExceptionContext {
    /// Create an unwind context using custom registers instead of ones captured from the current
    /// processor state.
    ///
    /// This is based on the ARM implementation of __unw_getcontext:
    /// <https://github.com/llvm/llvm-project/blob/6fc3b40b2cfc33550dd489072c01ffab16535840/libunwind/src/UnwindRegistersSave.S#L834>
    #[cfg(all(target_os = "vexos", feature = "backtrace"))]
    pub unsafe fn unwind_context(&self) -> UnwindContext<'_> {
        #[repr(C)]
        struct RawUnwindContext {
            // Value of each general-purpose register in the order of r0-r12, sp, lr, pc.
            r: [u32; 13],
            sp: u32,
            lr: u32,
            pc: u32,

            /// Padding (unused on ARM).
            data: [u8; const { size_of::<unw_context_t>() - size_of::<u32>() * 16 }],
        }

        // SAFETY: `context` is a valid `unw_context_t` because it has its general-purpose registers
        // field set.
        unsafe {
            UnwindContext::from_raw(core::mem::transmute::<RawUnwindContext, unw_context_t>(
                RawUnwindContext {
                    r: self.registers,
                    sp: self.stack_pointer,
                    lr: self.link_register,
                    pc: self.program_counter,
                    // This matches the behavior of __unw_getcontext, which leaves
                    // this data uninitialized.
                    data: [0; _],
                },
            ))
        }
    }
}

/// Type of exception causing a fault to be raised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExceptionType {
    UndefinedInstruction = 0,
    PrefetchAbort = 1,
    DataAbort = 2,
}

impl Display for ExceptionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ExceptionType::DataAbort => "Data Abort",
            ExceptionType::UndefinedInstruction => "Undefined Instruction",
            ExceptionType::PrefetchAbort => "Prefetch Abort",
        })
    }
}

impl ExceptionContext {
    /// Load the address or instruction which the faulting instruction attempted to operate on.
    ///
    /// # Safety
    ///
    /// This function accesses CPU state that's set post-exception. The caller must ensure that this
    /// state has not been invalidated.
    pub unsafe fn target(&self) -> u32 {
        let target: u32;

        match self.exception {
            ExceptionType::DataAbort => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {dfar}, c6, c0, 0",
                    dfar = out(reg) target,
                    options(nomem, nostack, preserves_flags)
                );

                target
            },
            ExceptionType::PrefetchAbort => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {ifar}, c6, c0, 1",
                    ifar = out(reg) target,
                    options(nomem, nostack, preserves_flags)
                );

                target
            },
            ExceptionType::UndefinedInstruction => unsafe {
                // This was an undefined instruction, not a prefetch abort, so presumably
                // the instruction is valid for access.
                core::ptr::read_volatile(self.program_counter as *const u32)
            },
        }
    }

    /// Load additional details about the circumstances of the fault.
    ///
    /// # Safety
    ///
    /// This function accesses CPU state that's set post-exception. The caller must ensure that this
    /// state has not been invalidated.
    pub unsafe fn status(&self) -> FaultStatus {
        const DFSR_WRITE_NOT_READ_BIT: u32 = 1 << 11;
        let fsr: u32;

        match self.exception {
            ExceptionType::DataAbort => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {dfsr}, c5, c0, 0",
                    dfsr = out(reg) fsr,
                );

                FaultStatus {
                    details: FaultDetails::from(fsr),
                    write_not_read: (fsr & DFSR_WRITE_NOT_READ_BIT) != 0,
                }
            },
            ExceptionType::PrefetchAbort => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {ifsr}, c5, c0, 1",
                    ifsr = out(reg) fsr,
                );

                FaultStatus {
                    details: FaultDetails::from(fsr),
                    write_not_read: false,
                }
            },
            ExceptionType::UndefinedInstruction => FaultStatus {
                details: FaultDetails::Unknown,
                write_not_read: false,
            },
        }
    }
}

pub struct FaultStatus {
    pub details: FaultDetails,
    pub write_not_read: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FaultDetails {
    Unknown,
    AlignmentFaultMMU,
    DebugEvent,
    AccessFlagFaultMMU,
    InstructionCacheMaintenanceFault,
    TranslationFaultMMU,
    SynchronousExternalAbort,
    DomainFaultMMU,
    TranslationTableWalkSynchronousExternalAbort,
    PermissionFaultMMU,
    TLBConflictAbort,
    ImplementationDefinedLockdown,
    AsynchronousExternalAbort,
    MemoryAccessAsynchronousParityError,
    MemoryAccessSynchronousParityError,
    ImplementationDefinedCoprocessorAbort,
    TranslationTableWalkSynchronousParityError,
}

impl From<u32> for FaultDetails {
    fn from(value: u32) -> Self {
        // See: ARMv7-A reference, Table B3-23 Short-descriptor format FSR encodings
        match value & 0b1111 {
            0b00001 => Self::AlignmentFaultMMU,
            0b00010 => Self::DebugEvent,
            0b00011 | 0b00110 => Self::AccessFlagFaultMMU,
            0b00100 => Self::InstructionCacheMaintenanceFault,
            0b00101 | 0b00111 => Self::TranslationFaultMMU,
            0b01000 => Self::SynchronousExternalAbort,
            0b01001 | 0b01011 => Self::DomainFaultMMU,
            0b01100 | 0b01110 => Self::TranslationTableWalkSynchronousExternalAbort,
            0b01101 | 0b01111 => Self::PermissionFaultMMU,
            0x10000 => Self::TLBConflictAbort,
            0b10100 => Self::ImplementationDefinedLockdown,
            0b10110 => Self::AsynchronousExternalAbort,
            0b11000 => Self::MemoryAccessAsynchronousParityError,
            0b11001 => Self::MemoryAccessSynchronousParityError,
            0b11010 => Self::ImplementationDefinedCoprocessorAbort,
            0b11100 | 0b11110 => Self::TranslationTableWalkSynchronousParityError,
            _ => Self::Unknown,
        }
    }
}

impl Display for FaultDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::Unknown => "<unknown>",
            Self::AlignmentFaultMMU => "Alignment fault	(MMU)",
            Self::InstructionCacheMaintenanceFault => "Instruction cache maintenance fault",
            Self::TranslationTableWalkSynchronousExternalAbort => {
                "Translation table walk synchronous external abort"
            }
            Self::TranslationTableWalkSynchronousParityError => {
                "Translation table walk synchronous parity error"
            }
            Self::TranslationFaultMMU => "Translation fault (MMU)",
            Self::AccessFlagFaultMMU => "Access Flag fault (MMU)",
            Self::DomainFaultMMU => "Domain fault (MMU)",
            Self::PermissionFaultMMU => "Permission fault (MMU)",
            Self::DebugEvent => "Debug event",
            Self::SynchronousExternalAbort => "Synchronous external abort",
            Self::ImplementationDefinedLockdown => "implementation defined (Lockdown)",
            Self::ImplementationDefinedCoprocessorAbort => {
                "implementation defined (Coprocessor abort)"
            }
            Self::MemoryAccessSynchronousParityError => "Memory access synchronous parity error",
            Self::AsynchronousExternalAbort => "Asynchronous external abort",
            Self::MemoryAccessAsynchronousParityError => {
                "Memory access asynchronous parity error (Including on translation table walk)"
            }
            Self::TLBConflictAbort => "TLB conflict abort",
        })
    }
}
