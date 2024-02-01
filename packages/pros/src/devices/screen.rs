//! Brain screen display and touch functions.
//!
//! Contains user calls to the v5 screen for touching and displaying graphics.

use alloc::{ffi::CString, string::String, vec::Vec};

use pros_sys::PROS_ERR;
use snafu::Snafu;

use crate::{
    color::{IntoRgb, Rgb},
    error::{bail_on, map_errno},
};

#[derive(Debug, Eq, PartialEq)]
pub struct Screen {
    writer_buffer: String,
    current_line: i16,
}

pub const SCREEN_MAX_VISIBLE_LINES: usize = 12;
pub const SCREEN_LINE_HEIGHT: i16 = 20;

pub const SCREEN_HORIZONTAL_RESOLUTION: i16 = 480;
pub const SCREEN_VERTICAL_RESOLUTION: i16 = 240;

impl core::fmt::Write for Screen {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        for character in text.chars() {
            if character == '\n' {
                if self.current_line > (SCREEN_MAX_VISIBLE_LINES as i16 - 2) {
                    self.scroll(0, SCREEN_LINE_HEIGHT as i16)
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

pub trait Fill {
    type Error;

    fn fill(&self, screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error>;
}

pub trait Stroke {
    type Error;

    fn stroke(&self, screen: &mut Screen, color: impl IntoRgb) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Circle {
    x: i16,
    y: i16,
    radius: i16,
}

impl Circle {
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
pub struct Line {
    x0: i16,
    y0: i16,
    x1: i16,
    y1: i16,
}

impl Line {
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
pub struct Rect {
    x0: i16,
    y0: i16,
    x1: i16,
    y1: i16,
}

impl Rect {
    pub const fn new(x0: i16, y0: i16, x1: i16, y1: i16) -> Self {
        Self { x0, y0, x1, y1 }
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
pub enum TextFormat {
    Small = pros_sys::E_TEXT_SMALL,
    Medium = pros_sys::E_TEXT_MEDIUM,
    Large = pros_sys::E_TEXT_LARGE,
    MediumCenter = pros_sys::E_TEXT_MEDIUM_CENTER,
    LargeCenter = pros_sys::E_TEXT_LARGE_CENTER,
}

impl From<TextFormat> for pros_sys::text_format_e_t {
    fn from(value: TextFormat) -> pros_sys::text_format_e_t {
        value as _
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextPosition {
    Point(i16, i16),
    Line(i16),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Text {
    position: TextPosition,
    text: CString,
    format: TextFormat,
}

impl Text {
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
pub struct TouchEvent {
    pub state: TouchState,
    pub x: i16,
    pub y: i16,
    pub press_count: i32,
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
pub enum TouchState {
    Released = pros_sys::E_TOUCH_RELEASED,
    Pressed = pros_sys::E_TOUCH_PRESSED,
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
    /// Create a new screen.
    ///
    /// # Safety
    ///
    /// Creating new `Screen`s is inherently unsafe due to the possibility of constructing
    /// more than one screen at once allowing multiple mutable references to the same
    /// hardware device. Prefer using [`Peripherals`] to register devices if possible.
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
        stride: i32,
    ) -> Result<(), ScreenError>
    where
        T: IntoIterator<Item = I>,
        I: IntoRgb,
    {
        let raw_buf = buf
            .into_iter()
            .map(|i| i.into_rgb().into())
            .collect::<Vec<_>>();

        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_copy_area(x0, y0, x1, y1, raw_buf.as_ptr(), stride)
        });

        Ok(())
    }

    /// Draw an error box to the screen.
    ///
    /// This function is internally used by the pros-rs panic handler for displaying
    /// panic messages graphically before exiting.
    pub(crate) fn draw_error(&mut self, msg: &str) -> Result<(), ScreenError> {
        const ERROR_BOX_MARGIN: i16 = 16;
        const ERROR_BOX_PADDING: i16 = 16;
        const LINE_MAX_WIDTH: usize = 52;

        let error_box_rect = Rect::new(
            ERROR_BOX_MARGIN,
            ERROR_BOX_MARGIN,
            SCREEN_HORIZONTAL_RESOLUTION - ERROR_BOX_MARGIN,
            SCREEN_VERTICAL_RESOLUTION - ERROR_BOX_MARGIN,
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
                            ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * SCREEN_LINE_HEIGHT),
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
                    ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * SCREEN_LINE_HEIGHT),
                ),
                TextFormat::Small,
            ),
            Rgb::WHITE,
        )?;

        Ok(())
    }

    pub fn touch_status(&self) -> Result<TouchEvent, ScreenError> {
        unsafe { pros_sys::screen_touch_status() }.try_into()
    }
}

#[derive(Debug, Snafu)]
pub enum ScreenError {
    #[snafu(display("Another resource is currently trying to access the screen mutex."))]
    ConcurrentAccess,
}

map_errno! {
    ScreenError {
        EACCES => Self::ConcurrentAccess,
    }
}
