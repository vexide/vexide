//! Panic handler implementation for [`pros-rs`](https://crates.io/crates/pros-rs).
//! Supports printing a backtrace when running in the simulator.
//! If the `display_panics` feature is enabled, it will also display the panic message on the V5 Brain display.

#![no_std]

extern crate alloc;

use alloc::{format, string::String};

use pros_core::eprintln;
#[cfg(feature = "display_panics")]
use pros_devices::Screen;

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Prints a backtrace to the debug console
    fn sim_log_backtrace();
}

/// Draw an error box to the screen.
///
/// This function is internally used by the pros-rs panic handler for displaying
/// panic messages graphically before exiting.
#[cfg(feature = "display_panics")]
fn draw_error(
    screen: &mut pros_devices::screen::Screen,
    msg: &str,
) -> Result<(), pros_devices::screen::ScreenError> {
    const ERROR_BOX_MARGIN: i16 = 16;
    const ERROR_BOX_PADDING: i16 = 16;
    const LINE_MAX_WIDTH: usize = 52;

    let error_box_rect = pros_devices::screen::Rect::new(
        ERROR_BOX_MARGIN,
        ERROR_BOX_MARGIN,
        pros_devices::screen::SCREEN_HORIZONTAL_RESOLUTION - ERROR_BOX_MARGIN,
        pros_devices::screen::SCREEN_VERTICAL_RESOLUTION - ERROR_BOX_MARGIN,
    );

    screen.fill(&error_box_rect, pros_devices::color::Rgb::RED)?;
    screen.stroke(&error_box_rect, pros_devices::color::Rgb::WHITE)?;

    let mut buffer = String::new();
    let mut line: i16 = 0;

    for (i, character) in msg.char_indices() {
        if !character.is_ascii_control() {
            buffer.push(character);
        }

        if character == '\n' || ((buffer.len() % LINE_MAX_WIDTH == 0) && (i > 0)) {
            screen.fill(
                &pros_devices::screen::Text::new(
                    buffer.as_str(),
                    pros_devices::screen::TextPosition::Point(
                        ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                        ERROR_BOX_MARGIN
                            + ERROR_BOX_PADDING
                            + (line * pros_devices::screen::SCREEN_LINE_HEIGHT),
                    ),
                    pros_devices::screen::TextFormat::Small,
                ),
                pros_devices::color::Rgb::WHITE,
            )?;

            line += 1;
            buffer.clear();
        }
    }

    screen.fill(
        &pros_devices::screen::Text::new(
            buffer.as_str(),
            pros_devices::screen::TextPosition::Point(
                ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                ERROR_BOX_MARGIN
                    + ERROR_BOX_PADDING
                    + (line * pros_devices::screen::SCREEN_LINE_HEIGHT),
            ),
            pros_devices::screen::TextFormat::Small,
        ),
        pros_devices::color::Rgb::WHITE,
    )?;

    Ok(())
}

#[panic_handler]
/// The panic handler for pros-rs.
pub fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let current_task = pros_core::task::current();

    let task_name = current_task.name().unwrap_or_else(|_| "<unknown>".into());

    // task 'User Initialization (PROS)' panicked at src/lib.rs:22:1:
    // panic message here
    let msg = format!("task '{task_name}' {info}");

    eprintln!("{msg}");

    unsafe {
        #[cfg(feature = "display_panics")]
        draw_error(&mut Screen::new(), &msg).unwrap_or_else(|err| {
            eprintln!("Failed to draw error message to screen: {err}");
        });

        #[cfg(target_arch = "wasm32")]
        sim_log_backtrace();

        pros_sys::exit(1);
    }
}
