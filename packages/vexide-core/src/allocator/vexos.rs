//! VexOS heap allocator implemented with the `linked_list_allocator` crate.
//! [`init_heap`] must be called before any heap allocations are made.
//! This is done automatically in the `vex-startup` crate,
//! so you should not need to call it yourself unless you are writing your own startup implementation.

use linked_list_allocator::LockedHeap;

extern "C" {
    static __heap_start: *mut u8;
    static __heap_length: usize;
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initializes the heap allocator.
///
/// # SAFETY
///
/// This function can only be called once.
pub unsafe fn init_heap() {
    //SAFETY: User must ensure that this function is only called once.
    unsafe {
        ALLOCATOR.lock().init(__heap_start, __heap_length);
    }
}
