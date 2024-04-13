#[cfg(all(target_arch = "arm", target_os = "none"))]
mod zynq;

#[cfg(target_arch = "wasm32")]
mod noop;
