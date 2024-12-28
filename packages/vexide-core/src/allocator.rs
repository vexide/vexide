//! VEXos heap allocator implemented with the `talc` crate.
//!
//! [`init_heap`] must be called before any heap allocations are made.
//! This is done automatically in the `vex-startup` crate,
//! so you should not need to call it yourself unless you are writing your own startup implementation.

use talc::{ErrOnOom, Span, Talc, Talck};

use crate::sync::RawMutex;

#[global_allocator]
static ALLOCATOR: Talck<RawMutex, ErrOnOom> = Talc::new(ErrOnOom).lock();

/// Claims a region of memory as heap space.
///
/// # Safety
/// - The memory within the `memory` must be valid for reads and writes, and
///   memory therein (when not allocated to the user) must not be mutated while
///   the allocator is in use.
/// - The region encompassed from [`start`, `end`] should not overlap with any
///   other active heap regions.
///
/// # Panics
///
/// Panics if the `__heap_start` or `__heap_end` symbols set in the linker script are null.
pub unsafe fn claim(start: *mut u8, end: *mut u8) {
    //SAFETY: User must ensure that this function is only called once.
    unsafe {
        ALLOCATOR.lock().claim(Span::new(start, end)).unwrap();
    }
}
