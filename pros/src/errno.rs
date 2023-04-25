use lazy_static::lazy_static;

#[repr(transparent)]
pub(crate) struct Errno(*mut core::ffi::c_int);
unsafe impl Send for Errno {}
unsafe impl Sync for Errno {}

impl Errno {
    pub unsafe fn new() -> Self {
        Self(pros_sys::errno_location())
    }

    pub unsafe fn get(&self) -> core::ffi::c_int {
        let err = self.0;
        *self.0 = 0 as core::ffi::c_int;
        *err
    }
}

lazy_static! {
    pub(crate) static ref ERRNO: Errno = unsafe { Errno::new() };
}
