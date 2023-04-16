extern crate alloc;

use alloc::{borrow::ToOwned, boxed::Box};

use crate::bindings;

pub(crate) struct Writer {
    lines: [&'static str; 8],
}

impl Writer {
    pub fn new() -> Self {
        unsafe { bindings::lcd_initialize(); }
        
        Self {
            lines: [""; 8]
        }
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for line in 1..self.lines.len() {
            unsafe {
                bindings::lcd_print((line - 1) as i16, self.lines[line].as_ptr() as *const core::ffi::c_char);
                self.lines[line - 1] = self.lines[line];
            }
        }

        unsafe {
            bindings::lcd_print(7, s.as_ptr() as *const core::ffi::c_char);
            let s_copy = s.clone().to_owned();
            self.lines[7] = Box::leak(Box::new(s_copy));
        }

        Ok(())
    }
}

// pub struct WriteError;
// impl core::error::Error for WriteError {};