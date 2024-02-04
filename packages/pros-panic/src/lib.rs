//! Panic handler implementation for [`pros-rs`](https://crates.io/crates/pros-rs).
//! Supports printing a backtrace when running in the simulator.
//! If the `display_panics` feature is enabled, it will also display the panic message on the V5 Brain display.

#![no_std]
#![warn(
    missing_docs,
    rust_2018_idioms,
    missing_debug_implementations,
    unsafe_op_in_unsafe_fn,
    clippy::missing_const_for_fn
)]
extern crate alloc;

use alloc::format;

use pros_core::eprintln;
#[cfg(feature = "display_panics")]
use pros_devices::Screen;
use pros_sync::task;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Prints a backtrace to the debug console
    fn sim_log_backtrace();
}

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
        #[cfg(feature = "display_panics")]
        Screen::new().draw_error(&msg).unwrap_or_else(|err| {
            eprintln!("Failed to draw error message to screen: {err}");
        });

        #[cfg(target_arch = "wasm32")]
        sim_log_backtrace();

        pros_sys::exit(1);
    }
}
