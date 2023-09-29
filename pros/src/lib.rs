#![feature(error_in_core, stdsimd)]
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

#[macro_export]
macro_rules! robot {
    ($rbt:ty) => {
        pub static mut ROBOT: Option<$rbt> = None;

        #[no_mangle]
        extern "C" fn opcontrol() {
            <$rbt as $crate::Robot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn autonomous() {
            <$rbt as $crate::Robot>::auto(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before auto")
            })
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn initialize() {
            unsafe {
                ::pros::__pros_sys::lcd_initialize();
            }
            unsafe {
                ROBOT = Some(<$rbt as $crate::Robot>::init().unwrap());
            }
        }

        #[no_mangle]
        extern "C" fn disabled() {
            <$rbt as $crate::Robot>::disabled(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before disabled")
            })
            .unwrap();
        }

        #[no_mangle]
        extern "C" fn competition_initialize() {
            <$rbt as $crate::Robot>::comp_init(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before comp_init")
            })
            .unwrap();
        }
    };
}

pub mod prelude {
    pub use crate::Robot;
    pub use crate::{print, println, robot};

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
