use alloc::{ffi::CString, format};
use pros_sys::exit;

use crate::{
    println,
    task::{self, suspend_all, PanicBehavior, PANIC_BEHAVIOR},
};
use core::{
    alloc::{GlobalAlloc, Layout},
    mem::forget,
    panic::PanicInfo,
};

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    let suspend = unsafe { suspend_all() };

    let current_task = task::current();
    let task_name = current_task.name().unwrap_or_else(|_| "<unknown>".into());
    // task 'User Initialization (PROS)' panicked at src/lib.rs:22:1:
    // panic message here
    let panic_msg = format!("task '{task_name}' {info}");

    let c_msg = CString::new(&*panic_msg).unwrap_or_else(|_| CString::new("Panicked!").unwrap());
    unsafe {
        pros_sys::puts(c_msg.as_ptr());
    }

    if PANIC_BEHAVIOR.with(|p| *p == PanicBehavior::Exit) {
        unsafe {
            exit(1);
        }
    }

    drop(suspend);

    println!("{panic_msg}");

    current_task.abort();
    unreachable!()
}

struct Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        pros_sys::memalign(layout.align() as _, layout.size() as _) as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        pros_sys::free(ptr as *mut core::ffi::c_void)
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
