//! WASM host will call these functions to send values to the program

use std::collections::HashMap;
use std::sync::Mutex;
use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    collections::HashMap,
};

static LAYOUTS: Mutex<HashMap<*mut u8, Layout>> = Mutex::new(HashMap::new());

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
    let Ok(mut layouts) = LAYOUTS.lock() else {
        unsafe { dealloc(ptr, layout) };
        handle_alloc_error(layout);
        unreachable!();
    };
    layouts.insert(ptr, layout);
    ptr as *mut core::ffi::c_void
}

#[no_mangle]
extern "C" fn wasm_free(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    let Ok(mut layouts) = LAYOUTS.lock() else {
        return;
    };
    let layout = layouts.remove(&ptr);
    if let Some(layout) = layout {
        unsafe { dealloc(ptr, layout) };
    }
}
