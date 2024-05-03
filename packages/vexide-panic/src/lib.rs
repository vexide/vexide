//! Panic handler implementation for [`vexide`](https://crates.io/crates/vexide).
//! Supports printing a backtrace when running in the simulator.
//! If the `display_panics` feature is enabled, it will also display the panic message on the V5 Brain display.

#![no_std]

extern crate alloc;

use alloc::string::{String, ToString};

use vexide_core::println;
#[cfg(feature = "display_panics")]
use vexide_devices::{
    color::Rgb,
    geometry::Point2,
    screen::{Rect, Screen, ScreenError, Text, TextSize},
};

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Prints a backtrace to the debug console
    fn sim_log_backtrace();
}

/// Draw an error box to the screen.
///
/// This function is internally used by the vexide panic handler for displaying
/// panic messages graphically before exiting.
#[cfg(feature = "display_panics")]
fn draw_error(screen: &mut Screen, msg: &str) -> Result<(), ScreenError> {
    const ERROR_BOX_MARGIN: i16 = 16;
    const ERROR_BOX_PADDING: i16 = 16;
    const LINE_HEIGHT: i16 = 20;
    const LINE_MAX_WIDTH: usize = 52;

    let error_box_rect = Rect::new(
        Point2 {
            x: ERROR_BOX_MARGIN,
            y: ERROR_BOX_MARGIN,
        },
        Point2 {
            x: Screen::HORIZONTAL_RESOLUTION - ERROR_BOX_MARGIN,
            y: Screen::VERTICAL_RESOLUTION - ERROR_BOX_MARGIN,
        },
    );

    screen.fill(&error_box_rect, Rgb::RED);
    screen.stroke(&error_box_rect, Rgb::WHITE);

    let mut buffer = String::new();
    let mut line: i16 = 0;

    for (i, character) in msg.char_indices() {
        if !character.is_ascii_control() {
            buffer.push(character);
        }

        if character == '\n' || ((buffer.len() % LINE_MAX_WIDTH == 0) && (i > 0)) {
            screen.fill(
                &Text::new(
                    buffer.as_str(),
                    Point2 {
                        x: ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                        y: ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * LINE_HEIGHT),
                    },
                    TextSize::Small,
                ),
                Rgb::WHITE,
            );

            line += 1;
            buffer.clear();
        }
    }

    screen.fill(
        &Text::new(
            buffer.as_str(),
            Point2 {
                x: ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                y: ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * LINE_HEIGHT),
            },
            TextSize::Small,
        ),
        Rgb::WHITE,
    );

    Ok(())
}

#[panic_handler]
/// The panic handler for vexide.
pub fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    println!("{info}");

    unsafe {
        #[cfg(feature = "display_panics")]
        draw_error(&mut Screen::new(), &info.to_string()).unwrap_or_else(|err| {
            println!("Failed to draw error message to screen: {err}");
        });

        #[cfg(target_arch = "wasm32")]
        sim_log_backtrace();

        #[cfg(not(feature = "display_panics"))]
        vexide_core::program::exit();
        // unreachable without display_panics
        #[cfg_attr(not(feature = "display_panics"), allow(unreachable_code))]
        loop {
            // Flush the serial buffer so that the panic message is printed
            vex_sdk::vexTasksRun();
        }
    }
}
