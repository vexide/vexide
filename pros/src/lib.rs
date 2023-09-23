#![feature(error_in_core)]
#![cfg_attr(not(target_arch = "wasm32"), no_std)]

extern crate alloc;

pub mod controller;
pub mod error;
pub mod motor;
pub mod multitasking;
pub mod pid;
pub mod position;
pub mod sensors;

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

pub type Result = core::result::Result<(), alloc::boxed::Box<dyn core::error::Error>>;

pub trait Robot {
    fn opcontrol() -> Result {
        Ok(())
    }
    fn auto() -> Result {
        Ok(())
    }
    fn init() -> Result {
        Ok(())
    }
    fn disabled() -> Result {
        Ok(())
    }
    fn comp_init() -> Result {
        Ok(())
    }
}

#[macro_export]
macro_rules! robot {
    ($rbt:ty) => {
        #[no_mangle]
        extern "C" fn opcontrol() {
            <$rbt as $crate::Robot>::opcontrol().unwrap();
        }

        #[no_mangle]
        extern "C" fn autonomous() {
            <$rbt as $crate::Robot>::auto().unwrap();
        }

        #[no_mangle]
        extern "C" fn initialize() {
            <$rbt as $crate::Robot>::init().unwrap();
        }

        #[no_mangle]
        extern "C" fn disabled() {
            <$rbt as $crate::Robot>::disabled().unwrap();
        }

        #[no_mangle]
        extern "C" fn competition_initialize() {
            <$rbt as $crate::Robot>::comp_init().unwrap();
        }
    };
}

pub mod prelude {
    pub use crate::robot;
    pub use crate::Robot;

    pub use crate::controller::{Controller, ControllerId};
    pub use crate::motor::{BrakeMode, Motor};
    pub use crate::sensors;
    pub use crate::{print, println};
}
