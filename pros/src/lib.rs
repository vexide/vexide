#![feature(error_in_core, stdsimd, negative_impls)]
#![cfg_attr(not(target_arch = "wasm32"), no_std)]

extern crate alloc;

pub mod async_runtime;
pub mod controller;
pub mod error;
pub mod motor;
pub mod pid;
pub mod position;
pub mod sensors;
pub mod sync;
#[macro_use]
pub mod task;

#[doc(hidden)]
pub use pros_sys as __pros_sys;

#[cfg(target_os = "vexos")]
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

pub use async_trait::async_trait;

pub type Result<T = ()> = core::result::Result<T, alloc::boxed::Box<dyn core::error::Error>>;

use alloc::boxed::Box;
#[async_trait::async_trait]
pub trait Robot {
    async fn opcontrol(&mut self) -> Result {
        Ok(())
    }
    async fn auto(&mut self) -> Result {
        Ok(())
    }
    async fn disabled(&mut self) -> Result {
        Ok(())
    }
    async fn comp_init(&mut self) -> Result {
        Ok(())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __gen_exports {
    ($rbt:ty) => {
        pub static mut ROBOT: Option<$rbt> = None;

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn opcontrol() {
            <$rbt as $crate::Robot>::opcontrol(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before opcontrol")
            })
            .block_on()
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn autonomous() {
            <$rbt as $crate::Robot>::auto(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before auto")
            })
            .block_on()
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn disabled() {
            <$rbt as $crate::Robot>::disabled(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before disabled")
            })
            .block_on()
            .unwrap();
        }

        #[doc(hidden)]
        #[no_mangle]
        extern "C" fn competition_initialize() {
            <$rbt as $crate::Robot>::comp_init(unsafe {
                ROBOT
                    .as_mut()
                    .expect("Expected initialize to run before comp_init")
            })
            .block_on()
            .unwrap();
        }
    };
}

/// Allows your robot code to be executed by the pros kernel.
/// If your robot struct implements Default then you can just supply this macro with its type.
/// If not, you can supply an expression that returns your robot type to initialize your robot struct.
///
/// Example of using the macro with a struct that implements Default:
/// ```rust
/// use pros::prelude::*;
/// #[derive(Default)]
/// struct ExampleRobot;
/// impl Robot for ExampleRobot {
///    fn opcontrol(&mut self) -> Result {
///       println!("Hello, world!");
///      Ok(())
///   }
/// }
/// robot!(ExampleRobot);
/// ```
///
/// Example of using the macro with a struct that does not implement Default:
/// ```rust
/// use pros::prelude::*;
/// struct ExampleRobot {
///    x: i32,
/// }
/// impl Robot for ExampleRobot {
///     fn opcontrol(&mut self) -> Result {
///         println!("Hello, world! {}", self.x);
///         Ok(())
///     }
/// }
/// impl ExampleRobot {
///     pub fn new() -> Self {
///        Self { x: 5 }
///    }
/// }
/// robot!(ExampleRobot, ExampleRobot::new());
#[macro_export]
macro_rules! robot {
    ($rbt:ty) => {
        $crate::__gen_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            ::pros::task::__init_main();
            unsafe {
                ROBOT = Some(Default::default());
            }
        }
    };
    ($rbt:ty, $init:expr) => {
        $crate::__gen_exports!($rbt);

        #[no_mangle]
        extern "C" fn initialize() {
            ::pros::task::__init_main();
            unsafe {
                ROBOT = Some($init);
            }
        }
    };
}

pub mod prelude {
    pub use crate::robot;
    pub use crate::Robot;

    // Import Box from alloc so that it can be used in async_trait!
    pub use crate::{async_trait, print, println, task_local};
    pub use alloc::boxed::Box;

    pub use crate::async_runtime::*;
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
