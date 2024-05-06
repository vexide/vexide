//! Provides implementations for the `critical_section` crate on the V5 brain and in WASM environments.

#[cfg(all(target_arch = "arm", target_os = "none"))]
mod zynq;

#[cfg(target_arch = "wasm32")]
mod noop;
