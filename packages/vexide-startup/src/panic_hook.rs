//! Custom panic hook for vexide programs.
//!
//! This extends the default `libstd` panic handler with support for capturing
//! backtrace data and drawing the panic message to the display screen.

use std::{fmt::Write, panic::PanicHookInfo};

use vexide_core::backtrace::Backtrace;
use vexide_devices::{
    display::{Display, Font, FontFamily, FontSize, Rect, Text},
    math::Point2,
};

use crate::colors::{RED, WHITE};

/// Draw an error box to the display.
///
/// This function is internally used by the vexide panic handler for displaying
/// panic messages graphically before exiting.
fn draw_error(display: &mut Display, msg: &str, backtrace: &Backtrace) {
    const ERROR_BOX_MARGIN: i16 = 16;
    const ERROR_BOX_PADDING: i16 = 16;
    const LINE_HEIGHT: i16 = 20;
    const LINE_MAX_WIDTH: usize = 52;

    fn draw_text(screen: &mut Display, buffer: &str, line: i16) {
        screen.draw_text(
            &Text::new(
                buffer,
                Font::new(FontSize::SMALL, FontFamily::Monospace),
                Point2 {
                    x: ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                    y: ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * LINE_HEIGHT),
                },
            ),
            (255, 255, 255),
            None,
        );
    }

    display.set_render_mode(vexide_devices::display::RenderMode::Immediate);

    let error_box_rect = Rect::new(
        Point2 {
            x: ERROR_BOX_MARGIN,
            y: ERROR_BOX_MARGIN,
        },
        Point2 {
            x: Display::HORIZONTAL_RESOLUTION - ERROR_BOX_MARGIN,
            y: Display::VERTICAL_RESOLUTION - ERROR_BOX_MARGIN,
        },
    );

    display.fill(&error_box_rect, RED);
    display.stroke(&error_box_rect, WHITE);

    let mut buffer = String::new();
    let mut line: i16 = 0;

    for (i, character) in msg.char_indices() {
        if !character.is_ascii_control() {
            buffer.push(character);
        }

        if character == '\n' || (buffer.len().is_multiple_of(LINE_MAX_WIDTH) && (i > 0)) {
            draw_text(display, &buffer, line);
            line += 1;
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        draw_text(display, &buffer, line);

        line += 1;
    }

    line += 1;
    draw_text(display, "stack backtrace:", line);
    line += 1;

    if !backtrace.frames.is_empty() {
        const ROW_LENGTH: usize = 3;
        for (col, frames) in backtrace.frames.chunks(ROW_LENGTH).enumerate() {
            let mut msg = String::new();
            for (row, frame) in frames.iter().enumerate() {
                write!(msg, "{:>3}: {:?}    ", col * ROW_LENGTH + row, frame).unwrap();
            }
            draw_text(display, msg.trim_end(), line);
            line += 1;
        }
    }
}

/// Panic hook for vexide programs.
///
/// This extends the default `libstd` panic handler with support for capturing
/// backtrace data and drawing the panic message to the display screen.
pub(crate) fn hook(info: &PanicHookInfo<'_>) {
    eprintln!("{info}");

    let backtrace = Backtrace::capture();

    draw_error(
        &mut unsafe { Display::new() },
        &info.to_string(),
        &backtrace,
    );

    if !backtrace.frames.is_empty() {
        eprintln!("{backtrace}");
    }

    // Don't exit the program, since we want to be able to see the panic message on the screen.
    loop {
        unsafe {
            vex_sdk::vexTasksRun();
        }
    }
}
