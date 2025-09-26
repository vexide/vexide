
use std::fmt::Write;

use vex_libunwind::{registers::UNW_REG_IP, UnwindContext, UnwindCursor, UnwindError};
use vex_libunwind_sys::unw_context_t;

use super::report::AbortWriter;

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
