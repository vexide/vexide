use core::arch::{asm, global_asm, naked_asm};

use crate::abort_handler::FaultException;

global_asm!(
    include_str!("vectors.s"),
    undefined_instruction = sym vexide_undefined_instruction,
    prefetch_abort = sym vexide_prefetch_abort,
    data_abort = sym vexide_data_abort,
);

/// Enable vexide's CPU exception handling logic by installing its custom vector table.
pub fn install_vector_table() {
    unsafe {
        asm!(
            // If vector_table is placed too far away, we'll need 2 instructions to load it.
            "movw r0, #:lower16:vector_table",
            "movt r0, #:upper16:vector_table",

            // Set VBAR; see <https://developer.arm.com/documentation/ddi0601/2025-06/AArch32-Registers/VBAR--Vector-Base-Address-Register>
            "mcr p15, 0, r0, c12, c0, 0",
            out("r0") _,
            options(nostack, preserves_flags)
        );
    }
}

/// Indicates that the fault handler should offset LR by 4 on ARM and 2 on Thumb.
const LR_OFFSET_INSTR_SIZE: isize = -1;

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
                include_str!("fault.s"),
                LR_OFFSET_INSTR_SIZE = const LR_OFFSET_INSTR_SIZE,
                exception_handler = sym crate::abort_handler::fault_exception_handler,
                lr_offset = const $lr_offset,
                exception = const $exception as u32,
            );
        }
    };
}

fault_exception_vector!(
    vexide_undefined_instruction:
        lr_offset = LR_OFFSET_INSTR_SIZE,
        exception = FaultException::UndefinedInstruction,
);
fault_exception_vector!(
    vexide_prefetch_abort:
        lr_offset = 4,
        exception = FaultException::PrefetchAbort,
);
fault_exception_vector!(
    vexide_data_abort:
        lr_offset = 8,
        exception = FaultException::DataAbort,
);
