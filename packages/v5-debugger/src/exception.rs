use std::sync::atomic::{AtomicBool, Ordering};

use crate::{DEBUGGER, instruction::Instruction};

core::arch::global_asm!(
    r#"
.text
.arm
.align 5
.global debugger_vector_table

debugger_vector_table:
    ldr pc, original_vector_addresses+0 @ reset
    ldr pc, original_vector_addresses+4
    ldr pc, original_vector_addresses+8
    b prefetch_abort_proxy
    ldr pc, original_vector_addresses+16
    nop
    ldr pc, original_vector_addresses+24
    ldr pc, original_vector_addresses+28

original_vector_addresses: .word 0, 0, 0, 0, 0, 0, 0, 0

prefetch_abort_proxy:
    dsb @ Workaround for Cortex-A9 erratum (id 775420)

    @ check if the exception is a debug event
    push {{r0}}
    mrc p15, 0, r0, c5, c0, 1 @ r0 <- IFSR
    and r0, #0b1111
    cmp r0, #0b00010
    pop {{r0}}

    ldrne pc, original_vector_addresses+12 @ not a debug event, use the original vector



    @ Apply an offset to link register so that it points at address of return
    @ instruction (see Abort::program_counter docs).
    sub lr, #4

    @ -- Save --
    @ Create a DebugEventContext struct on the stack.
    @ 1. Save general purpose registers for debugging and exception return
    push {{r0-r12, lr}}

    @ 2. Add secondary details: exception type and other debug info
    mov r0, 2
    push {{r0}}

    @ Store original SYS-mode stack pointer and link register for debug
    stmdb sp, {{sp,lr}}^
    @ Adjust our sp, since we can't use writeback on stmdb SYS
    sub sp, sp, #8

    @ 3. Save the caller's prog. status reg. in case a recursive exception happens.
    @ Note that this also helps align the stack to 8.
    mrs r0, spsr
    push {{r0}}

    @ -- Handle exception --
    @ Pass it to our handler using the C ABI, fn(*mut DebugEventContext) -> ():
    mov r0, sp                    @ set param 0
    blx handle_debug_event        @ Actually call the function now

    @ -- Restore --
    @ Restore spsr in case it was changed.
    pop {{r0}}
    msr spsr, r0

    @ Our exception handler wouldn't have modified anything in SYS-mode,
    @ so we can just discard that stuff. Also discard the exception type.
    add sp, sp, #12

    @ And... perform an exception return by loading the old LR into PC.
    @ Note that we've already offset LR as required to point to the correct address.
    ldm sp!, {{r0-r12, pc}}^
    "#,
);

#[unsafe(no_mangle)]
#[instruction_set(arm::a32)]
pub unsafe extern "aapcs" fn handle_debug_event(ctx: *mut DebugEventContext) {
    unsafe {
        core::arch::asm!("cpsie i"); // unmask IRQs
        DEBUGGER
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .handle_debug_event(&mut *ctx);
    }
}

#[must_use]
fn vbar() -> u32 {
    let vbar: u32;
    unsafe {
        core::arch::asm!(
            "mrc p15, 0, {0}, c12, c0, 0",
            out(reg) vbar,
            options(nostack, preserves_flags)
        );
    }
    vbar
}

static ORIGINAL_VECTOR_ADDRESSES_SET: AtomicBool = AtomicBool::new(false);

pub fn install_vectors() {
    unsafe extern "C" {
        static mut original_vector_addresses: [u32; 8];
    }

    if !ORIGINAL_VECTOR_ADDRESSES_SET.swap(true, Ordering::Relaxed) {
        let old_vbar = vbar();

        unsafe {
            core::arch::asm!("cpsid if", options(nostack));

            #[allow(clippy::needless_range_loop)]
            for i in 0..8 {
                original_vector_addresses[i] = (old_vbar as *mut u32).add(i) as _;
            }

            core::arch::asm!("cpsie if", options(nostack));
        }
    }

    unsafe {
        core::arch::asm!(
            "movw r0, #:lower16:debugger_vector_table",
            "movt r0, #:upper16:debugger_vector_table",
            "mcr p15, 0, r0, c12, c0, 0",
            out("r0") _,
            options(nostack, preserves_flags)
        );
    }
}

/// The saved state of a program from before an exception.
///
/// Note that updating these fields will cause the exception handler to apply the changes to the CPU
/// if/when the current exception handler returns.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct DebugEventContext {
    /// The saved program status register (spsr) from before the exception.
    pub spsr: ProgramStatus,
    /// The stack pointer from before the exception.
    pub stack_pointer: usize,
    /// The link register from before the exception.
    pub link_register: usize,

    pub _pad: u32,

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

impl DebugEventContext {
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

    /// Load the address or instruction which the faulting instruction attempted to operate on.
    ///
    /// # Safety
    ///
    /// This function accesses CPU state that's set post-exception. The caller must ensure that this
    /// state has not been invalidated.
    #[must_use]
    pub unsafe fn target(&self) -> usize {
        let target: usize;

        unsafe {
            core::arch::asm!(
                "mrc p15, 0, {ifar}, c6, c0, 1",
                ifar = out(reg) target,
                options(nomem, nostack, preserves_flags)
            );
        }

        target
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
