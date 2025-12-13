//! Functions to query system memory for display in the debugger.

use std::ptr;

use gdbstub::target::TargetResult;

use crate::dbg_target::VexideTarget;

#[allow(clippy::unnecessary_wraps, clippy::missing_const_for_fn)]
pub unsafe fn read(address: usize, buffer: &mut [u8]) -> TargetResult<usize, VexideTarget> {
    // TODO: check MMU table to ensure these pages are readable.

    let ptr = address as *const u8;
    unsafe {
        ptr::copy(ptr, buffer.as_mut_ptr(), buffer.len());
    }
    Ok(buffer.len())
}

#[allow(clippy::unnecessary_wraps, clippy::missing_const_for_fn)]
pub unsafe fn write(address: usize, buffer: &[u8]) -> TargetResult<usize, VexideTarget> {
    // TODO: check MMU table to ensure these pages are writable.

    let ptr = address as *mut u8;
    unsafe {
        ptr::copy(buffer.as_ptr(), ptr, buffer.len());
    }
    Ok(buffer.len())
}
