#![no_std]

use core::alloc::{Layout, GlobalAlloc};
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

//TODO: This currently does not compile. libc does not include the __errno_location function for some reason. FIX THIS.
// pub(crate) mod errno;

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    println!("Panicked! {_info}");
    loop {}
}

struct Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		pros_sys::malloc(layout.size() as u32) as *mut u8
	}
	unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
		pros_sys::free(ptr as *mut core::ffi::c_void)
	}
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
