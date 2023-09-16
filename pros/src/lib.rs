#![feature(error_in_core)]
#![no_std]

use core::panic::PanicInfo;

pub mod controller;
pub mod error;
pub mod motor;
pub mod multitasking;
pub mod pid;
pub mod position;
pub mod sensors;

#[cfg(not(target_arch = "wasm32"))]
pub mod memory;
#[cfg(target_arch = "wasm32")]
pub mod wasm_memory;

#[cfg(not(feature = "lvgl"))]
#[macro_use]
pub mod lcd;

#[cfg(feature = "lvgl")]
#[macro_use]
pub mod lvgl;

pub(crate) mod errno;

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    println!("Panicked! {_info}");
    loop {}
}

pub mod prelude {
    pub use crate::controller::{Controller, ControllerId};
    pub use crate::motor::{BrakeMode, Motor};
    pub use crate::sensors;
    pub use crate::{print, println};

    #[cfg(feature = "derive")]
    pub use pros_derive::*;
}
