#![no_std]

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;

pub mod controller;
pub mod error;
pub mod motor;
pub mod multitasking;
pub mod sensors;

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

struct Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        pros_sys::memalign(layout.align(), layout.size()) as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        pros_sys::free(ptr as *mut core::ffi::c_void)
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

/// Code that is run during the autonomous period.
#[macro_export]
macro_rules! autonomous {
    {$fn:block} => {
        #[no_mangle]
        extern "C" fn autonomous() {
            $fn
        }
    };
	() => {
		#[no_mangle]
        extern "C" fn autonomous() {}
	};
}

/// Code that is run after when the robot is initialized.
#[macro_export]
macro_rules! on_initialize {
    {$fn:block} => {
        #[no_mangle]
        extern "C" fn initialize() {
            $fn
        }
    };
	() => {
		#[no_mangle]
        extern "C" fn initialize() {}
	};
}

/// Code that is run when the robot is disabled.
#[macro_export]
macro_rules! on_disable {
    {$fn:block} => {
        #[no_mangle]
        extern "C" fn disabled() {
            $fn
        }
    };
	() => {
		#[no_mangle]
        extern "C" fn disabled() {}
	};
}

/// Code that is run after on_init and before autonomous.
#[macro_export]
macro_rules! comp_init {
    {$fn:block} => {
        #[no_mangle]
        extern "C" fn competition_initialize() {
            $fn
        }
    };
	() => {
		#[no_mangle]
        extern "C" fn competition_initialize() {}
	};
}

/// Code that is run during teleop.
#[macro_export]
macro_rules! opcontrol {
    {$fn:block} => {
        #[no_mangle]
        extern "C" fn opcontrol() {
            $fn
        }
    };
	() => {
		#[no_mangle]
        extern "C" fn opcontrol() {}
	};
}

pub mod prelude {
    pub use crate::{autonomous, comp_init, on_disable, on_initialize, opcontrol};
    pub use crate::{print, println};
}
