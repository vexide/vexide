use vexide::{
    devices::screen::{Rect, Text, TextSize},
    prelude::*,
};

use std::{fmt::Write, time::Duration};

#[vexide::main]
async fn main(peripherals: Peripherals) {
    // We can get the screen directly from peripherals because it is always connected to the Brain.
    let mut screen = peripherals.screen;

    // Print a message to the screen
    write!(screen, "Hello, world!").unwrap();

    // Create a rectangle to be drawn on the screen.
    let rect = Rect::new((20, 20), (100, 100));

    // Fill in the entire rectangle with white.
    screen.fill(&rect, Rgb::new(255, 255, 255));
    // Draw a thin magenta border of the same dimensions.
    // This will appear on top of the white fill because it is called later.
    screen.stroke(&rect, Rgb::new(255, 0, 255));

    // Create a piece of text to draw on the screen at a specific position.
    let text = Text::new("Nice to see you!", TextSize::Medium, (80, 80));
    // Fill in the text with cyan.
    screen.fill(&text, Rgb::new(0, 255, 255));

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
