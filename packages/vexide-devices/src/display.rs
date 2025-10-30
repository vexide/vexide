//! Brain Display & Touch Input
//!
//! Contains user calls to the V5 Brain display for touching and displaying graphics.
//!
//! The [`Fill`] trait can be used to draw filled in shapes to the display and the [`Stroke`] trait
//! can be used to draw the outlines of shapes.

use alloc::{borrow::Cow, ffi::CString, string::String};
use core::{
    ffi::{CStr, c_char},
    mem,
    time::Duration,
};

use snafu::{Snafu, ensure};
use vex_sdk::{
    V5_TouchEvent, V5_TouchStatus, vexDisplayBackgroundColor, vexDisplayCircleDraw,
    vexDisplayCircleFill, vexDisplayCopyRect, vexDisplayFontNamedSet, vexDisplayForegroundColor,
    vexDisplayLineDraw, vexDisplayPixelSet, vexDisplayPrintf, vexDisplayRectDraw,
    vexDisplayRectFill, vexDisplayScroll, vexDisplayScrollRect, vexDisplayString,
    vexDisplayStringHeightGet, vexDisplayStringWidthGet, vexDisplayTextSize, vexTouchDataGet,
};

use crate::{
    color::{Rgb, RgbExt},
    math::Point2,
};

/// A struct that implements a fixed width line buffer with a length of
/// 52 characters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LineBuffer {
    idx: usize,
    // We add one to account for the null terminator.
    buf: [u8; Display::LINE_LENGTH + 1],
}

impl Default for LineBuffer {
    fn default() -> Self {
        LineBuffer {
            idx: 0,
            buf: [0; Display::LINE_LENGTH + 1],
        }
    }
}

impl LineBuffer {
    fn buffered(&mut self, s: &str, mut func: impl FnMut(&[u8])) {
        let ascii_chars = s.chars().map(|c| match u32::from(c) {
            1..=127 => c as _,
            _ => b'?',
        });
        for char in ascii_chars {
            if self.idx == Display::LINE_LENGTH || char == b'\n' {
                func(&self.buf[0..=self.idx]);
                *self = Default::default();
            }
            if char != b'\n' {
                self.buf[self.idx] = char;
                self.idx += 1;
            }
        }
    }
}

/// The physical display and touchscreen on a VEX Brain.
#[derive(Debug, Eq, PartialEq)]
pub struct Display {
    writer_buffer: LineBuffer,
    render_mode: RenderMode,
    current_line: usize,
}

impl core::fmt::Write for Display {
    fn write_str(&mut self, text: &str) -> core::fmt::Result {
        let mut writer_buffer = self.writer_buffer;
        writer_buffer.buffered(text, |line| {
            let line = line.as_ptr() as *const c_char;
            if self.current_line > (Self::MAX_VISIBLE_LINES - 2) {
                self.scroll(0, Self::LINE_HEIGHT);
            } else {
                self.current_line += 1;
            }
            Font::default().apply();
            unsafe {
                vexDisplayForegroundColor(0xff_ff_ff);
                vexDisplayString(self.current_line as i32, c"%s".as_ptr(), line);
            }
        });
        self.writer_buffer = writer_buffer;
        Ok(())
    }
}

/// A color stored in ARGB format.
#[repr(C, align(4))]
#[derive(Clone, Copy, Default, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Argb {
    /// Alpha channel
    pub a: u8,
    /// Red channel
    pub r: u8,
    /// Green channel
    pub g: u8,
    /// Blue channel
    pub b: u8,
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
    ///
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
///
/// The width is the same as the pen width.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Line {
    /// Start point (coordinate) of the line
    pub start: Point2<i16>,

    /// End point (coordinate) of the line
    pub end: Point2<i16>,
}

impl Line {
    /// Creates a new line with the given start and endpoints.
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
/// When drawn to the display, both the start and the end points are included inside the drawn
/// region. Thus, the area of the drawn rectangle is `(1 + end.x - start.x) * (1 + end.y - start.y)`
/// pixels.
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

/// Calculates the greatest common divisor of two values using the Euclidean algorithm.
const fn gcd(mut a: i32, mut b: i32) -> i32 {
    while a != b {
        if a > b {
            a -= b;
        } else {
            b -= a;
        }
    }
    a
}

#[allow(clippy::cast_precision_loss)]
fn approximate_fraction(input: f32, precision: u32) -> (i32, i32) {
    // Separate the integral and fractional parts of the input.
    let integral_part = crate::math::floorf(input);
    let fractional_part = input - crate::math::truncf(input);

    // If the fractional part is 0, return the integral part.
    if fractional_part == 0.0 {
        return (integral_part as i32, 1);
    }

    let precision = precision as f32;

    let gcd = gcd(
        crate::math::roundf(fractional_part * precision) as _,
        precision as _,
    );

    let denominator = precision as i32 / gcd;
    let numerator = crate::math::roundf(fractional_part * precision) as i32 / gcd;

    (
        // Add back the integral part to the numerator.
        numerator + integral_part as i32 * denominator,
        denominator,
    )
}

impl FontSize {
    /// Creates a custom fractional font size.
    ///
    /// If you wish to create a font size from a floating-point size, use [`FontSize::from_float`]
    /// instead.
    #[must_use]
    pub const fn new(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    /// Creates a fractional font size from a floating-point size.
    ///
    /// # Precision
    ///
    /// This function is lossy, but negligibly so.
    /// The highest the denominator can be is 10000.
    ///
    /// # Errors
    ///
    /// - [`InvalidFontSizeError`] if the given size is negative.
    pub fn from_float(size: f32) -> Result<Self, InvalidFontSizeError> {
        ensure!(
            size.is_finite() && !size.is_sign_negative(),
            InvalidFontSizeSnafu { value: size }
        );
        let (numerator, denominator) = approximate_fraction(size, 10_000);
        // Unwraps are safe because we guarantee a positive fraction earlier.
        let (numerator, denominator) = (
            numerator.try_into().unwrap(),
            denominator.try_into().unwrap(),
        );
        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// An extra-small font size with a value of one-fifth.
    pub const EXTRA_SMALL: Self = Self::new(1, 5);
    /// A small font size with a value of one-fourth.
    pub const SMALL: Self = Self::new(1, 4);
    /// A medium font size with a value of one-third.
    pub const MEDIUM: Self = Self::new(1, 3);
    /// A large font size with a value of one-half.
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

impl TryFrom<f32> for FontSize {
    type Error = InvalidFontSizeError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::from_float(value)
    }
}

impl TryFrom<f64> for FontSize {
    type Error = InvalidFontSizeError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::from_float(value as f32)
    }
}

impl From<u32> for FontSize {
    fn from(value: u32) -> Self {
        Self::new(value, 1)
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
pub struct Text<'a> {
    /// Top left corner coordinates of text on the display
    position: Point2<i16>,
    /// C-String of the desired text to be displayed on the display
    text: Cow<'a, CStr>,
    /// The font that will be used when this text is displayed
    font: Font,
    /// Horizontal alignment of text displayed on the display
    horizontal_align: HAlign,
    /// Vertical alignment of text displayed on the display
    vertical_align: VAlign,
}

impl<'a> Text<'a> {
    /// Create a new text from a &CStr with a given position (defaults to top left corner alignment)
    /// and font
    pub fn new(text: &'a CStr, font: Font, position: impl Into<Point2<i16>>) -> Self {
        Self::new_aligned(text, font, position, HAlign::default(), VAlign::default())
    }

    /// Create a new text from a String with a given position (defaults to top left corner
    /// alignment) and font
    ///
    /// # Panics
    ///
    /// This function panics if `text` contains a null character.
    pub fn from_str(text: String, font: Font, position: impl Into<Point2<i16>>) -> Self {
        Self::from_str_aligned(text, font, position, HAlign::default(), VAlign::default())
    }

    /// Create a new text from a &CStr with a given position (based on alignment) and font
    pub fn new_aligned(
        text: &'a CStr,
        font: Font,
        position: impl Into<Point2<i16>>,
        horizontal_align: HAlign,
        vertical_align: VAlign,
    ) -> Self {
        Self {
            text: text.into(),
            position: position.into(),
            font,
            horizontal_align,
            vertical_align,
        }
    }

    /// Create a new text from a String with a given position (based on alignment) and font
    ///
    /// # Panics
    ///
    /// This function panics if `text` contains a null character.
    pub fn from_str_aligned(
        text: String,
        font: Font,
        position: impl Into<Point2<i16>>,
        horizontal_align: HAlign,
        vertical_align: VAlign,
    ) -> Self {
        Self {
            text: CString::new(text)
                .expect("Null character in non-terminating position")
                .into(),
            position: position.into(),
            font,
            horizontal_align,
            vertical_align,
        }
    }

    /// Change text alignment
    pub const fn align(&mut self, horizontal_align: HAlign, vertical_align: VAlign) {
        self.horizontal_align = horizontal_align;
        self.vertical_align = vertical_align;
    }

    /// Returns the height of the text widget in pixels
    #[must_use]
    pub fn height(&self) -> u16 {
        self.font.apply();
        unsafe { vexDisplayStringHeightGet(self.text.as_ptr()) as _ }
    }

    /// Returns the width of the text widget in pixels
    #[must_use]
    pub fn width(&self) -> u16 {
        self.font.apply();
        unsafe { vexDisplayStringWidthGet(self.text.as_ptr()) as _ }
    }
}

impl Text<'_> {
    /// Write the text to the display.
    ///
    /// # Arguments
    ///
    /// - `display` - The display to write the text to.
    /// - `color` - The color of the text.
    /// - `bg_color` - The background color of the text. If `None`, the background will be
    ///   transparent.
    pub fn draw(
        &self,
        _display: &mut Display,
        color: impl Into<Rgb<u8>>,
        bg_color: Option<Rgb<u8>>,
    ) {
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
            let bg_is_some = bg_color.is_some();
            if bg_is_some {
                vexDisplayBackgroundColor(bg_color.unwrap().into_raw());
            }
            self.font.apply();
            vexDisplayPrintf(
                i32::from(x),
                i32::from(y + Display::HEADER_HEIGHT),
                i32::from(bg_is_some),
                c"%s".as_ptr(),
                self.text.as_ptr(),
            );
        }
    }
}

/// A touch event on the display.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TouchEvent {
    /// Touch state (pressed, released, held).
    pub state: TouchState,

    /// Point at which the display was touched.
    pub point: Point2<i16>,

    /// Number of times the display has been pressed.
    pub press_count: i32,

    /// Number of times the display has been released.
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
/// When using the display in the [`Immediate`](RenderMode::Immediate) mode, all draw operations
/// will immediately show up on the display. The [`DoubleBuffered`](RenderMode::DoubleBuffered) mode
/// instead applies draw operations onto an intermediate buffer that can be swapped onto the display
/// by calling [`Display::render`], thereby preventing screen tearing. By default, the display uses
/// the [`Immediate`](RenderMode::Immediate) mode.
///
/// # Note
///
/// [`Display::render`] **MUST** be called for anything to appear on the display when using the
/// [`DoubleBuffered`](RenderMode::DoubleBuffered) mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderMode {
    /// Draw operations are immediately applied to the display without the need to call
    /// [`Display::render`].
    Immediate,
    /// Draw calls are affected on an intermediary display buffer, rather than directly drawn to
    /// the display. The intermediate buffer can later be applied to the display using
    /// [`Display::render`]
    ///
    /// This mode is necessary for preventing screen tearing when drawing at high speeds.
    DoubleBuffered,
}

impl Display {
    /// The maximum number of lines that can be visible on the display at once.
    pub(crate) const MAX_VISIBLE_LINES: usize = 12;

    /// The height of a single line of text on the display.
    pub(crate) const LINE_HEIGHT: i16 = 20;

    /// The number of characters that fit in one line with the default font.
    pub(crate) const LINE_LENGTH: usize = 20;

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
    /// Creating new `display`s is inherently unsafe due to the possibility of constructing more
    /// than one display at once allowing multiple mutable references to the same
    /// hardware device. Prefer using [`Peripherals`](crate::peripherals::Peripherals) to register
    /// devices if possible.
    #[must_use]
    pub unsafe fn new() -> Self {
        Self {
            current_line: 0,
            render_mode: RenderMode::Immediate,
            writer_buffer: Default::default(),
        }
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
    ///
    /// This is a no-op with the [`Immediate`](RenderMode::Immediate) rendering mode,
    /// but is necessary for anything to be displayed on the displayed when using the
    /// [`DoubleBuffered`](RenderMode::DoubleBuffered) mode.
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
    /// This function y-offsets the pixels in the display buffer which are at or below the given
    /// start point (`start`) by a number (`offset`) of pixels. Positive values move the pixels
    /// upwards, and pixels that are moved out of the scroll region are discarded. Empty spaces
    /// are then filled with the display's background color.
    pub fn scroll(&mut self, start: i16, offset: i16) {
        unsafe { vexDisplayScroll(start.into(), offset.into()) }
    }

    /// Scroll a region of the display.
    ///
    /// This function y-offsets the pixels in the display buffer which are contained in the
    /// specified scroll region (`region`) by a number (`offset`) of pixels. Positive values
    /// move the pixels upwards, and pixels that are moved out of the scroll region are
    /// discarded. Empty spaces are then filled with the display's background color.
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

    /// Draws a filled shape to the display with the specified color.
    ///
    /// Any type implementing the [`Fill`] trait (such as [`Rect`] or [`Circle`]) may be drawn using
    /// this method.
    pub fn fill(&mut self, shape: &impl Fill, color: impl Into<Rgb<u8>>) {
        shape.fill(self, color);
    }

    /// Draws an outlined shape to the display with the specified color.
    ///
    /// Any type implementing the [`Stroke`] trait (such as [`Rect`] or [`Circle`]) may be drawn
    /// using this method.
    pub fn stroke(&mut self, shape: &impl Stroke, color: impl Into<Rgb<u8>>) {
        shape.stroke(self, color);
    }

    /// Draws a line of text with the specified color and background color to the display.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::{
    ///     color::Rgb,
    ///     display::{Font, FontFamily, FontSize, Text},
    ///     math::Point2,
    ///     prelude::*,
    /// };
    ///
    /// let mut peripherals = Peripherals::take().unwrap();
    /// let mut display = peripherals.display;
    ///
    /// // Create a new text widget.
    /// let text = Text::new(
    ///     c"Hello, World!",
    ///     Font::new(FontSize::MEDIUM, FontFamily::Monospace),
    ///     Point2 { x: 10, y: 10 },
    /// );
    ///
    /// // Write red text with a blue background to the display.
    /// display.draw_text(&text, Rgb::new(255, 0, 0), Some(Rgb::new(0, 0, 255)));
    /// ```
    pub fn draw_text(
        &mut self,
        text: &Text<'_>,
        color: impl Into<Rgb<u8>>,
        bg_color: Option<Rgb<u8>>,
    ) {
        text.draw(self, color, bg_color);
    }

    /// Clears the entire display, filling it with the specified color.
    pub fn erase(&mut self, color: impl Into<Rgb<u8>>) {
        // We don't use `vexDisplayErase` here because it doesn't take a color
        // and we want to preserve the API.
        Rect::from_dimensions(
            Point2 { x: 0, y: 0 },
            // Technically max x/y position is one pixel less than this and the
            // SDK that Rect::fill uses inclusive coordinates, but it doesn't
            // matter that much and this way we don't have to do any
            // subtraction (though it would probably be inlined anyway).
            Display::HORIZONTAL_RESOLUTION as _,
            Display::VERTICAL_RESOLUTION as _,
        )
        .fill(self, color);
    }

    /// Draws a buffer of pixels to a specified region of the display.
    ///
    /// This function copies the pixels in the specified buffer to the specified region of the
    /// display.
    ///
    /// # Panics
    ///
    /// This function panics if `buf` does not have the correct number of bytes to fill the
    /// specified region.
    pub fn draw_buffer(&mut self, region: Rect, buf: &[Argb]) {
        let raw_buf: &[u32] = bytemuck::must_cast_slice(buf);
        // Convert the coordinates to u32 to avoid overflows when multiplying.
        let expected_size = ((region.end.y - region.start.y) as u32
            * (region.end.x - region.start.x) as u32) as usize;
        let buffer_size = raw_buf.len();
        assert_eq!(
            buffer_size, expected_size,
            "The given buffer of colors was wrong size to fill the specified area: expected {expected_size} bytes, got {buffer_size}."
        );

        // SAFETY: The buffer is guaranteed to be the correct size.
        unsafe {
            vexDisplayCopyRect(
                i32::from(region.start.x),
                i32::from(region.start.y + Self::HEADER_HEIGHT),
                i32::from(region.end.x),
                i32::from(region.end.y + Self::HEADER_HEIGHT),
                raw_buf.as_ptr() as *mut _,
                i32::from(region.end.x - region.start.x),
            );
        }
    }

    /// Returns the last recorded state of the display's touchscreen.
    ///
    /// See [`TouchEvent`] for more information.
    #[must_use]
    pub fn touch_status(&self) -> TouchEvent {
        // `vexTouchDataGet` (probably) doesn't read from the given status pointer, so this is fine.
        let mut touch_status: V5_TouchStatus = unsafe { mem::zeroed() };

        unsafe {
            vexTouchDataGet(&raw mut touch_status);
        }

        TouchEvent {
            state: touch_status.lastEvent.into(),
            point: Point2 {
                x: touch_status.lastXpos,
                y: touch_status.lastYpos,
            },
            press_count: touch_status.pressCount,
            release_count: touch_status.releaseCount,
        }
    }
}

/// An error that occurs when a negative or non-finite font size is attempted to be created.
#[derive(Debug, Clone, Copy, PartialEq, Snafu)]
#[snafu(display("Attempted to create a font size with a negative/non-finite value ({value})."))]
pub struct InvalidFontSizeError {
    /// The negative value that was attempted to be used as a font size.
    pub value: f32,
}
