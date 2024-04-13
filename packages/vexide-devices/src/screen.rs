//! Brain screen display and touch functions.
//!
//! Contains user calls to the v5 screen for touching and displaying graphics.
//! The [`Fill`] trait can be used to draw shapes and text to the screen.

use alloc::{ffi::CString, string::String, vec::Vec};
use core::{mem, time::Duration};

use snafu::Snafu;
use vex_sdk::{
    vexDisplayBackgroundColor, vexDisplayBigCenteredString, vexDisplayBigString,
    vexDisplayBigStringAt, vexDisplayCenteredString, vexDisplayCircleDraw, vexDisplayCircleFill,
    vexDisplayCopyRect, vexDisplayErase, vexDisplayForegroundColor, vexDisplayLineDraw,
    vexDisplayPixelSet, vexDisplayRectDraw, vexDisplayRectFill, vexDisplayScroll,
    vexDisplayScrollRect, vexDisplaySmallStringAt, vexDisplayString, vexDisplayStringAt,
    vexTouchDataGet, V5_TouchEvent, V5_TouchStatus,
};

use crate::color::{IntoRgb, Rgb};

/// Represents the physical display on the V5 Brain.
#[derive(Debug, Eq, PartialEq)]
pub struct Screen {
    writer_buffer: String,
    render_mode: RenderMode,
    current_line: i16,
}

impl core::fmt::Write for Screen {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        for character in text.chars() {
            if character == '\n' {
                if self.current_line > (Self::MAX_VISIBLE_LINES as i16 - 2) {
                    self.scroll(0, Self::LINE_HEIGHT);
                } else {
                    self.current_line += 1;
                }

                self.flush_writer();
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
        );

        Ok(())
    }
}

/// A type implementing this trait can draw a filled shape to the display.
pub trait Fill {
    /// Draw a filled shape to the display.
    fn fill(&self, screen: &mut Screen, color: impl IntoRgb);
}

/// A type implementing this trait can draw an outlined shape to the display.
pub trait Stroke {
    /// Draw an outlined shape to the display.
    fn stroke(&self, screen: &mut Screen, color: impl IntoRgb);
}

/// A circle that can be drawn on the screen.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Circle {
    x: i16,
    y: i16,
    radius: i16,
}

impl Circle {
    /// Create a circle with the given coordinates and radius.
    /// The coordinates are the center of the circle.
    pub const fn new(x: i16, y: i16, radius: i16) -> Self {
        Self {
            x,
            y: y + 0x20,
            radius,
        }
    }
}

impl Fill for Circle {
    fn fill(&self, _screen: &mut Screen, color: impl IntoRgb) {
        unsafe {
            vexDisplayForegroundColor(color.into_rgb().into());
            vexDisplayCircleFill(self.x as _, self.y as _, self.radius as _);
        }
    }
}

impl Stroke for Circle {
    fn stroke(&self, _screen: &mut Screen, color: impl IntoRgb) {
        unsafe {
            vexDisplayForegroundColor(color.into_rgb().into());
            vexDisplayCircleDraw(self.x as _, self.y as _, self.radius as _);
        }
    }
}

/// A line that can be drawn on the screen.
/// The width is the same as the pen width.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Line {
    x0: i16,
    y0: i16,
    x1: i16,
    y1: i16,
}

impl Line {
    /// Create a new line with the given coordinates.
    pub const fn new(x0: i16, y0: i16, x1: i16, y1: i16) -> Self {
        Self {
            x0,
            y0: y0 + 0x20,
            x1,
            y1: y1 + 0x20,
        }
    }
}

impl Fill for Line {
    fn fill(&self, _screen: &mut Screen, color: impl IntoRgb) {
        unsafe {
            vexDisplayForegroundColor(color.into_rgb().into());
            vexDisplayLineDraw(self.x0 as _, self.y0 as _, self.x1 as _, self.y1 as _);
        }
    }
}

/// A rectangle that can be drawn on the screen.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
            y0: start_y + 0x20,
            x1: end_x,
            y1: end_y + 0x20,
        }
    }
}

impl Stroke for Rect {
    fn stroke(&self, _screen: &mut Screen, color: impl IntoRgb) {
        unsafe {
            vexDisplayForegroundColor(color.into_rgb().into());
            vexDisplayRectDraw(self.x0 as _, self.y0 as _, self.x1 as _, self.y1 as _);
        }
    }
}

impl Fill for Rect {
    fn fill(&self, _screen: &mut Screen, color: impl IntoRgb) {
        unsafe {
            vexDisplayForegroundColor(color.into_rgb().into());
            vexDisplayRectFill(self.x0 as _, self.y0 as _, self.x1 as _, self.y1 as _)
        }
    }
}

/// Options for how a text object should be formatted.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextFormat {
    /// Small text.
    Small,
    /// Medium text.
    Medium,
    /// Large text.
    Large,
    /// Medium horizontally centered text.
    MediumCenter,
    /// Large horizontally centered text.
    LargeCenter,
}

/// The position of a text object on the screen.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TextPosition {
    /// A point to draw the text at.
    Point(i16, i16),
    /// A line number to draw the text at.
    Line(i16),
}

/// A peice of text that can be drawn on the display.
#[derive(Debug, Clone, Eq, PartialEq)]
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
                .expect("CString::new encountered NUL (U+0000) byte in non-terminating position."),
            position,
            format,
        }
    }
}

impl Fill for Text {
    fn fill(&self, _screen: &mut Screen, color: impl IntoRgb) {
        // This implementation is technically broken because it doesn't account errno.
        // This will be fixed once we have switched to vex-sdk.
        unsafe {
            vexDisplayForegroundColor(color.into_rgb().into());

            match self.position {
                TextPosition::Point(x, y) => match self.format {
                    TextFormat::Small | TextFormat::LargeCenter => {
                        vexDisplaySmallStringAt(x as i32, (y + 0x20) as i32, self.text.as_ptr())
                    }
                    TextFormat::Medium | TextFormat::MediumCenter => {
                        vexDisplayStringAt(x as i32, (y + 0x20) as i32, self.text.as_ptr())
                    }
                    TextFormat::Large => {
                        vexDisplayBigStringAt(x as i32, (y + 0x20) as i32, self.text.as_ptr())
                    }
                },
                TextPosition::Line(line) => match self.format {
                    TextFormat::Small | TextFormat::Medium => {
                        vexDisplayString(line as i32, self.text.as_ptr())
                    }
                    TextFormat::Large => vexDisplayBigString(line as i32, self.text.as_ptr()),
                    TextFormat::MediumCenter => {
                        vexDisplayCenteredString(line as i32, self.text.as_ptr())
                    }
                    TextFormat::LargeCenter => {
                        vexDisplayBigCenteredString(line as i32, self.text.as_ptr())
                    }
                },
            };
        }
    }
}

/// A touch event on the screen.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

/// The state of a given touch.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TouchState {
    /// The touch has been released.
    Released,
    /// The screen has been touched.
    Pressed,
    /// The touch is still being held.
    Held,
}

impl From<V5_TouchEvent> for TouchState {
    fn from(value: V5_TouchEvent) -> Self {
        match value {
            V5_TouchEvent::kTouchEventPress => Self::Pressed,
            V5_TouchEvent::kTouchEventRelease => Self::Released,
            V5_TouchEvent::kTouchEventPressAuto => Self::Held,
            _ => unreachable!(),
        }
    }
}

/// The rendering mode for the screen.
/// When the screen is on the [`Immediate`](RenderMode::Immediate) mode, all draw calls will immediately show up on the display.
/// The [`DoubleBuffered`](RenderMode::DoubleBuffered) mode instead pushes all draw calls onto an intermediate buffer
/// that can be swapped onto the screen by calling [`Screen::render`].
/// By default the screen uses the [`Immediate`](RenderMode::Immediate) mode.
/// # Note
/// [`Screen::render`] **MUST** be called for anything to appear on the screen when using the [`DoubleBuffered`](RenderMode::DoubleBuffered) mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderMode {
    /// Draw calls are immediately pushed to the screen.
    /// This mode is more convenient because you dont have to call [`Screen::render`] to see anything on the screen.
    Immediate,
    /// Draw calls are pushed to an intermediary buffer which can be pushed to the screen with [`Screen::render`].
    /// This mode is useful for removing screen flicker when drawing at high speeds.
    DoubleBuffered,
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

    /// The amount of time it takes for the brain display to fully re-render.
    /// The brain display is 60fps.
    pub const REFRESH_INTERVAL: Duration = Duration::from_micros(16667);

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
            render_mode: RenderMode::Immediate,
            writer_buffer: String::default(),
        }
    }

    fn flush_writer(&mut self) {
        self.fill(
            &Text::new(
                self.writer_buffer.as_str(),
                TextPosition::Line(self.current_line),
                TextFormat::Medium,
            ),
            Rgb::WHITE,
        );

        self.writer_buffer.clear();
    }

    /// Set the render mode for the screen.
    /// For more info on render modes, look at the [`RenderMode`] docs.
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
        unsafe {
            match mode {
                RenderMode::Immediate => vex_sdk::vexDisplayDoubleBufferDisable(),
                RenderMode::DoubleBuffered => vex_sdk::vexDisplayRender(false, true),
            }
        }
    }

    /// Gets the [`RenderMode`] of the screen.
    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    /// Flushes the screens double buffer if it is enabled.
    /// This is a no-op with the [`Immediate`](RenderMode::Immediate) rendering mode,
    /// but is necessary for anything to be displayed on the screen when using the  [`DoubleBuffered`](RenderMode::DoubleBuffered) mode.
    pub fn render(&mut self) {
        if let RenderMode::DoubleBuffered = self.render_mode {
            unsafe {
                // TODO: create an async function that does the equivalent of bVsyncWait.
                vex_sdk::vexDisplayRender(false, false)
            }
        }
    }

    /// Scroll the entire display buffer.
    ///
    /// This function effectively y-offsets all pixels drawn to the display buffer by
    /// a number (`offset`) of pixels.
    pub fn scroll(&mut self, start: i16, offset: i16) {
        unsafe { vexDisplayScroll(start as i32, offset as i32) }
    }

    /// Scroll a region of the screen.
    ///
    /// This will effectively y-offset the display buffer in this area by
    /// `offset` pixels.
    pub fn scroll_area(&mut self, x0: i16, y0: i16, x1: i16, y1: i16, offset: i16) {
        unsafe {
            vexDisplayScrollRect(
                x0 as i32,
                (y0 + 0x20) as i32,
                x1 as i32,
                (y1 + 0x20) as i32,
                offset as i32,
            )
        }
    }

    /// Draw a filled object to the screen.
    pub fn fill(&mut self, shape: &impl Fill, color: impl IntoRgb) {
        shape.fill(self, color)
    }

    /// Draw an outlined object to the screen.
    pub fn stroke(&mut self, shape: &impl Stroke, color: impl IntoRgb) {
        shape.stroke(self, color)
    }

    /// Wipe the entire display buffer, filling it with a specified color.
    pub fn erase(color: impl IntoRgb) {
        unsafe {
            vexDisplayBackgroundColor(color.into_rgb().into());
            vexDisplayErase();
        };
    }

    /// Draw a color to a specified pixel position on the screen.
    pub fn draw_pixel(x: i16, y: i16) {
        unsafe {
            vexDisplayPixelSet(x as _, (y + 0x20) as _);
        }
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
        let mut raw_buf = buf
            .into_iter()
            .map(|i| i.into_rgb().into())
            .collect::<Vec<_>>();
        // Convert the coordinates to u32 to avoid overflows when multiplying.
        let expected_size = ((x1 - x0) as u32 * (y1 - y0) as u32) as usize;
        if raw_buf.len() != expected_size {
            return Err(ScreenError::BufferSize {
                buffer_size: raw_buf.len(),
                expected_size,
            });
        }

        // SAFETY: The buffer is guaranteed to be the correct size.
        unsafe {
            vexDisplayCopyRect(
                x0 as _,
                y0 as _,
                x1 as _,
                y1 as _,
                raw_buf.as_mut_ptr(),
                src_stride,
            );
        }

        Ok(())
    }

    /// Get the csurrent touch status of the screen.
    pub fn touch_status(&self) -> TouchEvent {
        // vexTouchDataGet (probably) doesn't read from the given status pointer so this is fine.
        let mut touch_status: V5_TouchStatus = unsafe { mem::zeroed() };

        unsafe {
            vexTouchDataGet(&mut touch_status as *mut _);
        }

        TouchEvent {
            state: touch_status.lastEvent.into(),
            x: touch_status.lastXpos,
            y: touch_status.lastYpos,
            press_count: touch_status.pressCount,
            release_count: touch_status.releaseCount,
        }
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with the screen.
pub enum ScreenError {
    /// The given buffer of colors was wrong size to fill the specified area.
    BufferSize {
        /// The size of the buffer.
        buffer_size: usize,
        /// The expected size of the buffer.
        expected_size: usize,
    },
}
