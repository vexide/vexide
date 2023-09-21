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
pub mod gps;
pub mod gps;
pub mod imu;
pub mod imu;
pub mod link;
pub mod llemu;
pub mod misc;
pub mod motor;
pub mod rotation;
pub mod rtos;
pub mod serial;
pub mod vision;

pub use adi::*;
pub use colors::*;
pub use distance::*;
pub use error::*;
pub use ext_adi::*;
pub use gps::*;
pub use gps::*;
pub use imu::*;
pub use imu::*;
pub use link::*;
pub use llemu::*;
pub use misc::*;
pub use motor::*;
pub use rotation::*;
pub use rtos::*;
pub use serial::*;
pub use vision::*;

pub const CLOCKS_PER_SEC: u32 = 1000;

extern "C" {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn memalign(alignment: usize, size: usize) -> *mut core::ffi::c_void;
    pub fn free(ptr: *mut core::ffi::c_void);
    pub fn __errno() -> *mut i32;
    pub fn clock() -> i32;
}
