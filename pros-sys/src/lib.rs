#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod adi;
pub mod colors;
pub mod distance;
pub mod error;
pub mod ext_adi;
pub mod llemu;
pub mod misc;
pub mod rtos;
pub mod motor;

pub use adi::*;
pub use colors::*;
pub use distance::*;
pub use error::*;
pub use ext_adi::*;
pub use llemu::*;
pub use misc::*;
pub use rtos::*;
pub use motor::*;

pub const CLOCKS_PER_SEC: u32 = 1000;

extern "C" {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn memalign(alignment: usize, size: usize) -> *mut core::ffi::c_void;
    pub fn free(ptr: *mut core::ffi::c_void);
    pub fn __errno() -> *mut i32;
    pub fn clock() -> i32;
}
