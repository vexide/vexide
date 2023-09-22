pub fn take_err() -> i32 {
    let err = unsafe { *pros_sys::__errno() };
    if err != 0 {
        unsafe { *pros_sys::__errno() = 0 };
    }
    err
}
