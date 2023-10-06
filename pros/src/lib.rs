#![feature(error_in_core, stdsimd, negative_impls)]
#![cfg_attr(not(target_arch = "wasm32"), no_std)]

extern crate alloc;

pub mod controller;
pub mod error;
pub mod motor;
pub mod pid;
pub mod position;
pub mod sensors;
pub mod sync;
pub mod task;
pub mod async_runtime;

#[doc(hidden)]
pub use pros_sys as __pros_sys;

#[cfg(not(target_arch = "wasm32"))]
mod vexos_env;
#[cfg(target_arch = "wasm32")]
mod wasm_env;

#[cfg(not(feature = "lvgl"))]
#[macro_use]
pub mod lcd;

#[cfg(feature = "lvgl")]
#[macro_use]
pub mod lvgl;

pub mod adi;
pub mod link;

pub type Result<T = ()> = core::result::Result<T, alloc::boxed::Box<dyn core::error::Error>>;

pub trait Robot {
    fn opcontrol(&mut self) -> Result {
        Ok(())
    }
    fn auto(&mut self) -> Result {
        Ok(())
    }
    fn init() -> Result<Self>
    where
        Self: Sized;
    fn disabled(&mut self) -> Result {
        Ok(())
    }
    fn comp_init(&mut self) -> Result {
        Ok(())
    }
}

pub mod prelude {
    pub use crate::Robot;
    pub use crate::{print, println};
    pub use pros_macros::robot;

    pub use crate::controller::*;
    pub use crate::error::PortError;
    pub use crate::lcd::{buttons::Button, LcdError};
    pub use crate::link::*;
    pub use crate::motor::*;
    pub use crate::pid::*;
    pub use crate::position::*;
    pub use crate::sensors::distance::*;
    pub use crate::sensors::gps::*;
    pub use crate::sensors::rotation::*;
    pub use crate::sensors::vision::*;
    pub use crate::task::{sleep, spawn};
}
