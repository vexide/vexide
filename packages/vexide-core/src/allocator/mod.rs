//! Simple allocator using the VEX libc allocation functions in vexos and jemalloc in the sim.

#[cfg(target_os = "vexos")]
mod vexos;
#[cfg(target_arch = "wasm32")]
mod wasm;
