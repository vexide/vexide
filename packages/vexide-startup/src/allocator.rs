//! Global heap memory allocator.
//!
//! This module provide's vexide's `#[global_allocator]`, which is implemented using the `talc`
//! crate. This allocator is preferred over the default `dlmalloc` allocator provided by libstd
//! because it's more optimized with respect to both performance and size.
//!
//! [`claim`] must be called before any heap allocations are made. This is done automatically when
//! calling [`startup`](crate::startup), so you should not need to call it yourself unless you are
//! writing your own startup routine implementation or need to claim a new heap region.

#[cfg(target_os = "vexos")]
use talc::{ErrOnOom, Span, Talc, Talck, locking::AssumeUnlockable};

#[cfg(target_os = "vexos")]
#[global_allocator]
static ALLOCATOR: Talck<AssumeUnlockable, ErrOnOom> = Talc::new(ErrOnOom).lock();

/// Claims a region of memory as heap space.
///
/// # Safety
///
/// - The memory within the `memory` must be valid for reads and writes, and memory therein (when
///   not allocated to the user) must not be mutated while the allocator is in use.
///
///  - The region encompassed from [`start`, `end`] should not overlap with any other active heap
///    regions.
#[allow(unused_variables, clippy::missing_const_for_fn)] // Silences warnings when not compiling for VEXos
pub unsafe fn claim(start: *mut u8, end: *mut u8) {
    #[cfg(target_os = "vexos")]
    unsafe {
        ALLOCATOR.lock().claim(Span::new(start, end)).unwrap();
    }
}
