extern crate alloc;

use alloc::{ffi::CString, borrow::ToOwned};

pub(crate) struct Writer {
    lines: [CString; 8],
}

impl Writer {
    pub fn new() -> Self {
        unsafe {
            pros_sys::lcd_initialize();
        }

        Self {
            lines: Default::default(),
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut owned_line = self.lines[7].to_str().unwrap().to_owned();
        
        for ch in s.chars() {
            match ch {
                '\n' => {
                    self.new_line();
                    owned_line.clear();
                },
                ch => owned_line.push(ch),
            }
        }

        self.lines[7] = CString::new(owned_line).unwrap();

        unsafe {
            pros_sys::lcd_print(7, self.lines[7].as_ptr())
        };

        Ok(())
    }
}

impl Writer {
    fn new_line(&mut self) {
        let mut new_line = CString::default();

        for line in (0..self.lines.len()).rev() {
            core::mem::swap(&mut self.lines[line], &mut new_line);

            let string_ptr = self.lines[line].as_ptr();

            unsafe {
                pros_sys::lcd_print(line as i16, string_ptr);
            }
        }
    }
}
