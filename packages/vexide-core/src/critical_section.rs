//! Critical section implementation for the V5 Brain

use core::arch::asm;

struct ZynqCriticalSection;
critical_section::set_impl!(ZynqCriticalSection);

unsafe impl critical_section::Impl for ZynqCriticalSection {
    unsafe fn acquire() -> critical_section::RawRestoreState {
        let mut state: u32;
        unsafe {
            asm!("
                    // Save the current state
                    mrs {0}, cpsr
                    // Disable IRQs
                    cpsid i
                    // Synchronization barriers
                    dsb
                    isb
                ",
                out(reg) state
            )
        }
        (state & 0b1000000) == 0b1000000
    }

    unsafe fn release(restore_state: critical_section::RawRestoreState) {
        // Don't enable IRQs if we are in a nested critical section
        if restore_state {
            unsafe {
                asm!("cpsie i")
            }
        }
    }
}
