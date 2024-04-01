use core::alloc::{GlobalAlloc, Layout};

struct Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: caller must ensure that the alignment and size are valid for the given layout
        unsafe { pros_sys::memalign(layout.align() as _, layout.size() as _) as *mut u8 }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        // SAFETY: caller must ensure that the given ptr can be deallocated
        unsafe { pros_sys::free(ptr as *mut core::ffi::c_void) }
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
