use lazy_static::lazy_static;
use crate::bindings;
use spin::Mutex;

#[repr(transparent)]
pub struct Errno(*mut core::ffi::c_int);

unsafe impl Send for Errno {}

impl Errno {
    pub unsafe fn new() -> Self {
        Self(bindings::__errno_location())
    }

    pub unsafe fn get(&mut self) -> core::ffi::c_int {
        let err = self.0;
        *self.0 = 0 as core::ffi::c_int;
        *err
    }
}

lazy_static! {
    pub static ref ERRNO: Mutex<Errno> = unsafe { Mutex::new(Errno::new()) };
}