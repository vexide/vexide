use lazy_static::lazy_static;
use crate::bindings;
use spin::Mutex;

pub struct Errno {
    errno: *mut core::ffi::c_int,
}
unsafe impl Send for Errno {}

impl Errno {
    pub unsafe fn new() -> Self {
        Self {
            errno: bindings::__errno_location(),
        }
    }
}

lazy_static! {
    pub static ref ERRNO: Mutex<Errno> = unsafe { Mutex::new(Errno::new()) };
}