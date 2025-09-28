use std::fmt::{self, Display};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Fault {
    pub link_register: u32,
    pub stack_pointer: u32,
    pub exception: FaultException,
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
    pub program_counter: u32,
    /// Registers r0 through r12
    pub registers: [u32; 13],
}

impl Fault {
    pub fn load_status(&self) -> FaultStatus {
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

    pub fn address(&self) -> u32 {
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

/// Type of exception causing a fault to be raised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FaultException {
    UndefinedInstruction = 0,
    PrefetchAbort = 1,
    DataAbort = 2,
}

impl Display for FaultException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            FaultException::DataAbort => "Memory Permission",
            FaultException::UndefinedInstruction => "Undefined Instruction",
            FaultException::PrefetchAbort => "Prefetch Abort",
        };

        write!(f, "{name}")
    }
}

pub enum FaultStatus {
    DataFault(u32),
    InstructionFault(u32),
}

impl FaultStatus {
    pub fn source(&self) -> u8 {
        let fsr = match self {
            FaultStatus::DataFault(dfsr) => dfsr,
            FaultStatus::InstructionFault(ifsr) => ifsr,
        };

        (fsr & 0b1111) as u8
    }

    pub fn source_description(&self) -> &'static str {
        // https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/Virtual-Memory-System-Architecture--VMSA-/Fault-Status-and-Fault-Address-registers-in-a-VMSA-implementation/Fault-Status-Register-encodings-for-the-VMSA?lang=en#CBHHADIB

        let source = self.source();

        match self {
            FaultStatus::DataFault(_) => match source {
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
                0b11000 => {
                    "Memory access asynchronous parity error (Including on translation table walk)"
                }
                _ => "unknown",
            },
            FaultStatus::InstructionFault(_) => match source {
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
            },
        }
    }

    pub fn is_write(&self) -> bool {
        match self {
            FaultStatus::DataFault(dfsr) => {
                const WRITE_NOT_READ_BIT: u32 = 1 << 11;

                dfsr & WRITE_NOT_READ_BIT != 0
            }
            FaultStatus::InstructionFault(_ifsr) => false,
        }
    }

    pub fn action_description(&self) -> (&'static str, &'static str) {
        match self {
            FaultStatus::DataFault(_) => {
                if self.is_write() {
                    ("writing", "to")
                } else {
                    ("reading", "from")
                }
            }
            FaultStatus::InstructionFault(_) => ("fetching", "instruction at"),
        }
    }
}
