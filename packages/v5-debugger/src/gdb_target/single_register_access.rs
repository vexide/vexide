use gdbstub::target::{
    TargetError, TargetResult, ext::base::single_register_access::SingleRegisterAccess,
};
use gdbstub_arch::arm::reg::id::ArmCoreRegId;

use crate::{exception::ProgramStatus, gdb_target::V5Target};

impl SingleRegisterAccess<()> for V5Target {
    fn read_register(
        &mut self,
        _tid: (),
        reg_id: ArmCoreRegId,
        buf: &mut [u8],
    ) -> TargetResult<usize, Self> {
        if let Some(ctx) = &mut self.exception_ctx {
            let reg = match reg_id {
                ArmCoreRegId::Gpr(rid) => ctx.registers.get(rid as usize).copied(),
                ArmCoreRegId::Sp => Some(ctx.stack_pointer as u32),
                ArmCoreRegId::Lr => Some(ctx.link_register as u32),
                ArmCoreRegId::Pc => Some(ctx.program_counter as u32),
                ArmCoreRegId::Cpsr => Some(ctx.spsr.0),
                _ => None,
            };

            if let Some(reg) = reg {
                let bytes = reg.to_ne_bytes();
                buf.copy_from_slice(&bytes);
                Ok(bytes.len())
            } else {
                Ok(0)
            }
        } else {
            Err(TargetError::NonFatal)
        }
    }

    fn write_register(
        &mut self,
        _tid: (),
        reg_id: ArmCoreRegId,
        val: &[u8],
    ) -> TargetResult<(), Self> {
        if let Some(ctx) = &mut self.exception_ctx
            && let Ok(bytes) = val.try_into()
        {
            let val = u32::from_ne_bytes(bytes);

            match reg_id {
                ArmCoreRegId::Gpr(rid) => {
                    let Some(storage) = ctx.registers.get_mut(rid as usize) else {
                        return Err(TargetError::NonFatal);
                    };

                    *storage = val;
                }
                ArmCoreRegId::Sp => ctx.stack_pointer = val as usize,
                ArmCoreRegId::Lr => ctx.link_register = val as usize,
                ArmCoreRegId::Pc => ctx.program_counter = val as usize,
                ArmCoreRegId::Cpsr => ctx.spsr = ProgramStatus(val),
                _ => return Err(TargetError::NonFatal),
            }

            Ok(())
        } else {
            Err(TargetError::NonFatal)
        }
    }
}
