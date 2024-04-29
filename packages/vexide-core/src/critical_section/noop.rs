//! Critical section implementation for WebAssembly.

struct NoopCriticalSection;
critical_section::set_impl!(NoopCriticalSection);

unsafe impl critical_section::Impl for NoopCriticalSection {
    unsafe fn acquire() -> critical_section::RawRestoreState {
        false
    }

    unsafe fn release(_restore_state: critical_section::RawRestoreState) {}
}
