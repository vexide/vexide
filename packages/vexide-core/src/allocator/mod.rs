//! Simple allocator using the Talc on the Brain and jemalloc in the sim.

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod vexos;
#[cfg(target_arch = "wasm32")]
mod wasm;
