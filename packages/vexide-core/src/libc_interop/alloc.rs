//! This is an implementation of C memory management which delegates to vexide's allocator.

use core::{
    alloc::Layout,
    ffi::{c_int, c_void},
    ptr::{self, NonNull},
};

use static_assertions::const_assert;

use crate::libc_interop::errors;

/// The alignment used for `malloc` allocations. This is set to 8 bytes
/// because that is the largest alignment size listed in the ARM manual.
///
/// Always a power of two.
///
/// See: <https://developer.arm.com/documentation/dui0472/m/C-and-C---Implementation-Details/Basic-data-types-in-ARM-C-and-C-->
const DEFAULT_ALIGNMENT: usize = {
    if cfg!(target_arch = "arm") {
        8
    } else {
        panic!("The alignment of malloc() allocations is not configured for this arch");
    }
};

const_assert!(DEFAULT_ALIGNMENT.is_power_of_two());

/// A plan for allocating a buffer with a header.
///
/// # Invariants
///
/// - [`Self::header_size`] is always greater than or equal to the size of [`AllocMetadata`]
///   and smaller than [`Self::allocation_size`] and aligned to [`Self::align`].
/// - [`Self::align`] is a power of two.
/// - [`Self::allocation_size`] is never zero.
#[derive(Debug, Clone, Copy)]
struct AllocationPlan {
    header_size: usize,
    allocation_size: usize,
    align: usize,
}

impl AllocationPlan {
    /// Creates a plan that will allocate a buffer suitable for holding
    /// the specified amount of data at the correct alignment.
    ///
    /// # Safety
    ///
    /// `align` must be a power of two and be at last as large as the alignment
    /// of [`AllocMetadata`].
    pub const unsafe fn for_size_align(size: usize, align: usize) -> Self {
        debug_assert!(align.is_power_of_two() && align >= align_of::<AllocMetadata>());

        let header_size = Self::header_size(align);

        Self {
            header_size,
            allocation_size: header_size + size,
            align,
        }
    }

    /// Returns a non-zero-sized Layout suitable for storing the allocation's
    /// header and buffer.
    pub const fn layout(&self) -> Option<Layout> {
        if self.allocation_size.next_multiple_of(self.align) > isize::MAX as usize {
            return None;
        }

        // SAFETY: We just checked that the size won't overflow isize and `self.align` being
        // a power of 2 is an invariant of this type.
        Some(unsafe { Layout::from_size_align_unchecked(self.allocation_size, self.align) })
    }

    pub const fn header_size(align: usize) -> usize {
        // This needs to be big enough to store the metadata but also aligned
        // at least as much as the buffer so that the two can be concatenated.
        size_of::<AllocMetadata>().next_multiple_of(align)
    }
}

/// Describes an allocation. Stored in memory before the allocated chunk of memory.
#[derive(Debug)]
struct AllocMetadata {
    layout: Layout,
}

const_assert!(DEFAULT_ALIGNMENT >= align_of::<AllocMetadata>());

/// A buffer of memory prefixed by a header containing metadata.
///
/// Note that the actual span of memory requested from Rust's
/// allocator begins `sizeof(AllocMetadata)` bytes before the location
/// of this pointer.
#[repr(transparent)]
#[derive(Debug)]
struct AllocationPtr(NonNull<u8>);

impl AllocationPtr {
    /// Allocates a new buffer, or returns None
    pub fn alloc(plan: AllocationPlan, zeroed: bool) -> Option<Self> {
        let layout = plan.layout()?;

        // SAFETY: plan.layout() is guaranteed to return a non-zero size
        let memory = NonNull::new(unsafe {
            if zeroed {
                alloc::alloc::alloc_zeroed(layout)
            } else {
                alloc::alloc::alloc(layout)
            }
        })?;

        // We store a header before the actual buffer so we can remember the layout
        // for when we have to deallocate.

        // SAFETY: This is all part of the same allocation.
        let allocation = unsafe { Self::from_buffer_ptr(memory.add(plan.header_size)) };

        // SAFETY: the entire allocation is valid for writes because we have exclusive mutable access
        unsafe {
            allocation.metadata().write(AllocMetadata { layout });
        }

        Some(allocation)
    }

    /// Deallocate this allocation
    pub fn dealloc(self) {
        let metadata: AllocMetadata = unsafe { self.metadata().read() };
        // SAFETY: this is part of the same allocation as long as this struct was
        // instaniated from a valid pointer
        let base_ptr = unsafe {
            self.into_ptr()
                .sub(AllocationPlan::header_size(metadata.layout.align()))
        };

        // SAFETY: `base_ptr` is the pointer originally returned by alloc, it's still
        // allocated, and this is the same Layout.
        unsafe {
            alloc::alloc::dealloc(base_ptr, metadata.layout);
        }
    }

    /// Interpret a buffer pointer as an allocation
    ///
    /// # Safety
    ///
    /// - The pointer must originate from a previously-created [`AllocationPtr`]
    ///   which has not been deallocated.
    /// - The pointer must have, at minimum, the alignment of [`AllocMetadata`].
    pub const unsafe fn from_buffer_ptr(ptr: NonNull<u8>) -> Self {
        Self(ptr)
    }

    /// Interpret a newly allocated span of memory as an allocation.
    ///
    /// # Safety
    ///
    /// - The pointer must be valid for writes and designate a span of memory
    ///   large enough to contain a [header](AllocationPlan::header_size).
    /// - The pointer must have, at minimum, the alignment of [`AllocMetadata`].
    /// - The pointer's memory must have been allocated with the given `layout`.
    pub const unsafe fn from_base_ptr(ptr: NonNull<u8>, layout: Layout) -> Self {
        // SAFETY: caller has confirmed there is enough space for the header
        Self(unsafe { ptr.add(AllocationPlan::header_size(layout.align())) })
    }

    pub const fn into_ptr(self) -> *mut u8 {
        self.0.as_ptr()
    }

    /// Returns a pointer to beginning of the header, i.e. the pointer
    /// originally returned from Rust's memory allocation facilities.
    ///
    /// # Safety
    ///
    /// - This buffer must have been allocated with the specified alignment.
    ///
    /// See also: [`Self::metadata`]
    pub const unsafe fn base_ptr(&self, align: usize) -> NonNull<u8> {
        // SAFETY: This is all part of the same allocation
        unsafe { self.0.sub(AllocationPlan::header_size(align)).cast() }
    }

    /// Returns a pointer to the metadata stored just before this buffer.
    ///
    /// Note this is not the same as a pointer to the beginning of the header;
    /// the metadata is only the last 8 bytes of the header. The rest is padding.
    pub const fn metadata(&self) -> NonNull<AllocMetadata> {
        // SAFETY: Allocations are always prefixed with an AllocMetadata
        unsafe { self.0.sub(size_of::<AllocMetadata>()).cast() }
    }
}

/// Allocates `size` bytes of memory such that the allocation's base
/// address is a multiple of `align.`
///
/// `align` must be a power of 2 and a multiple of the alignment of a pointer
/// or else this will return `EINVAL`.
///
/// # Safety
///
/// `memptr` must be valid for writes during the duration of this function call.
#[unsafe(no_mangle)]
unsafe extern "C" fn posix_memalign(memptr: *mut *mut c_void, align: usize, size: usize) -> c_int {
    // The "align is a multiple of sizeof(void *)" thing just comes from the spec,
    // but it also means that `align` is always big enough for an AllocMetadata.
    const_assert!(size_of::<*const ()>() >= align_of::<AllocMetadata>());

    if !align.is_power_of_two() || !align.is_multiple_of(size_of::<*const ()>()) {
        return errors::EINVAL;
    }

    if size == 0 {
        unsafe {
            ptr::write(memptr, ptr::null_mut());
        }
        return 0;
    }

    // SAFETY: We just checked that `align` is a power of two.
    // We also just checked that `align` is large enough for an AllocMetadata.
    let plan = unsafe { AllocationPlan::for_size_align(size, align) };
    let Some(alloc_ptr) = AllocationPtr::alloc(plan, false) else {
        return errors::ENOMEM;
    };

    unsafe {
        ptr::write(memptr, alloc_ptr.into_ptr().cast());
    }

    0
}

/// The C `aligned_alloc` function.
///
/// In comparision to `posix_memalign`, this also fails on 0-sized allocations.
#[unsafe(no_mangle)]
unsafe extern "C" fn aligned_alloc(align: usize, size: usize) -> *mut c_void {
    if align.is_power_of_two() && align <= DEFAULT_ALIGNMENT {
        return unsafe { malloc(size) };
    }

    let mut memptr = ptr::null_mut();
    // SAFETY: &memptr is valid for writes because we have exclusive mutable access
    _ = unsafe { posix_memalign(&raw mut memptr, align, size) };

    // If it failed or size is zero, this will still be null.
    // We return null on failure, so no need to do anything else.
    memptr
}

/// The C `malloc` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: usize) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }

    // SAFETY: `DEFAULT_ALIGNMENT` is always a power of two.
    // `DEFAULT_ALIGNMENT` is >= the alignment of `AllocMetadata`.
    let plan = unsafe { AllocationPlan::for_size_align(size, DEFAULT_ALIGNMENT) };
    let Some(alloc_ptr) = AllocationPtr::alloc(plan, false) else {
        return ptr::null_mut();
    };

    alloc_ptr.into_ptr().cast()
}

/// The C `calloc` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn calloc(size: usize) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }

    // SAFETY: `DEFAULT_ALIGNMENT` is always a power of two.
    // `DEFAULT_ALIGNMENT` is >= the alignment of `AllocMetadata`.
    let plan = unsafe { AllocationPlan::for_size_align(size, DEFAULT_ALIGNMENT) };
    let Some(alloc_ptr) = AllocationPtr::alloc(plan, true) else {
        return ptr::null_mut();
    };

    alloc_ptr.into_ptr().cast()
}

/// The C `realloc` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, new_size: usize) -> *mut c_void {
    debug_assert_ne!(new_size, 0);

    let Some(ptr) = NonNull::new(ptr.cast()) else {
        return unsafe { malloc(new_size) };
    };

    // SAFETY: caller guarantees ptr originally came from malloc or an equivalent
    let old_allocation = unsafe { AllocationPtr::from_buffer_ptr(ptr) };
    let old_metadata: AllocMetadata = unsafe { old_allocation.metadata().read() };
    let align = old_metadata.layout.align();

    // SAFETY: Alignments read from a `Layout` are always a power of two.
    // Also, the alignment is big enough to store a header because it originated from
    // an already-existing allocation with a header.
    let new_plan = unsafe { AllocationPlan::for_size_align(new_size, align) };
    let Some(new_layout) = new_plan.layout() else {
        return ptr::null_mut();
    };

    // SAFETY: This alignment is what the the allocation was originally created with.
    let base_ptr = unsafe { old_allocation.base_ptr(align) };

    // SAFETY: Caller guarantees ptr originally came from malloc or an equivalent, so the base_ptr
    // will be correctly offset from that.
    // `layout` is directly pulled from the old metadata, so it's the same as it was when first allocated.
    // Layouts from `plan.layout()` are always non-zero.
    // Sizes from `Layout::size` never overflow isize.
    let Some(new_base_ptr) = NonNull::new(unsafe {
        alloc::alloc::realloc(base_ptr.as_ptr(), old_metadata.layout, new_layout.size())
    }) else {
        return ptr::null_mut();
    };

    // SAFETY: `new_ptr` is valid for writes because we just allocated it.
    // Also, `new_ptr` is valid to store a header because `plan.layout()` returns a buffer
    // big enough and aligned enough to store both the header and buffer.
    let new_allocation = unsafe { AllocationPtr::from_base_ptr(new_base_ptr, new_layout) };

    // Time to update the old metadata so we don't get confused when we free this.
    // SAFETY: This is valid for writes because we just allocated it and it's not null.
    unsafe {
        new_allocation
            .metadata()
            .write(AllocMetadata { layout: new_layout });
    }

    new_allocation.into_ptr().cast()
}

/// The C `free` function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    let Some(ptr) = NonNull::new(ptr.cast()) else {
        return;
    };

    // SAFETY: caller guarantees ptr originally came from malloc or an equivalent
    let allocation = unsafe { AllocationPtr::from_buffer_ptr(ptr) };
    allocation.dealloc();
}
