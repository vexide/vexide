use std::{
    arch::asm,
    sync::{Mutex, OnceLock},
};

use snafu::Snafu;

use crate::abort_handler::fault::Fault;

pub static DEBUGGER: OnceLock<Mutex<&mut dyn Debugger>> = OnceLock::new();

#[derive(Debug, Snafu)]
pub enum BreakpointError {
    /// There is already a breakpoint with this address.
    AlreadyExists,
    /// There are no free breakpoint slots.
    NoSpace,
}

/// A struct which can manage breakpoints and program debug state.
///
/// # Safety
///
/// Some handler functions are given access to saved CPU state and can view/modify it as needed.
/// The debugger must not place the CPU into an invalid state.
pub unsafe trait Debugger: Send {
    /// Initializes the debugger.
    fn initialize(&mut self);

    /// Registers a breakpoint at the specified address.
    ///
    /// # Safety
    ///
    /// Breakpoints may only be placed on executable addresses containing instructions.
    ///
    /// The caller must ensure that the instruction on which the breakpoint is being placed
    /// has linear control flow (it is not a jump).
    ///
    /// # Errors
    ///
    /// This function will return an error if there are no more free breakpoint slots or if
    /// the specified address already has a breakpoint on it.
    unsafe fn register_breakpoint(&mut self, addr: usize, thumb: bool) -> Result<(), BreakpointError>;

    /// A callback function which is run whenever a breakpoint is triggered.
    ///
    /// The function is given access to the pre-breakpoint CPU state and can view/modify it as
    /// needed.
    ///
    /// # Safety
    ///
    /// The given fault must represent valid, saved CPU state.
    unsafe fn handle_exception(&mut self, fault: &mut Fault<'_>);
}

/// Set the current debugger.
pub fn install(debugger: impl Debugger + 'static) {
    DEBUGGER
        .set(Mutex::new(Box::leak(Box::new(debugger))))
        .map_err(|_| ())
        .expect("A debugger is already installed.");
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub fn breakpoint() {
    unsafe {
        asm!("bkpt");
    }
}

pub(crate) unsafe fn handle_breakpoint(fault: &mut Fault<'_>) {
    debug_assert!(fault.is_breakpoint());
    if let Some(debugger) = DEBUGGER.get()
        && let Ok(mut debugger) = debugger.try_lock()
    {
        unsafe {
            debugger.handle_exception(fault);
        }
    }
}
