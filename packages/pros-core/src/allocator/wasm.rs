//! WASM host will call these functions to send values to the program

extern crate alloc;

use alloc::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    collections::BTreeMap,
};

use dlmalloc::GlobalDlmalloc;

// no multithreading in wasm
static mut LAYOUTS: BTreeMap<*mut u8, Layout> = BTreeMap::new();

#[no_mangle]
extern "C" fn wasm_memalign(alignment: usize, size: usize) -> *mut u8 {
    if size == 0 {
        return core::ptr::null_mut();
    }
    let Ok(layout) = Layout::from_size_align(size, alignment) else {
        return core::ptr::null_mut();
    };
    let ptr = unsafe { alloc(layout) };
    if ptr.is_null() {
        handle_alloc_error(layout);
    }
    unsafe {
        LAYOUTS.insert(ptr, layout);
    }
    ptr
}

#[no_mangle]
extern "C" fn wasm_free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    let layout = unsafe { LAYOUTS.remove(&ptr) };
    if let Some(layout) = layout {
        unsafe { dealloc(ptr, layout) };
    }
}

#[global_allocator]
static ALLOCATOR: GlobalDlmalloc = GlobalDlmalloc;
