//! Simple allocator using the Talc on the Brain and jemalloc in the sim.

#[cfg(all(target_arch = "arm", target_os = "vexos"))]
pub mod vexos;
#[cfg(target_arch = "wasm32")]
mod wasm;
