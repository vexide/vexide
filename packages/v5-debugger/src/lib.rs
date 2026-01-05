#![allow(missing_docs)]

use std::sync::{Mutex, Once, OnceLock};

use crate::{
    exception::{DebugEventContext, install_vectors},
    gdb_target::breakpoint::BreakpointError,
    transport::{StdioTransport, Transport},
};

pub mod cache;
pub mod debugger;
pub mod exception;
pub mod gdb_target;
pub mod instruction;
pub(crate) mod regs;
pub mod transport;

pub static DEBUGGER: OnceLock<Mutex<&mut dyn Debugger>> = OnceLock::new();

pub unsafe trait Debugger: Send {
    /// Initializes the debugger.
    fn initialize(&mut self) {}

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
    unsafe fn register_breakpoint(
        &mut self,
        addr: usize,
        thumb: bool,
    ) -> Result<(), BreakpointError>;

    /// A callback function which is run whenever a breakpoint is triggered.
    ///
    /// The function is given access to the pre-breakpoint CPU state and can view/modify it as
    /// needed.
    ///
    /// # Safety
    ///
    /// The given fault must represent valid, saved CPU state.
    unsafe fn handle_debug_event(&mut self, ctx: &mut DebugEventContext);
}

/// Set the current debugger.
pub fn install(debugger: impl Debugger + 'static) {
    DEBUGGER
        .set(Mutex::new(Box::leak(Box::new(debugger))))
        .map_err(|_| ())
        .expect("A debugger is already installed.");
    install_vectors();
}
