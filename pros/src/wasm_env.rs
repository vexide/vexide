//! WASM host will call these functions to send values to the program

extern crate alloc;

use core::panic::PanicInfo;

use alloc::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    collections::BTreeMap,
    ffi::CString,
    format,
};

use dlmalloc::GlobalDlmalloc;

// no multithreading in wasm
static mut LAYOUTS: BTreeMap<*mut u8, Layout> = BTreeMap::new();

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    extern "C" {
        fn sim_abort(msg: *const core::ffi::c_char) -> !;
    }

    let msg_str = format!("{info}");
    let msg_c_str = CString::new(msg_str).unwrap();

    unsafe {
        sim_abort(msg_c_str.as_ptr());
    }
}

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
