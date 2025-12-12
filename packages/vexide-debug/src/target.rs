use std::fmt::Display;

use gdbstub::{
    arch::Arch,
    common::Signal,
    target::{
        Target, TargetError, TargetResult,
        ext::base::{
            BaseOps,
            single_register_access::SingleRegisterAccessOps,
            singlethread::{SingleThreadBase, SingleThreadResume, SingleThreadResumeOps},
        },
    },
};
use gdbstub_arch::arm::reg::ArmCoreRegs;
use snafu::Snafu;
use vexide_startup::abort_handler::fault::{ExceptionContext, ProgramStatus};

use crate::arch::ARMv7;

#[derive(Debug, Snafu)]
pub enum VexideTargetError {}

#[derive(Debug)]
pub struct VexideTarget {
    pub exception_ctx: ExceptionContext,
    pub resume: bool,
}

impl VexideTarget {
    pub const fn new(exception_ctx: ExceptionContext) -> Self {
        Self {
            exception_ctx,
            resume: false,
        }
    }
}

impl Target for VexideTarget {
    type Arch = ARMv7;
    type Error = VexideTargetError;

    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }
}

impl SingleThreadBase for VexideTarget {
    fn read_registers(&mut self, regs: &mut <ARMv7 as Arch>::Registers) -> TargetResult<(), Self> {
        *regs = ArmCoreRegs {
            r: self.exception_ctx.registers,
            cpsr: self.exception_ctx.spsr.0,
            lr: self.exception_ctx.link_register as u32,
            pc: self.exception_ctx.program_counter as u32,
            sp: self.exception_ctx.stack_pointer as u32,
        };
        Ok(())
    }

    fn write_registers(&mut self, regs: &<ARMv7 as Arch>::Registers) -> TargetResult<(), Self> {
        self.exception_ctx = ExceptionContext {
            registers: regs.r,
            spsr: ProgramStatus(regs.cpsr),
            link_register: regs.lr as usize,
            program_counter: regs.pc as usize,
            stack_pointer: regs.sp as usize,
            ..self.exception_ctx
        };
        Ok(())
    }

    fn read_addrs(&mut self, _start_addr: u32, _data: &mut [u8]) -> TargetResult<usize, Self> {
        Err(TargetError::NonFatal)
    }

    fn write_addrs(&mut self, _start_addr: u32, _data: &[u8]) -> TargetResult<(), Self> {
        Err(TargetError::NonFatal)
    }

    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }

    fn support_single_register_access(&mut self) -> Option<SingleRegisterAccessOps<'_, (), Self>> {
        None
    }
}

impl SingleThreadResume for VexideTarget {
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        self.resume = true;
        Ok(())
    }
}

impl Display for VexideTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[VexideTarget]")
    }
}
