//! A protected section that cannot be entered by more than one "thread" of execution at a time.
//!
//! Provides implementations for the `critical_section` crate on various platforms.

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod zynq;

#[cfg(target_arch = "wasm32")]
mod noop;
