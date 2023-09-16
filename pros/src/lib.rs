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

pub trait Robot {
    fn opcontrol() {}
    fn auto() {}
    fn init() {}
    fn disabled() {}
    fn comp_init() {}
}

#[macro_export]
macro_rules! robot {
    ($rbt:ty) => {
        #[no_mangle]
        extern "C" fn opcontrol() {
            <$rbt as $crate::Robot>::opcontrol();
        }

        #[no_mangle]
        extern "C" fn autonomous() {
            <$rbt as $crate::Robot>::auto();
        }

        #[no_mangle]
        extern "C" fn initialize() {
            <$rbt as $crate::Robot>::init();
        }

        #[no_mangle]
        extern "C" fn disabled() {
            <$rbt as $crate::Robot>::disabled();
        }

        #[no_mangle]
        extern "C" fn competition_initialize() {
            <$rbt as $crate::Robot>::comp_init();
        }
    };  
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    println!("Panicked! {_info}");
    loop {}
}

pub mod prelude {
    pub use crate::Robot;
    pub use crate::robot;

    pub use crate::controller::{Controller, ControllerId};
    pub use crate::motor::{BrakeMode, Motor};
    pub use crate::sensors;
    pub use crate::{print, println};
}
