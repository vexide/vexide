//! Brain screen display and touch functions.
//!
//! Contains user calls to the v5 screen for touching and displaying graphics.

use alloc::{ffi::CString, vec::Vec};

use pros_sys::PROS_ERR;
use snafu::Snafu;

use crate::{
    color::IntoRgb,
    error::{bail_on, map_errno},
};

#[derive(Debug, Eq, PartialEq)]
pub struct Screen {
    // Forces unsafe construction through [`Self::new`].
    _private: (),
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
    pub fn new(text: impl AsRef<str>, position: TextPosition, format: TextFormat) -> Self {
        Self {
            text: CString::new(text.as_ref())
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
    pub const unsafe fn new() -> Self {
        Self { _private: () }
    }

    pub fn scroll(&mut self, start_line: i16, lines: i16) -> Result<(), ScreenError> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_scroll(start_line, lines)
        });

        Ok(())
    }

    pub fn scroll_area(
        &mut self,
        x0: i16,
        y0: i16,
        x1: i16,
        y1: i16,
        lines: i16,
    ) -> Result<(), ScreenError> {
        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_scroll_area(x0, y0, x1, y1, lines)
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
        let raw_buf = buf.into_iter().map(|i| i.into_rgb().into()).collect::<Vec<_>>();

        bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::screen_copy_area(x0, y0, x1, y1, raw_buf.as_ptr(), stride)
        });

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
