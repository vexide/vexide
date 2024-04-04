//! Critical section implementation for the V5 Brain

use core::arch::asm;

struct ZynqCriticalSection;
critical_section::set_impl!(ZynqCriticalSection);

unsafe impl critical_section::Impl for ZynqCriticalSection {
    unsafe fn acquire() -> critical_section::RawRestoreState {
        let state: u32;
        unsafe {
            asm!("
                    mrs {0}, cpsr
                    cpsid i
                ",
                out(reg) state
            )
        }
        (state & (1 << 7)) == 0
    }

    unsafe fn release(restore_state: critical_section::RawRestoreState) {
        let mask: u32 = if restore_state { 1 << 7 } else { 0 };
        unsafe {
            asm!("
                mrs r1, cpsr
                bic r1, r1, {0}
                msr cpsr_c, r1
            ",
                in(reg) mask
            )
        }
    }
}
