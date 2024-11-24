//! Brain Display & Touch Input
//!
//! Contains user calls to the V5 Brain display for touching and displaying graphics.
//! The [`Fill`] trait can be used to draw filled in shapes to the display
//! and the [`Stroke`] trait can be used to draw the outlines of shapes.

use alloc::{ffi::CString, string::String, vec::Vec};
use core::{ffi::CStr, mem, ptr::addr_of_mut, time::Duration};

use snafu::{ensure, Snafu};
use vex_sdk::{
    vexDisplayBackgroundColor, vexDisplayCircleDraw, vexDisplayCircleFill, vexDisplayCopyRect,
    vexDisplayErase, vexDisplayFontNamedSet, vexDisplayForegroundColor, vexDisplayLineDraw,
    vexDisplayPixelSet, vexDisplayPrintf, vexDisplayRectDraw, vexDisplayRectFill, vexDisplayScroll,
    vexDisplayScrollRect, vexDisplayString, vexDisplayStringHeightGet, vexDisplayStringWidthGet,
    vexDisplayTextSize, vexTouchDataGet, V5_TouchEvent, V5_TouchStatus,
};

use crate::{
    math::Point2,
    rgb::{Rgb, RgbExt},
};

/// Represents the physical display on the V5 Brain.
#[derive(Debug, Eq, PartialEq)]
pub struct Display {
    writer_buffer: String,
    render_mode: RenderMode,
    current_line: usize,
}

impl core::fmt::Write for Display {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        for character in text.chars() {
            if character == '\n' {
                if self.current_line > (Self::MAX_VISIBLE_LINES - 2) {
                    self.scroll(0, Self::LINE_HEIGHT);
                    self.flush_writer();
                } else {
                    self.flush_writer();
                    self.current_line += 1;
                }
            } else {
                self.writer_buffer.push(character);
            }
        }

        unsafe {
            vexDisplayForegroundColor(0xff_ff_ff);
            vexDisplayString(
                self.current_line as i32,
                c"%s".as_ptr(),
                CString::new(self.writer_buffer.clone())
                    .expect(
                        "CString::new encountered NUL (U+0000) byte in non-terminating position.",
                    )
                    .into_raw(),
            );
        }

        Ok(())
    }
}

/// A type implementing this trait can draw a filled shape to the display.
pub trait Fill {
    /// Draw a filled shape to the display.
    fn fill(&self, display: &mut Display, color: impl Into<Rgb<u8>>);
}

/// A type implementing this trait can draw an outlined shape to the display.
pub trait Stroke {
    /// Draw an outlined shape to the display.
    fn stroke(&self, display: &mut Display, color: impl Into<Rgb<u8>>);
}

/// A circle that can be drawn on the  display.
///
/// Circles are not antialiased.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Circle {
    /// Center point of the circle
    pub center: Point2<i16>,

    /// Radius of the circle
    pub radius: u16,
}

impl Circle {
    /// Create a circle with the given coordinates and radius.
    /// The coordinates are the center of the circle.
    pub fn new(center: impl Into<Point2<i16>>, radius: u16) -> Self {
        Self {
            center: center.into(),
            radius,
        }
    }
}

impl Fill for Circle {
    fn fill(&self, _display: &mut Display, color: impl Into<Rgb<u8>>) {
        unsafe {
            vexDisplayForegroundColor(color.into().into_raw());
            vexDisplayCircleFill(
                i32::from(self.center.x),
                i32::from(self.center.y + Display::HEADER_HEIGHT),
                i32::from(self.radius),
            );
        }
    }
}

impl Stroke for Circle {
    fn stroke(&self, _display: &mut Display, color: impl Into<Rgb<u8>>) {
        unsafe {
            vexDisplayForegroundColor(color.into().into_raw());
            vexDisplayCircleDraw(
                i32::from(self.center.x),
                i32::from(self.center.y + Display::HEADER_HEIGHT),
                i32::from(self.radius),
            );
        }
    }
}

/// A line that can be drawn on the display.
/// The width is the same as the pen width.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Line {
    /// Start point (coordinate) of the line
    pub start: Point2<i16>,

    /// End point (coordinate) of the line
    pub end: Point2<i16>,
}

impl Line {
    /// Create a new line with a given start and end coordinate.
    pub fn new(start: impl Into<Point2<i16>>, end: impl Into<Point2<i16>>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

impl Fill for Line {
    fn fill(&self, _display: &mut Display, color: impl Into<Rgb<u8>>) {
        unsafe {
            vexDisplayForegroundColor(color.into().into_raw());
            vexDisplayLineDraw(
                i32::from(self.start.x),
                i32::from(self.start.y + Display::HEADER_HEIGHT),
                i32::from(self.end.x),
                i32::from(self.end.y + Display::HEADER_HEIGHT),
            );
        }
    }
}

impl<T: Into<Point2<i16>> + Copy> Fill for T {
    fn fill(&self, _display: &mut Display, color: impl Into<Rgb<u8>>) {
        let point: Point2<i16> = (*self).into();

        unsafe {
            vexDisplayForegroundColor(color.into().into_raw());
            vexDisplayPixelSet(point.x as _, (point.y + Display::HEADER_HEIGHT) as _);
        }
    }
}

/// A rectangular region of the display.
///
/// When drawn to the display, both the start and the end points are included inside
/// the drawn region. Thus, the area of the drawn rectangle is
/// `(1 + end.x - start.x) * (1 + end.y - start.y)` pixels.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Rect {
    /// First point (coordinate) of the rectangle
    pub start: Point2<i16>,

    /// Second point (coordinate) of the rectangle
    pub end: Point2<i16>,
}

impl Rect {
    /// Create a new rectangle with the given coordinates.
    pub fn new(start: impl Into<Point2<i16>>, end: impl Into<Point2<i16>>) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }

    /// Create a new rectangle from a given origin point (top-left) and dimensions (width/height).
    pub fn from_dimensions(origin: impl Into<Point2<i16>>, width: u16, height: u16) -> Self {
        let origin = origin.into();
        Self {
            start: origin,
            end: Point2 {
                x: origin.x + (width as i16),
                y: origin.y + (height as i16),
            },
        }
    }

    /// Create a new rectangle from a given origin point (top-left) and dimensions (width/height).
    pub fn from_dimensions_centered(
        center: impl Into<Point2<i16>>,
        width: u16,
        height: u16,
    ) -> Self {
        let center = center.into();

        Self::from_dimensions(
            Point2 {
                x: center.x - (width as i16) / 2,
                y: center.y - (height as i16) / 2,
            },
            width,
            height,
        )
    }
}

impl Stroke for Rect {
    fn stroke(&self, _display: &mut Display, color: impl Into<Rgb<u8>>) {
        unsafe {
            vexDisplayForegroundColor(color.into().into_raw());
            vexDisplayRectDraw(
                i32::from(self.start.x),
                i32::from(self.start.y + Display::HEADER_HEIGHT),
                i32::from(self.end.x),
                i32::from(self.end.y + Display::HEADER_HEIGHT),
            );
        }
    }
}

impl Fill for Rect {
    fn fill(&self, _display: &mut Display, color: impl Into<Rgb<u8>>) {
        unsafe {
            vexDisplayForegroundColor(color.into().into_raw());
            vexDisplayRectFill(
                i32::from(self.start.x),
                i32::from(self.start.y + Display::HEADER_HEIGHT),
                i32::from(self.end.x),
                i32::from(self.end.y + Display::HEADER_HEIGHT),
            );
        }
    }
}

/// Options for how a text object should be formatted.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct Font {
    /// The size of the font.
    pub size: FontSize,
    /// The font family of the font.
    pub family: FontFamily,
}

impl Font {
    /// Create a new font with a given size and family.
    #[must_use]
    pub const fn new(size: FontSize, family: FontFamily) -> Self {
        Self { size, family }
    }

    /// Set the display's font to this font.
    fn apply(self) {
        unsafe {
            vexDisplayFontNamedSet(self.family.raw().as_ptr());
            vexDisplayTextSize(self.size.numerator, self.size.denominator);
        }
    }
}

/// A fractional font scaling factor.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct FontSize {
    /// The numerator of the fractional font scale.
    pub numerator: u32,
    /// The denominator of the fractional font scale.
    pub denominator: u32,
}

impl FontSize {
    /// Create a custom fractional font size.
    #[must_use]
    pub const fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// An extra-small font size with a value of one-fifth.
    pub const EXTRA_SMALL: Self = Self::new(1, 5);
    /// A small font size with a value of one-fourth.
    pub const SMALL: Self = Self::new(1, 4);
    /// A medium font size with a value of one-third.
    pub const MEDIUM: Self = Self::new(1, 3);
    /// A medium font size with a value of one-half.
    pub const LARGE: Self = Self::new(1, 2);
    /// An extra-large font size with a value of two-thirds.
    pub const EXTRA_LARGE: Self = Self::new(2, 3);
    /// The full size of the font.
    pub const FULL: Self = Self::new(1, 1);
}

impl Default for FontSize {
    fn default() -> Self {
        Self::MEDIUM
    }
}

/// The font family of a text object.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum FontFamily {
    /// A monospaced font which has a fixed width for each character.
    ///
    /// This font at full size is 49pt Noto Mono.
    #[default]
    Monospace,
    /// A proportional font which has a varying width for each character.
    ///
    /// This font at full size is 49pt Noto Sans.
    Proportional,
}

impl FontFamily {
    #[must_use]
    const fn raw(self) -> &'static CStr {
        match self {
            FontFamily::Monospace => c"monospace",
            FontFamily::Proportional => c"proportional",
        }
    }
}

/// Horizontal alignment for text on the display
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum HAlign {
    /// Input coordinate is at the left of the text box
    #[default]
    Left,
    /// Input coordinate is at the center of the text box
    Center,
    /// Input coordinate is at the right of the text box
    Right,
}

/// Vertical alignment for text on the display
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum VAlign {
    /// Input coordinate is at the top of the text box
    #[default]
    Top,
    /// Input coordinate is at the center of the text box
    Center,
    /// Input coordinate is at the bottom of the text box
    Bottom,
}

/// A piece of text that can be drawn on the display.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Text {
    /// Top left corner coordinates of text on the display
    pub position: Point2<i16>,
    /// C-String of the desired text to be displayed on the display
    pub text: CString,
    /// The font that will be used when this text is displayed
    pub font: Font,
    /// Horizontal alignment of text displayed on the display
    pub horizontal_align: HAlign,
    /// Vertical alignment of text displayed on the display
    pub vertical_align: VAlign,
}

impl Text {
    /// Create a new text with a given position (defaults to top left corner alignment) and font
    pub fn new(text: &str, font: Font, position: impl Into<Point2<i16>>) -> Self {
        Self::new_aligned(text, font, position, HAlign::default(), VAlign::default())
    }

    /// Create a new text with a given position (based on alignment) and font
    pub fn new_aligned(
        text: &str,
        font: Font,
        position: impl Into<Point2<i16>>,
        horizontal_align: HAlign,
        vertical_align: VAlign,
    ) -> Self {
        Self {
            text: CString::new(text)
                .expect("CString::new encountered NUL (U+0000) byte in non-terminating position."),
            position: position.into(),
            font,
            horizontal_align,
            vertical_align,
        }
    }

    /// Change text alignment
    pub fn align(&mut self, horizontal_align: HAlign, vertical_align: VAlign) {
        self.horizontal_align = horizontal_align;
        self.vertical_align = vertical_align;
    }

    /// Returns the height of the text widget in pixels
    #[must_use]
    pub fn height(&self) -> u16 {
        unsafe {
            self.font.apply();
            vexDisplayStringHeightGet(self.text.as_ptr()) as _
        }
    }

    /// Returns the width of the text widget in pixels
    #[must_use]
    pub fn width(&self) -> u16 {
        unsafe {
            self.font.apply();
            vexDisplayStringWidthGet(self.text.as_ptr()) as _
        }
    }
}

impl Fill for Text {
    fn fill(&self, _display: &mut Display, color: impl Into<Rgb<u8>>) {
        // Horizontally align text
        let x = match self.horizontal_align {
            HAlign::Left => self.position.x,
            HAlign::Center => self.position.x - (self.width() / 2) as i16,
            HAlign::Right => self.position.x - self.width() as i16,
        };

        // Vertically align text
        let y = match self.vertical_align {
            VAlign::Top => self.position.y,
            VAlign::Center => self.position.y - (self.height() / 2) as i16,
            VAlign::Bottom => self.position.y - self.height() as i16,
        };

        unsafe {
            vexDisplayForegroundColor(color.into().into_raw());
            self.font.apply();
            vexDisplayPrintf(
                i32::from(x),
                i32::from(y + Display::HEADER_HEIGHT),
                1,
                c"%s".as_ptr(),
                self.text.as_ptr(),
            );
        }
    }
}

/// A touch event on the display.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TouchEvent {
    /// Touch state.
    pub state: TouchState,
    /// X coordinate of the touch.
    pub x: i16,
    /// Y coordinate of the touch.
    pub y: i16,
    /// how many times the display has been pressed.
    pub press_count: i32,
    /// how many times the display has been released.
    pub release_count: i32,
}

/// The state of a given touch.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TouchState {
    /// The touch has been released.
    Released,
    /// The display has been touched.
    Pressed,
    /// The display has been touched and is still being held.
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

/// The rendering mode for the VEX V5's display
///
/// When using the display in the [`Immediate`](RenderMode::Immediate) mode, all draw operations will immediately show up on the display.
/// The [`DoubleBuffered`](RenderMode::DoubleBuffered) mode instead applies draw operations onto an intermediate buffer
/// that can be swapped onto the display by calling [`Display::render`], thereby preventing screen tearing.
/// By default, the display uses the [`Immediate`](RenderMode::Immediate) mode.
/// # Note
/// [`Display::render`] **MUST** be called for anything to appear on the display when using the [`DoubleBuffered`](RenderMode::DoubleBuffered) mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderMode {
    /// Draw operations are immediately applied to the display without the need to call [`Display::render`].
    Immediate,
    /// Draw calls are affected on an intermediary display buffer, rather than directly drawn to the display.
    /// The intermediate buffer can later be applied to the display using [`Display::render`]
    ///
    /// This mode is necessary for preventing screen tearing when drawing at high speeds.
    DoubleBuffered,
}

impl Display {
    /// The maximum number of lines that can be visible on the display at once.
    pub(crate) const MAX_VISIBLE_LINES: usize = 12;

    /// The height of a single line of text on the display.
    pub(crate) const LINE_HEIGHT: i16 = 20;

    /// Vertical height taken by the user program header when visible.
    pub const HEADER_HEIGHT: i16 = 32;

    /// The horizontal resolution of the display.
    pub const HORIZONTAL_RESOLUTION: i16 = 480;

    /// The vertical resolution of the writable part of the display.
    pub const VERTICAL_RESOLUTION: i16 = 240;

    /// The amount of time it takes for the Brain display to fully re-render.
    /// The Brain display is 60fps.
    pub const REFRESH_INTERVAL: Duration = Duration::from_micros(16667);

    /// Create a new display.
    ///
    /// # Safety
    ///
    /// Creating new `display`s is inherently unsafe due to the possibility of constructing
    /// more than one display at once allowing multiple mutable references to the same
    /// hardware device. Prefer using [`Peripherals`](crate::peripherals::Peripherals) to register devices if possible.
    #[must_use]
    pub unsafe fn new() -> Self {
        Self {
            current_line: 0,
            render_mode: RenderMode::Immediate,
            writer_buffer: String::default(),
        }
    }

    fn flush_writer(&mut self) {
        unsafe {
            vexDisplayForegroundColor(0xff_ff_ff);
            vexDisplayString(
                self.current_line as i32,
                c"%s".as_ptr(),
                CString::new(self.writer_buffer.clone())
                    .expect(
                        "CString::new encountered NUL (U+0000) byte in non-terminating position.",
                    )
                    .into_raw(),
            );
        }

        self.writer_buffer.clear();
    }

    /// Set the render mode for the display.
    ///
    /// For more info on render modes, see [`RenderMode`].
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
        unsafe {
            match mode {
                RenderMode::Immediate => vex_sdk::vexDisplayDoubleBufferDisable(),
                RenderMode::DoubleBuffered => vex_sdk::vexDisplayRender(false, true),
            }
        }
    }

    /// Returns the current [`RenderMode`] of the display.
    #[must_use]
    pub const fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    /// Flushes the displays double buffer if it is enabled.
    /// This is a no-op with the [`Immediate`](RenderMode::Immediate) rendering mode,
    /// but is necessary for anything to be displayed on the displayed when using the [`DoubleBuffered`](RenderMode::DoubleBuffered) mode.
    pub fn render(&mut self) {
        if let RenderMode::DoubleBuffered = self.render_mode {
            unsafe {
                // TODO: create an async function that does the equivalent of `bVsyncWait`.
                vex_sdk::vexDisplayRender(false, false);
            }
        }
    }

    /// Scroll the pixels at or below the specified y-coordinate.
    ///
    /// This function y-offsets the pixels in the display buffer which are at or below the given start point (`start`) by
    /// a number (`offset`) of pixels. Positive values move the pixels upwards, and pixels that are moved out of the scroll
    /// region are discarded. Empty spaces are then filled with the display's background color.
    pub fn scroll(&mut self, start: i16, offset: i16) {
        unsafe { vexDisplayScroll(start.into(), offset.into()) }
    }

    /// Scroll a region of the display.
    ///
    /// This function y-offsets the pixels in the display buffer which are contained in the specified scroll region (`region`) by
    /// a number (`offset`) of pixels. Positive values move the pixels upwards, and pixels that are moved out of the scroll
    /// region are discarded. Empty spaces are then filled with the display's background color.
    pub fn scroll_region(&mut self, region: Rect, offset: i16) {
        unsafe {
            vexDisplayScrollRect(
                i32::from(region.start.x),
                i32::from(region.start.y + Self::HEADER_HEIGHT),
                (region.end.x).into(),
                i32::from(region.end.y + Self::HEADER_HEIGHT),
                i32::from(offset),
            );
        }
    }

    /// Draw a filled object to the display.
    pub fn fill(&mut self, shape: &impl Fill, color: impl Into<Rgb<u8>>) {
        shape.fill(self, color);
    }

    /// Draw an outlined object to the display.
    pub fn stroke(&mut self, shape: &impl Stroke, color: impl Into<Rgb<u8>>) {
        shape.stroke(self, color);
    }

    /// Wipe the entire display buffer, filling it with a specified color.
    pub fn erase(&mut self, color: impl Into<Rgb<u8>>) {
        unsafe {
            vexDisplayBackgroundColor(color.into().into_raw());
            vexDisplayErase();
        };
    }

    /// Draw a buffer of pixels to a specified region of the display.
    ///
    /// This function copies the pixels in the specified buffer to the specified region of the display.
    /// The stride parameter is defined as the number of pixels per row.
    ///
    /// # Errors
    ///
    /// A [`DisplayError::BufferSize`] error is returned if `buf` does not have the correct number of bytes
    /// to fill the specified region.
    pub fn draw_buffer<T, I>(
        &mut self,
        region: Rect,
        buf: T,
        src_stride: i32,
    ) -> Result<(), DisplayError>
    where
        T: IntoIterator<Item = I>,
        I: Into<Rgb<u8>>,
    {
        let mut raw_buf = buf
            .into_iter()
            .map(|i| i.into().into_raw())
            .collect::<Vec<_>>();
        // Convert the coordinates to u32 to avoid overflows when multiplying.
        let expected_size = ((region.end.x - region.start.x) as u32
            * (region.end.y - region.start.y) as u32) as usize;

        ensure!(
            raw_buf.len() == expected_size,
            BufferSizeSnafu {
                buffer_size: raw_buf.len(),
                expected_size
            }
        );

        // SAFETY: The buffer is guaranteed to be the correct size.
        unsafe {
            vexDisplayCopyRect(
                i32::from(region.start.x),
                i32::from(region.start.y + Self::HEADER_HEIGHT),
                i32::from(region.end.x),
                i32::from(region.end.y + Self::HEADER_HEIGHT),
                raw_buf.as_mut_ptr(),
                src_stride,
            );
        }

        Ok(())
    }

    /// Returns the current touch status of the display.
    #[must_use]
    pub fn touch_status(&self) -> TouchEvent {
        // `vexTouchDataGet` (probably) doesn't read from the given status pointer, so this is fine.
        let mut touch_status: V5_TouchStatus = unsafe { mem::zeroed() };

        unsafe {
            vexTouchDataGet(addr_of_mut!(touch_status));
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
/// Errors that can occur when interacting with the display.
pub enum DisplayError {
    /// The given buffer of colors was wrong size to fill the specified area.
    #[snafu(display(
        "The given buffer of colors was wrong size to fill the specified area: expected {expected_size} bytes, got {buffer_size}."
    ))]
    BufferSize {
        /// The size of the buffer.
        buffer_size: usize,
        /// The expected size of the buffer.
        expected_size: usize,
    },
}
