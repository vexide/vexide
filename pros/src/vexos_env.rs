use alloc::{ffi::CString, format};
use pros_sys::exit;

use crate::{
    println,
    task::{self, suspend_all, PanicBehavior, PANIC_BEHAVIOR},
};
use core::{
    alloc::{GlobalAlloc, Layout},
    panic::PanicInfo,
};

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
