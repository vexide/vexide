use std::arch::asm;

use crate::abort_handler::fault::Fault;

pub mod bkpt;

#[allow(clippy::inline_always)]
#[inline(always)]
pub fn breakpoint() {
    unsafe {
        asm!("bkpt");
    }
}

trait Debugger: Sync {
    /// Initializes the debugger.
    fn initialize(&'static self);
    fn poll(&'static self);
    fn handle_breakpoint(&'static self, fault: &Fault<'_>);
}
