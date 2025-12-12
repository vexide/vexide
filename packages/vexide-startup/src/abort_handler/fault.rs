//! APIs for inspecting caught CPU faults.

use std::{
    fmt::{Display, Formatter},
    ptr,
};

#[cfg(all(target_os = "vexos", feature = "backtrace"))]
use vex_libunwind::UnwindContext;
#[cfg(all(target_os = "vexos", feature = "backtrace"))]
use vex_libunwind_sys::unw_context_t;

/// The captured details of a CPU fault.
pub struct Fault<'a> {
    /// The saved CPU state from before the exception.
    ///
    /// After the exception handling process is finished, this state will be restored to the CPU.
    /// Modifying values in this struct may cause changes to the CPU's state post-exception.
    pub ctx: &'a mut ExceptionContext,
    /// The value (pointer or instruction) that was being operated on when the fault occurred.
    ///
    /// For undefined instruction exceptions, this is the encoded instruction that couldn't be
    /// interpreted by the CPU. For other types of exceptions, this is address whose dereference
    /// lead to the abort. This value may be "0" if it is not applicable to the fault (for
    /// instance, breakpoint faults do not have a target).
    pub target: usize,
    /// Additional information about the circumstances of the fault.
    pub status: FaultStatus,
}

impl Fault<'_> {
    /// Load a fault's details from an exception context pointer.
    ///
    /// This function is intended to be called early in the handling of an exception.
    ///
    /// # Safety
    ///
    /// This function accesses CPU state that's captured post-exception. The caller must ensure that
    /// this state has not been invalidated.
    ///
    /// The caller must not invalidate the given [`ExceptionContext`] while the exception is being
    /// handled. The caller must also not assign it a lifetime longer than the exception handling
    /// process.
    pub unsafe fn from_ptr(ptr: *mut ExceptionContext) -> Self {
        let ctx = unsafe { &mut *ptr };

        Self {
            target: unsafe { ctx.target() },
            status: unsafe { ctx.status() },
            ctx,
        }
    }

    /// Returns whether this fault was caused by hitting a breakpoint.
    #[must_use]
    pub fn is_breakpoint(&self) -> bool {
        self.ctx.exception == ExceptionType::PrefetchAbort
            && self.status.details == FaultDetails::DebugEvent
    }
}

impl Display for Fault<'_> {
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

/// The saved state of a program from before an exception.
///
/// Note that updating these fields will cause the exception handler to apply the changes to the CPU
/// if/when the current exception handler returns.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct ExceptionContext {
    /// The saved program status register (spsr) from before the exception.
    pub spsr: ProgramStatus,
    /// The stack pointer from before the exception.
    pub stack_pointer: usize,
    /// The link register from before the exception.
    pub link_register: usize,
    /// The type of the exception that is being handled.
    pub exception: ExceptionType,
    /// Registers r0 through r12
    pub registers: [u32; 13],
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
    pub program_counter: usize,
}

impl ExceptionContext {
    /// Read the ARM instruction which the exception would return to.
    ///
    /// # Safety
    ///
    /// The caller must ensure the return address is valid for reads. This might not be the case if,
    /// for example, the exception was a prefetch abort caused by the instruction being
    /// inaccessible.
    #[must_use]
    pub unsafe fn read_instr(&self) -> Instruction {
        if self.spsr.is_thumb() {
            let ptr = self.program_counter as *mut u16;
            Instruction::Thumb(unsafe { ptr.read_volatile() })
        } else {
            let ptr = self.program_counter as *mut u32;
            Instruction::Arm(unsafe { ptr.read_volatile() })
        }
    }

    /// Create an unwind context using custom registers instead of ones captured from the current
    /// processor state.
    ///
    /// This is based on the ARM implementation of __unw_getcontext:
    /// <https://github.com/llvm/llvm-project/blob/6fc3b40b2cfc33550dd489072c01ffab16535840/libunwind/src/UnwindRegistersSave.S#L834>
    ///
    /// # Safety
    ///
    /// The caller must ensure this saved CPU state is valid for unwinding from.
    #[cfg(all(target_os = "vexos", feature = "backtrace"))]
    #[must_use]
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
                    sp: self.stack_pointer as u32,
                    lr: self.link_register as u32,
                    pc: self.program_counter as u32,
                    // This matches the behavior of __unw_getcontext, which leaves
                    // this data uninitialized.
                    data: [0; _],
                },
            ))
        }
    }
}

/// Type of exception causing a fault to be raised.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum ExceptionType {
    /// An undefined instruction was executed.
    #[default]
    UndefinedInstruction = 0,
    /// A memory abort occurred while attempting to fetch and execute an instruction.
    ///
    /// This type of abort can also be emitted when a breakpoint is triggered.
    PrefetchAbort = 1,
    /// A data memory access failed.
    ///
    /// This type of abort can also be emitted when a watchpoint is triggered.
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
    #[must_use]
    pub unsafe fn target(&self) -> usize {
        let target: usize;

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
                Instruction::read(self.program_counter as *const u32, self.spsr.is_thumb())
                    .as_usize()
            },
        }
    }

    /// Load additional details about the circumstances of the fault.
    ///
    /// # Safety
    ///
    /// This function accesses CPU state that's set post-exception. The caller must ensure that this
    /// state has not been invalidated.
    #[must_use]
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

/// Additional information about the circumstances of a fault.
pub struct FaultStatus {
    /// The reason that the fault was emitted.
    pub details: FaultDetails,
    /// Indicated whether a data abort was caused by a write operation rather than a read
    /// operation.
    pub write_not_read: bool,
}

/// The reasons for which a fault could be emitted.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FaultDetails {
    /// Unknown exception cause
    Unknown,
    /// Alignment fault (MMU exception)
    AlignmentFaultMMU,
    /// A Breakpoint, Watchpoint, or Vector Catch debug event was triggered.
    ///
    /// Vector Catch is a mechanism for placing breakpoints on CPU exceptions. It's mostly useful
    /// alongside a debug probe.
    DebugEvent,
    /// Access flag fault (MMU exception)
    AccessFlagFaultMMU,
    /// Instruction cache maintenance fault
    InstructionCacheMaintenanceFault,
    /// Translation fault (MUU exception)
    TranslationFaultMMU,
    /// Synchronous external abort
    SynchronousExternalAbort,
    /// Domain fault (MMU exception)
    DomainFaultMMU,
    /// Synchronous external abort during translation table walk
    TranslationTableWalkSynchronousExternalAbort,
    /// Permission fault (MMU exception)
    PermissionFaultMMU,
    /// Translation Lookaside Buffer conflict
    TLBConflictAbort,
    /// \<Implementation defined\> (Lockdown)
    ImplementationDefinedLockdown,
    /// Asynchronous external abort
    AsynchronousExternalAbort,
    /// Asynchronous Parity Error during memory access
    MemoryAccessAsynchronousParityError,
    /// Synchronous Parity Error during memory access
    MemoryAccessSynchronousParityError,
    /// \<Implementation defined\> (Coprocessor abort)
    ImplementationDefinedCoprocessorAbort,
    /// Synchronous parity error during translation table walk
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

/// The status of an ARMv7 CPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(transparent)]
pub struct ProgramStatus(pub u32);

impl ProgramStatus {
    /// Returns whether the CPU state has a 16-bit instruction set enabled (Thumb or ThumbEE).
    #[must_use]
    pub const fn is_thumb(self) -> bool {
        const T_BIT: u32 = 1 << 5;
        self.0 & T_BIT != 0
    }
}

/// An instruction-set independent CPU instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// An instruction from the ARM32 instruction set.
    Arm(u32),
    /// An instruction from the Thumb instruction set.
    Thumb(u16),
}

impl Instruction {
    /// Returns whether this is a thumb instruction.
    #[must_use]
    pub const fn is_thumb(self) -> bool {
        matches!(self, Self::Thumb(_))
    }

    /// Returns the size of the instruction in bytes.
    #[must_use]
    pub const fn size(self) -> usize {
        match self {
            Self::Arm(instr) => size_of_val(&instr),
            Self::Thumb(instr) => size_of_val(&instr),
        }
    }

    /// Returns the integer representation of the instruction casted to a usize.
    #[must_use]
    pub const fn as_usize(self) -> usize {
        match self {
            Self::Arm(i) => i as usize,
            Self::Thumb(i) => i as usize,
        }
    }

    /// Reads either a thumb or ARM instruction from the given pointer.
    ///
    /// # Safety
    ///
    /// The address must be valid for reads.
    #[must_use]
    pub unsafe fn read(addr: *const u32, thumb: bool) -> Self {
        debug_assert!(!addr.is_null());
        if thumb {
            Self::Thumb(unsafe { ptr::read_volatile(addr.cast()) })
        } else {
            Self::Arm(unsafe { ptr::read_volatile(addr) })
        }
    }

    /// Writes this instruction to the given pointer.
    ///
    /// # Safety
    ///
    /// The address must be valid for writes. The caller must handle flushing the CPU instruction
    /// cache after calling this method.
    pub unsafe fn write_to(self, addr: *mut u32) {
        debug_assert!(!addr.is_null());
        match self {
            Self::Arm(instr) => unsafe {
                ptr::write_volatile(addr, instr);
            },
            Self::Thumb(instr) => unsafe {
                ptr::write_volatile(addr.cast(), instr);
            },
        }
    }
}
