//! Embedded-graphics driver for the V5 Brain screen.

use embedded_graphics_core::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};
use vexide_devices::{color::Rgb, screen::Screen};

/// An embedded-graphics draw target for the V5 brain screen
/// Currently, this does not support touch detection like the regular [`Screen`] API.
pub struct BrainDisplay {
    screen: Screen,
    triple_buffer:
        [u32; Screen::HORIZONTAL_RESOLUTION as usize * Screen::VERTICAL_RESOLUTION as usize],
}
impl BrainDisplay {
    /// Create a new [`BrainDisplay`] from a [`Screen`].
    /// The screen must be moved into this struct,
    /// as it is used to render the display and having multiple mutable references to it is unsafe.
    pub fn new(mut screen: Screen) -> Self {
        screen.set_render_mode(vexide_devices::screen::RenderMode::DoubleBuffered);
        Self {
            screen,
            triple_buffer: [0; Screen::HORIZONTAL_RESOLUTION as usize
                * Screen::VERTICAL_RESOLUTION as usize],
        }
    }
}
impl Dimensions for BrainDisplay {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::new(0, 0),
            Size::new(
                Screen::HORIZONTAL_RESOLUTION as _,
                Screen::VERTICAL_RESOLUTION as _,
            ),
        )
    }
}
impl DrawTarget for BrainDisplay {
    type Color = Rgb888;

    type Error = !;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        pixels
            .into_iter()
            .map(|p| (p.0, Rgb::new(p.1.r(), p.1.g(), p.1.b()).into()))
            .for_each(|(pos, col)| {
                self.triple_buffer
                    [pos.y as usize * Screen::HORIZONTAL_RESOLUTION as usize + pos.x as usize] =
                    col;
            });

        unsafe {
            vex_sdk::vexDisplayCopyRect(
                0,
                0x20,
                Screen::HORIZONTAL_RESOLUTION as _,
                Screen::VERTICAL_RESOLUTION as _,
                self.triple_buffer.as_mut_ptr(),
                Screen::HORIZONTAL_RESOLUTION as _,
            );
        };
        self.screen.render();

        Ok(())
    }
}
