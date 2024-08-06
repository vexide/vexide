//! Critical section implementation for the V5 Brain

use core::arch::asm;

struct ZynqCriticalSection;
critical_section::set_impl!(ZynqCriticalSection);

unsafe impl critical_section::Impl for ZynqCriticalSection {
    unsafe fn acquire() -> critical_section::RawRestoreState {
        unsafe {
            let mut cpsr: u32;
            asm!("mrs {0}, cpsr", out(reg) cpsr);
            let masked = (cpsr & 0b10000000) == 0b10000000;

            asm!(
                "
                // Disable IRQs
                cpsid i
                // Synchronization barriers
                dsb
                isb
                "
            );

            masked
        }
    }

    unsafe fn release(masked: critical_section::RawRestoreState) {
        // Don't enable IRQs if we are in a nested critical section
        if !masked {
            unsafe {
                asm!(
                    "
                    // Re-enable IRQs
                    cpsie i
                    // Synchronization barriers
                    dsb
                    isb
                    "
                )
            }
        }
    }
}
