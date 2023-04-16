use crate::bindings;

pub(crate) struct Writer;

impl Writer {
    pub fn new() -> Self {
        unsafe { bindings::lcd_initialize(); }
        
        Self
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            if bindings::lcd_print(line as i16, s.as_ptr() as *const core::ffi::c_char) {
                Ok(())
            } else {
                Err("Failed to print")
            }
        }
    }
}