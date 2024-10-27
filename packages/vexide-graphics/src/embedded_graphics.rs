//! Embedded-graphics driver for the V5 Brain display.

use embedded_graphics_core::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};
use vexide_devices::{display::Display, rgb::Rgb};

fn rgb_into_raw(rgb: Rgb<u8>) -> u32 {
    (u32::from(rgb.r) << 16) + (u32::from(rgb.g) << 8) + u32::from(rgb.b)
}

/// An embedded-graphics draw target for the V5 Brain display
/// Currently, this does not support touch detection like the regular [`Display`] API.
pub struct BrainDisplay {
    display: Display,
    triple_buffer:
        [u32; Display::HORIZONTAL_RESOLUTION as usize * Display::VERTICAL_RESOLUTION as usize],
}
impl BrainDisplay {
    /// Create a new [`BrainDisplay`] from a [`Display`].
    /// The display must be moved into this struct,
    /// as it is used to render the display and having multiple mutable references to it is unsafe.
    pub fn new(mut display: Display) -> Self {
        display.set_render_mode(vexide_devices::display::RenderMode::DoubleBuffered);
        Self {
            display,
            triple_buffer: [0; Display::HORIZONTAL_RESOLUTION as usize
                * Display::VERTICAL_RESOLUTION as usize],
        }
    }
}
impl Dimensions for BrainDisplay {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::new(0, 0),
            Size::new(
                Display::HORIZONTAL_RESOLUTION as _,
                Display::VERTICAL_RESOLUTION as _,
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
            .map(|p| (p.0, rgb_into_raw(Rgb::new(p.1.r(), p.1.g(), p.1.b()))))
            .for_each(|(pos, col)| {
                self.triple_buffer
                    [pos.y as usize * Display::HORIZONTAL_RESOLUTION as usize + pos.x as usize] =
                    col;
            });

        unsafe {
            vex_sdk::vexDisplayCopyRect(
                0,
                0x20,
                Display::HORIZONTAL_RESOLUTION as _,
                Display::VERTICAL_RESOLUTION as _,
                self.triple_buffer.as_mut_ptr(),
                Display::HORIZONTAL_RESOLUTION as _,
            );
        };
        self.display.render();

        Ok(())
    }
}
