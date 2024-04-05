//! Simple allocator using the VEX libc allocation functions in vexos and jemalloc in the sim.

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod vexos;
#[cfg(target_arch = "wasm32")]
mod wasm;
