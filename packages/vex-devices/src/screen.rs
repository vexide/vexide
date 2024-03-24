//! Brain screen display and touch functions.
//!
//! Contains user calls to the v5 screen for touching and displaying graphics.
//! The [`Fill`] trait can be used to draw shapes and text to the screen.

use alloc::{ffi::CString, string::String, vec::Vec};

use pros_core::{bail_on, map_errno};
use pros_sys::PROS_ERR;
use snafu::Snafu;

use crate::color::{IntoRgb, Rgb};

#[derive(Debug, Eq, PartialEq)]
/// Represents the physical display on the V5 Brain.
pub struct Screen {
    writer_buffer: String,
    current_line: i16,
}

impl core::fmt::Write for Screen {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        for character in text.chars() {
            if character == '\n' {
                if self.current_line > (Self::MAX_VISIBLE_LINES as i16 - 2) {
                    self.scroll(0, Self::LINE_HEIGHT)
                        .map_err(|_| core::fmt::Error)?;
                } else {
                    self.current_line += 1;
                }

                self.flush_writer().map_err(|_| core::fmt::Error)?;
            } else {
                self.writer_buffer.push(character);
            }
        }

        self.fill(
            &Text::new(
                self.writer_buffer.as_str(),
                TextPosition::Line(self.current_line),
                TextFormat::Medium,
            ),
            Rgb::WHITE,
        )
        .map_err(|_| core::fmt::Error)?;

        Ok(())
    }
}

/// A type implementing this trait can draw a filled shape to the display.
pub trait Fill {
    /// The type of error that can be generated when drawing to the screen.
    type Error;

    /// Draw a filled shape to the display.
    fn fill(&self, screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error>;
}

/// A type implementing this trait can draw an outlined shape to the display.
pub trait Stroke {
    /// The type of error that can be generated when drawing to the screen.
    type Error;

    /// Draw an outlined shape to the display.
    fn stroke(&self, screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A circle that can be drawn on the screen.
pub struct Circle {
    x: i16,
    y: i16,
    radius: i16,
}

impl Circle {
    /// Create a circle with the given coordinates and radius.
    /// The coordinates are the center of the circle.
    pub const fn new(x: i16, y: i16, radius: i16) -> Self {
        Self { x, y, radius }
    }
}

impl Fill for Circle {
    type Error = ScreenError;

    fn fill(&self, _screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_set_pen(color.into_rgb().into())
        });
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_fill_circle(self.x, self.y, self.radius)
        });

        Ok(())
    }
}

impl Stroke for Circle {
    type Error = ScreenError;

    fn stroke(&self, _screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_set_pen(color.into_rgb().into())
        });
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_draw_circle(self.x, self.y, self.radius)
        });

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A line that can be drawn on the screen.
/// The width is the same as the pen width.
pub struct Line {
    x0: i16,
    y0: i16,
    x1: i16,
    y1: i16,
}

impl Line {
    /// Create a new line with the given coordinates.
    pub const fn new(x0: i16, y0: i16, x1: i16, y1: i16) -> Self {
        Self { x0, y0, x1, y1 }
    }
}

impl Fill for Line {
    type Error = ScreenError;

    fn fill(&self, _screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_set_pen(color.into_rgb().into())
        });
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_draw_line(self.x0, self.y0, self.x1, self.y1)
        });

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A rectangle that can be drawn on the screen.
pub struct Rect {
    x0: i16,
    y0: i16,
    x1: i16,
    y1: i16,
}

impl Rect {
    /// Create a new rectangle with the given coordinates.
    pub const fn new(start_x: i16, start_y: i16, end_x: i16, end_y: i16) -> Self {
        Self {
            x0: start_x,
            y0: start_y,
            x1: end_x,
            y1: end_y,
        }
    }
}

impl Stroke for Rect {
    type Error = ScreenError;

    fn stroke(&self, _screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_set_pen(color.into_rgb().into())
        });
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_draw_rect(self.x0, self.y0, self.x1, self.y1)
        });

        Ok(())
    }
}

impl Fill for Rect {
    type Error = ScreenError;

    fn fill(&self, _screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_set_pen(color.into_rgb().into())
        });
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_fill_rect(self.x0, self.y0, self.x1, self.y1)
        });

        Ok(())
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Options for how a text object should be formatted.
pub enum TextFormat {
    /// Small text.
    Small = pros_sys::E_TEXT_SMALL,
    /// Medium text.
    Medium = pros_sys::E_TEXT_MEDIUM,
    /// Large text.
    Large = pros_sys::E_TEXT_LARGE,
    /// Medium horizontally centered text.
    MediumCenter = pros_sys::E_TEXT_MEDIUM_CENTER,
    /// Large horizontally centered text.
    LargeCenter = pros_sys::E_TEXT_LARGE_CENTER,
}

impl From<TextFormat> for pros_sys::text_format_e_t {
    fn from(value: TextFormat) -> pros_sys::text_format_e_t {
        value as _
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// The position of a text object on the screen.
pub enum TextPosition {
    /// A point to draw the text at.
    Point(i16, i16),
    /// A line number to draw the text at.
    Line(i16),
}

#[derive(Debug, Clone, Eq, PartialEq)]
/// A peice of text that can be drawn on the display.
pub struct Text {
    position: TextPosition,
    text: CString,
    format: TextFormat,
}

impl Text {
    /// Create a new text with a given position and format
    pub fn new(text: &str, position: TextPosition, format: TextFormat) -> Self {
        Self {
            text: CString::new(text)
                .expect("CString::new encountered NULL (U+0000) byte in non-terminating position."),
            position,
            format,
        }
    }
}

impl Fill for Text {
    type Error = ScreenError;

    fn fill(&self, _screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_set_pen(color.into_rgb().into())
        });
        bail_on!(PROS_ERR as u32, unsafe {
            match self.position {
                TextPosition::Point(x, y) => {
                    pros_sys::screen_print_at(self.format.into(), x, y, self.text.as_ptr())
                }
                TextPosition::Line(line) => {
                    pros_sys::screen_print(self.format.into(), line, self.text.as_ptr())
                }
            }
        });

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// A touch event on the screen.
pub struct TouchEvent {
    /// Touch state.
    pub state: TouchState,
    /// X coordinate of the touch.
    pub x: i16,
    /// Y coordinate of the touch.
    pub y: i16,
    /// how many times the screen has been pressed.
    pub press_count: i32,
    /// how many times the screen has been released.
    pub release_count: i32,
}

impl TryFrom<pros_sys::screen_touch_status_s_t> for TouchEvent {
    type Error = ScreenError;

    fn try_from(value: pros_sys::screen_touch_status_s_t) -> Result<Self, Self::Error> {
        Ok(Self {
            state: value.touch_status.try_into()?,
            x: value.x,
            y: value.y,
            press_count: value.press_count,
            release_count: value.release_count,
        })
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// The state of a given touch.
pub enum TouchState {
    /// The touch has been released.
    Released = pros_sys::E_TOUCH_RELEASED,
    /// The screen has been touched.
    Pressed = pros_sys::E_TOUCH_PRESSED,
    /// The touch is still being held.
    Held = pros_sys::E_TOUCH_HELD,
}

impl TryFrom<pros_sys::last_touch_e_t> for TouchState {
    type Error = ScreenError;

    fn try_from(value: pros_sys::last_touch_e_t) -> Result<Self, Self::Error> {
        bail_on!(pros_sys::E_TOUCH_ERROR, value);

        Ok(match value {
            pros_sys::E_TOUCH_RELEASED => Self::Released,
            pros_sys::E_TOUCH_PRESSED => Self::Pressed,
            pros_sys::E_TOUCH_HELD => Self::Held,
            _ => unreachable!(),
        })
    }
}

impl From<TouchState> for pros_sys::last_touch_e_t {
    fn from(value: TouchState) -> pros_sys::last_touch_e_t {
        value as _
    }
}

impl Screen {
    /// The maximum number of lines that can be visible on the screen at once.
    pub const MAX_VISIBLE_LINES: usize = 12;

    /// The height of a single line of text on the screen.
    pub const LINE_HEIGHT: i16 = 20;

    /// The horizontal resolution of the display.
    pub const HORIZONTAL_RESOLUTION: i16 = 480;

    /// The vertical resolution of the writable part of the display.
    pub const VERTICAL_RESOLUTION: i16 = 240;

    /// Create a new screen.
    ///
    /// # Safety
    ///
    /// Creating new `Screen`s is inherently unsafe due to the possibility of constructing
    /// more than one screen at once allowing multiple mutable references to the same
    /// hardware device. Prefer using [`Peripherals`](crate::peripherals::Peripherals) to register devices if possible.
    pub unsafe fn new() -> Self {
        Self {
            current_line: 0,
            writer_buffer: String::default(),
        }
    }

    fn flush_writer(&mut self) -> Result<(), ScreenError> {
        self.fill(
            &Text::new(
                self.writer_buffer.as_str(),
                TextPosition::Line(self.current_line),
                TextFormat::Medium,
            ),
            Rgb::WHITE,
        )?;

        self.writer_buffer.clear();

        Ok(())
    }

    /// Scroll the entire display buffer.
    ///
    /// This function effectively y-offsets all pixels drawn to the display buffer by
    /// a number (`offset`) of pixels.
    pub fn scroll(&mut self, start: i16, offset: i16) -> Result<(), ScreenError> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_scroll(start, offset)
        });

        Ok(())
    }

    /// Scroll a region of the screen.
    ///
    /// This will effectively y-offset the display buffer in this area by
    /// `offset` pixels.
    pub fn scroll_area(
        &mut self,
        x0: i16,
        y0: i16,
        x1: i16,
        y1: i16,
        offset: i16,
    ) -> Result<(), ScreenError> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_scroll_area(x0, y0, x1, y1, offset)
        });

        Ok(())
    }

    /// Draw a filled object to the screen.
    pub fn fill(
        &mut self,
        shape: &impl Fill<Error = ScreenError>,
        color: impl IntoRgb,
    ) -> Result<(), ScreenError> {
        shape.fill(self, color)
    }

    /// Draw an outlined object to the screen.
    pub fn stroke(
        &mut self,
        shape: &impl Stroke<Error = ScreenError>,
        color: impl IntoRgb,
    ) -> Result<(), ScreenError> {
        shape.stroke(self, color)
    }

    /// Wipe the entire display buffer, filling it with a specified color.
    pub fn erase(color: impl IntoRgb) -> Result<(), ScreenError> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_set_eraser(color.into_rgb().into())
        });
        bail_on!(PROS_ERR as u32, unsafe { pros_sys::screen_erase() });

        Ok(())
    }

    /// Draw a color to a specified pixel position on the screen.
    pub fn draw_pixel(x: i16, y: i16) -> Result<(), ScreenError> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_draw_pixel(x, y)
        });

        Ok(())
    }

    /// Draw a buffer of pixel colors to a specified region of the screen.
    pub fn draw_buffer<T, I>(
        &mut self,
        x0: i16,
        y0: i16,
        x1: i16,
        y1: i16,
        buf: T,
        src_stride: i32,
    ) -> Result<(), ScreenError>
    where
        T: IntoIterator<Item = I>,
        I: IntoRgb,
    {
        let raw_buf = buf
            .into_iter()
            .map(|i| i.into_rgb().into())
            .collect::<Vec<_>>();
        // Convert the coordinates to u32 to avoid overflows when multiplying.
        let expected_size = ((x1 - x0) as u32 * (y1 - y0) as u32) as usize;
        if raw_buf.len() != expected_size {
            return Err(ScreenError::CopyBufferWrongSize {
                buffer_size: raw_buf.len(),
                expected_size,
            });
        }

        // SAFETY: The buffer is guaranteed to be the correct size.
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_copy_area(x0, y0, x1, y1, raw_buf.as_ptr(), src_stride)
        });

        Ok(())
    }

    /// Draw an error box to the screen.
    ///
    /// This function is internally used by the pros-rs panic handler for displaying
    /// panic messages graphically before exiting.
    pub fn draw_error(&mut self, msg: &str) -> Result<(), ScreenError> {
        const ERROR_BOX_MARGIN: i16 = 16;
        const ERROR_BOX_PADDING: i16 = 16;
        const LINE_MAX_WIDTH: usize = 52;

        let error_box_rect = Rect::new(
            ERROR_BOX_MARGIN,
            ERROR_BOX_MARGIN,
            Self::HORIZONTAL_RESOLUTION - ERROR_BOX_MARGIN,
            Self::VERTICAL_RESOLUTION - ERROR_BOX_MARGIN,
        );

        self.fill(&error_box_rect, Rgb::RED)?;
        self.stroke(&error_box_rect, Rgb::WHITE)?;

        let mut buffer = String::new();
        let mut line: i16 = 0;

        for (i, character) in msg.char_indices() {
            if !character.is_ascii_control() {
                buffer.push(character);
            }

            if character == '\n' || ((buffer.len() % LINE_MAX_WIDTH == 0) && (i > 0)) {
                self.fill(
                    &Text::new(
                        buffer.as_str(),
                        TextPosition::Point(
                            ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                            ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * Self::LINE_HEIGHT),
                        ),
                        TextFormat::Small,
                    ),
                    Rgb::WHITE,
                )?;

                line += 1;
                buffer.clear();
            }
        }

        self.fill(
            &Text::new(
                buffer.as_str(),
                TextPosition::Point(
                    ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                    ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * Self::LINE_HEIGHT),
                ),
                TextFormat::Small,
            ),
            Rgb::WHITE,
        )?;

        Ok(())
    }

    /// Get the current touch status of the screen.
    pub fn touch_status(&self) -> Result<TouchEvent, ScreenError> {
        unsafe { pros_sys::screen_touch_status() }.try_into()
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with the screen.
pub enum ScreenError {
    /// Another resource is currently trying to access the screen mutex.
    ConcurrentAccess,

    /// The given buffer of colors was wrong size to fill the specified area.
    CopyBufferWrongSize {
        /// The size of the buffer.
        buffer_size: usize,
        /// The expected size of the buffer.
        expected_size: usize,
    },
}

map_errno! {
    ScreenError {
        EACCES => Self::ConcurrentAccess,
    }
}
