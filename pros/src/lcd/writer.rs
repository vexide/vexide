extern crate alloc;

use alloc::{borrow::ToOwned, string::String, ffi::CString};

pub(crate) struct Writer {
    lines: [CString; 8],
}

impl Writer {
    pub fn new() -> Self {
        unsafe {
            pros_sys::lcd_initialize();
        }

        Self { lines: Default::default() }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut new_line = CString::new(s).unwrap();

        for line in (0..self.lines.len()).rev() {
            core::mem::swap(&mut self.lines[line], &mut new_line);

            let string_ptr = self.lines[line].as_ptr();

            unsafe {
                pros_sys::lcd_print(
                    line as i16,
                    string_ptr,
                );
            }
        }

        Ok(())
    }
}