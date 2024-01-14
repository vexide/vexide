//! Scrolling text writer for the LLEMU.

extern crate alloc;

use alloc::{ffi::CString, string::String};

const V5_SCREEN_HEIGHT: usize = 8;

pub(crate) struct ConsoleLcd {
    lines: [CString; V5_SCREEN_HEIGHT],
    bottom_line_index: usize,
    current_line: String,
}

impl ConsoleLcd {
    pub fn new() -> Self {
        unsafe {
            pros_sys::lcd_initialize();
        }

        Self {
            lines: Default::default(),
            bottom_line_index: V5_SCREEN_HEIGHT - 1,
            current_line: String::new(),
        }
    }
}

impl core::fmt::Write for ConsoleLcd {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let mut should_render = false;
        for c in text.chars() {
            if c == '\n' {
                should_render = true;
                let line = CString::new(core::mem::take(&mut self.current_line))
                    .expect("line should not contain null (U+0000) bytes");

                self.shift_up_wrapping();
                self.lines[self.bottom_line_index] = line;
            } else {
                self.current_line.push(c);
            }
        }

        if should_render {
            self.render()?;
        }

        Ok(())
    }
}

impl ConsoleLcd {
    fn shift_up_wrapping(&mut self) {
        self.bottom_line_index = (self.bottom_line_index + 1) % V5_SCREEN_HEIGHT;
    }
    fn render(&self) -> core::fmt::Result {
        for (i, text) in self.lines.iter().enumerate() {
            const MAX_INDEX: usize = V5_SCREEN_HEIGHT - 1;
            let index_offset = MAX_INDEX - self.bottom_line_index;
            let line_num = (i + index_offset) % V5_SCREEN_HEIGHT;
            let success =
                unsafe { pros_sys::lcd_set_text(line_num.try_into().unwrap(), text.as_ptr()) };
            if !success {
                return Err(core::fmt::Error);
            }
        }
        Ok(())
    }
}
