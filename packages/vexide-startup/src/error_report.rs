use std::fmt::Write;

pub struct ErrorReport {
    pub y_offset: i32,

    // One extra leading byte for a null terminator when word-breaking.
    buf: [u8; Self::LINE_MAX_WIDTH + 1],
    pos: usize,
}

impl ErrorReport {
    pub const HEADER_HEIGHT: i32 = 32;
    pub const DISPLAY_WIDTH: i32 = 480;
    pub const DISPLAY_HEIGHT: i32 = 272;
    pub const BOX_MARGIN: i32 = 16;
    pub const BOX_PADDING: i32 = 16;
    pub const LINE_HEIGHT: i32 = 20;
    pub const LINE_MAX_WIDTH: usize = 52;

    pub const COLOR_RED: u32 = 0x8b0_000;
    pub const COLOR_WHITE: u32 = 0xFFF_FFF;

    pub fn begin() -> Self {
        unsafe {
            vex_sdk::vexDisplayDoubleBufferDisable();
            vex_sdk::vexDisplayForegroundColor(Self::COLOR_RED);
            vex_sdk::vexDisplayRectFill(
                Self::BOX_MARGIN,
                Self::HEADER_HEIGHT + Self::BOX_MARGIN,
                Self::DISPLAY_WIDTH - Self::BOX_MARGIN,
                Self::DISPLAY_HEIGHT - Self::BOX_MARGIN,
            );
            vex_sdk::vexDisplayForegroundColor(Self::COLOR_WHITE);
            vex_sdk::vexDisplayRectDraw(
                Self::BOX_MARGIN,
                Self::HEADER_HEIGHT + Self::BOX_MARGIN,
                Self::DISPLAY_WIDTH - Self::BOX_MARGIN,
                Self::DISPLAY_HEIGHT - Self::BOX_MARGIN,
            );
            vex_sdk::vexDisplayFontNamedSet(c"monospace".as_ptr());
        }

        Self {
            buf: [0; 53],
            pos: 0,
            y_offset: Self::HEADER_HEIGHT + Self::BOX_MARGIN + Self::BOX_PADDING,
        }
    }

    #[allow(unused)]
    pub fn write_registers(&mut self, regs: [u32; 16]) {
        unsafe {
            vex_sdk::vexDisplayTextSize(1, 5);
        }
        for (i, reg) in regs.iter().enumerate() {
            let x = 50 + (i as i32 % 4) * 95;
            match i {
                0..=12 => unsafe {
                    let format = c" r%d: 0x%08x";

                    vex_sdk::vexDisplayPrintf(
                        x,
                        self.y_offset,
                        0,
                        if i < 10 {
                            format.as_ptr()
                        } else {
                            format[1..].as_ptr()
                        },
                        i,
                        *reg,
                    );
                },
                13..16 => unsafe {
                    vex_sdk::vexDisplayPrintf(
                        x,
                        self.y_offset,
                        0,
                        match i {
                            13 => c" sp: 0x%08x".as_ptr(),
                            14 => c" lr: 0x%08x".as_ptr(),
                            15 => c" pc: 0x%08x".as_ptr(),
                            _ => core::hint::unreachable_unchecked(),
                        },
                        *reg,
                        i,
                    );
                },
                _ => {}
            }

            if i % 4 == 3 {
                self.y_offset += 10;
            }
        }

        self.y_offset += 10;
    }

    #[cfg(all(target_os = "vexos", feature = "backtrace"))]
    pub fn write_backtrace(&mut self, trace: impl Iterator<Item = u32>) {
        unsafe {
            vex_sdk::vexDisplayTextSize(1, 5);
        }
        let mut i = 0;
        for frame in trace {
            let format = c"  %d: 0x%08x";

            unsafe {
                vex_sdk::vexDisplayPrintf(
                    50 + (i % 4) * 95,
                    self.y_offset,
                    0,
                    if i < 10 {
                        format.as_ptr()
                    } else {
                        format[1..].as_ptr()
                    },
                    i,
                    frame,
                );
            }

            if i % 4 == 3 {
                self.y_offset += 10;
            }

            i += 1;
        }

        if i % 4 != 0 {
            self.y_offset += 20;
        } else {
            self.y_offset += 10;
        }
    }
}

impl Write for ErrorReport {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        unsafe {
            vex_sdk::vexDisplayTextSize(1, 4);
        }

        for mut character in s.chars() {
            // Brain's default font handling only supports ASCII (though there are CJK fonts).
            if !character.is_ascii() {
                character = '?';
            }

            self.buf[self.pos] = character as u8;

            if character == '\n' || self.pos == Self::LINE_MAX_WIDTH - 1 {
                let wrap_point = if character == '\n' {
                    // Wrap due to early LF.
                    self.buf[self.pos] = 0; // insert null terminator at newline
                    self.pos
                } else if let Some(space_position) = self.buf.iter().rposition(|x| *x == b' ') {
                    // Word wrap if there's a space on the current line.
                    self.buf[space_position] = 0; // insert null terminator at space
                    space_position
                } else {
                    // Fallback to letter wrapping if there's no space on the line.
                    Self::LINE_MAX_WIDTH - 1
                };

                // Put the thing on the screen!
                unsafe {
                    vex_sdk::vexDisplayPrintf(
                        Self::BOX_MARGIN + Self::BOX_PADDING,
                        self.y_offset,
                        0,
                        self.buf.as_ptr().cast(),
                    );
                }
                self.y_offset += Self::LINE_HEIGHT;

                // Since we just wrapped, we need to move the remaining characters in the string (if
                // there are any) to the front of the next line.
                if self.pos == Self::LINE_MAX_WIDTH - 1 {
                    self.buf.copy_within(wrap_point + 1.., 0);
                    self.pos -= wrap_point;
                } else {
                    self.pos = 0;
                }

                continue;
            }

            self.pos += 1;
        }

        Ok(())
    }
}

#[cfg(all(target_os = "vexos", feature = "backtrace"))]
pub mod backtrace {
    use vex_libunwind::{registers, UnwindCursor};

    /// An iterator that lazily walks up the stack, yielding frames in a backtrace.
    pub struct BacktraceIter<'a> {
        pub cursor: Option<UnwindCursor<'a>>,
    }

    impl<'a> BacktraceIter<'a> {
        pub const fn new(cursor: UnwindCursor<'a>) -> Self {
            Self {
                cursor: Some(cursor),
            }
        }
    }

    impl Iterator for BacktraceIter<'_> {
        type Item = u32;

        fn next(&mut self) -> Option<Self::Item> {
            let cursor = self.cursor.as_mut()?;

            let mut instruction_pointer = cursor.register(registers::UNW_REG_IP).ok()?;
            if !cursor.is_signal_frame().ok()? {
                instruction_pointer -= 1;
            }

            if !cursor.step().ok()? {
                self.cursor = None;
            }

            Some(instruction_pointer as u32)
        }
    }
}
