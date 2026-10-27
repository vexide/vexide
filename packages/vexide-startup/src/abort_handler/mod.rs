use core::fmt::Display;

use arbitrary_int::prelude::*;
use vex_sdk::{V5_TouchEvent, V5_TouchStatus, vexTasksRun, vexTouchDataGet};

mod report;

/// Handle a fault.
///
/// # Safety
///
/// `fault` must be valid for reads and writes for the duration of the function call.
pub unsafe extern "C" fn fault_exception_handler(fault: *mut Fault) -> ! {
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

    // SAFETY: Caller guarantees fault is valid for reads and writes.
    let fault = unsafe { fault.as_mut_unchecked() };

    report::serialize_fault(fault);
    report::report_fault(fault);

    let mut prev_touch_event = V5_TouchEvent::kTouchEventRelease;
    loop {
        let mut status = V5_TouchStatus::default();
        unsafe {
            vexTouchDataGet(&raw mut status);
        }

        if status.lastEvent == V5_TouchEvent::kTouchEventRelease
            && prev_touch_event != V5_TouchEvent::kTouchEventRelease
        {
            report::report_fault(fault);
        }

        prev_touch_event = status.lastEvent;

        unsafe {
            vexTasksRun();
        }
    }
}

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Fault {
    pub link_register: u32,
    pub stack_pointer: u32,
    pub exception: FaultException2,
    /// The current program status register at the time of the fault.
    pub cpsr: ProgramStatus,
    /// The address at which the abort occurred.
    ///
    /// This is calculated using the Link Register (`lr`), which is set to this address plus an
    /// offset when an exception occurs.
    ///
    /// Offsets:
    ///
    /// * [plus 8 bytes][da-exception] for data aborts.
    /// * [plus 4 bytes][pf-exception] for prefetch aborts.
    /// * [plus the minimum size of an instruction][svc-exception] for SVCs and undefined
    ///   instruction aborts (this is different in thumb mode).
    ///
    /// [da-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Data-Abort-exception
    /// [pf-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Prefetch-Abort-exception
    /// [svc-exception]: https://developer.arm.com/documentation/ddi0406/b/System-Level-Architecture/The-System-Level-Programmers--Model/Exceptions/Supervisor-Call--SVC--exception
    pub program_counter: u32,
    /// Registers r0 through r12
    pub registers: [u32; 13],
}

impl Fault {
    /// Read the faulting instruction.
    ///
    /// # Safety
    ///
    /// The PC field must point to an instruction that's valid for a volatile read.
    unsafe fn faulting_instruction(&self) -> Instruction {
        if self.cpsr.thumb() {
            let ptr = self.program_counter as *const u16;
            unsafe {
                let first = ptr.read_volatile();

                // See the Armv7-A reference, A6.1 Thumb instruction set encoding
                let is_32bit = (first >> 11) > 0b11100;
                if is_32bit {
                    let second = ptr.add(1).read_volatile();
                    Instruction::Thumb32([first, second])
                } else {
                    Instruction::Thumb16(first)
                }
            }
        } else {
            let ptr = self.program_counter as *const u32;
            Instruction::Arm32(unsafe { ptr.read_volatile() })
        }
    }
}

/// Type of exception causing a fault to be raised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum _FaultException {
    UndefinedInstruction = 0,
    PrefetchAbort = 1,
    DataAbort = 2,
}

/// Type of exception causing a fault to be raised.
#[derive(Debug, Clone, Copy, PartialEq, Eq, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(transparent)]
pub struct FaultException2(pub u32);

impl FaultException2 {
    pub const UNDEFINED_INSTRUCTION: Self = Self(0);
    pub const PREFETCH_ABORT: Self = Self(1);
    pub const DATA_ABORT: Self = Self(2);
}

impl Display for FaultException2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            Self::DATA_ABORT => "Data Abort",
            Self::UNDEFINED_INSTRUCTION => "Undefined Instruction",
            Self::PREFETCH_ABORT => "Prefetch Abort",
            _ => "[invalid value]",
        })
    }
}

impl Display for _FaultException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::DataAbort => "Data Abort",
            Self::UndefinedInstruction => "Undefined Instruction",
            Self::PrefetchAbort => "Prefetch Abort",
        })
    }
}

impl Fault {
    pub fn address(&self) -> u32 {
        let address: u32;

        match self.exception {
            FaultException2::DATA_ABORT => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {dfar}, c6, c0, 0",
                    dfar = out(reg) address,
                );

                address
            },
            FaultException2::PREFETCH_ABORT => unsafe {
                core::arch::asm!(
                    "mrc p15, 0, {ifar}, c6, c0, 1",
                    ifar = out(reg) address,
                );

                address
            },
            FaultException2::UNDEFINED_INSTRUCTION => self.program_counter,
            // TODO: handle better?
            _ => loop {}
        }
    }
    pub fn fault_data(&self) -> FaultException {
        let addr = self.address();
        match self.exception {
            FaultException2::DATA_ABORT => {
                let dfsr: u32;
                unsafe {
                    core::arch::asm!(
                        "mrc p15, 0, {dfsr}, c5, c0, 0",
                        dfsr = out(reg) dfsr,
                    );
                };
                FaultException::DataAbort { addr, dfsr: FaultStatus::new_with_raw_value(dfsr) }
            }
            FaultException2::PREFETCH_ABORT => {
                let ifsr: u32;
                unsafe {
                    core::arch::asm!(
                        "mrc p15, 0, {ifsr}, c5, c0, 1",
                        ifsr = out(reg) ifsr,
                    );
                }
                FaultException::PrefetchAbort { addr, ifsr: FaultStatus::new_with_raw_value(ifsr) }
            }
            FaultException2::UNDEFINED_INSTRUCTION => {
                FaultException::UndefinedInstruction {
                    instr: unsafe {
                        // SAFETY: Since the abort was a undefined instruction, not a prefetch
                        // abort, it implies that reading the instruction
                        // won't cause another fault.
                        self.faulting_instruction()
                    },
                }
            },
            // TODO: handle better?
            _ => loop {}
        }
    }
}

pub enum FaultException {
    DataAbort { addr: u32, dfsr: FaultStatus },
    PrefetchAbort { addr: u32, ifsr: FaultStatus },
    UndefinedInstruction { instr: Instruction },
}

impl Into<u128> for FaultException {
    fn into(self) -> u128 {
        match self {
            Self::DataAbort { addr, dfsr } => {
                let mut bytes = [0u8; 16];
                bytes[0..4].copy_from_slice(&addr.to_le_bytes());
                bytes[4..8].copy_from_slice(&dfsr.raw_value().to_le_bytes());
                bytes[15] = 3;
                u128::from_le_bytes(bytes)
            },
            Self::PrefetchAbort { addr, ifsr } => {
                let mut bytes = [0u8; 16];
                bytes[0..4].copy_from_slice(&addr.to_le_bytes());
                bytes[4..8].copy_from_slice(&ifsr.raw_value().to_le_bytes());
                bytes[15] = 2;
                u128::from_le_bytes(bytes)
            },
            Self::UndefinedInstruction { instr } => {
                let mut bytes = [0u8; 16];
                match instr {
                    Instruction::Arm32(val) => {
                        bytes[0..4].copy_from_slice(&val.to_le_bytes());
                        bytes[5] = 0;
                    },
                    Instruction::Thumb16(val) => {
                        bytes[0..2].copy_from_slice(&val.to_le_bytes());
                        bytes[5] = 1;
                    },
                    Instruction::Thumb32([val_1, val_2]) => {
                        bytes[0..2].copy_from_slice(&val_1.to_le_bytes());
                        bytes[2..4].copy_from_slice(&val_2.to_le_bytes());
                        bytes[5] = 2;
                    }
                }
                bytes[15] = 1;
                u128::from_le_bytes(bytes)
            }
        }
    }
}

impl Display for FaultException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            FaultException::DataAbort { addr, dfsr } => {
                f.write_str(match dfsr.reason().value() {
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
                    _ => "<unknown>",
                })?;
                f.write_str(" while ")?;

                f.write_str(if dfsr.write_not_read() {
                    "writing to "
                } else {
                    "reading from "
                })?;

                write!(f, "0x{addr:x}")?;
            }
            FaultException::PrefetchAbort { addr, ifsr } => {
                f.write_str(match ifsr.reason().value() {
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
                    _ => "<unknown>",
                })?;

                write!(f, " while fetching address 0x{addr:x}")?;
            }
            FaultException::UndefinedInstruction { instr } => {
                // SAFETY: Since the abort was a undefined instruction, not a prefetch abort, it
                // implies that reading the instruction won't cause another fault.
                write!(f, "Failed to decode instruction {instr:x?}")?;
            }
        }

        Ok(())
    }
}

impl Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fault_data().fmt(f)
    }
}

#[bitbybit::bitfield(u32)]
pub struct ProgramStatus {
    /// Indicates that the program was running in Thumb mode.
    #[bit(5, rw)]
    thumb: bool,
}

// Safety: ProgramStatus is a repr(C) struct with a single u32 member
unsafe impl bytemuck::Zeroable for ProgramStatus {}
// Safety: ProgramStatus is a repr(C) struct with a single u32 member
unsafe impl bytemuck::Pod for ProgramStatus {}

#[bitbybit::bitfield(u32)]
pub struct FaultStatus {
    /// Indicates that the fault was caused by a write operation.
    #[bit(11, r)]
    write_not_read: bool,
    /// Indicates the encoded reason of the fault.
    #[bits([0..=3, 10], r)]
    reason: u5,
}

/// Holds either an ARM or Thumb instruction.
///
/// All ARM instructions are 32-bit, but as Thumb is a variable-length instruction set,
/// some Thumb instructions are only 16-bit.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub enum Instruction {
    Arm32(u32),
    Thumb32([u16; 2]),
    Thumb16(u16),
}
