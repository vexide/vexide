//! Panic Handler
//!
//! This module contains the internal `#[panic_handler]` used by pros-rs for
//! dealing with unrecoverable program errors.

use alloc::format;

use crate::{devices::screen::Screen, io::eprintln, task};

#[panic_handler]
/// The panic handler for pros-rs.
pub fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let current_task = task::current();

    let task_name = current_task.name().unwrap_or_else(|_| "<unknown>".into());

    // task 'User Initialization (PROS)' panicked at src/lib.rs:22:1:
    // panic message here
    let msg = format!("task '{task_name}' {info}");

    eprintln!("{msg}");

    unsafe {
        Screen::new().draw_error(&msg).unwrap_or_else(|err| {
            eprintln!("Failed to draw error message to screen: {err}");
        });

        #[cfg(target_arch = "wasm32")]
        crate::wasm_env::sim_log_backtrace();

        pros_sys::exit(1);
    }
}
