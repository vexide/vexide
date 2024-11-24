#![no_main]
#![no_std]

use core::{fmt::Write, time::Duration};

use vexide::{
    devices::display::{Font, FontFamily, FontSize, Rect, Text},
    prelude::*,
};

#[vexide::main]
async fn main(peripherals: Peripherals) {
    // We can get the screen directly from peripherals because it is always connected to the Brain.
    let mut display = peripherals.display;

    // Print a message to the screen. This will be displayed in the top left corner.
    write!(display, "Hello, world!").unwrap();

    let rect = Rect::new([20, 20], [120, 120]);

    // Fill in the entire rectangle with white.
    display.fill(&rect, Rgb::new(255, 255, 255));
    // Draw a thin magenta border of the same dimensions.
    // This will appear on top of the white fill because it is called later.
    display.stroke(&rect, Rgb::new(255, 0, 255));

    let text = Text::new("Nice to see you!", Font::default(), [80, 40]);

    // Draw the text on the display in cyan.
    display.fill(&text, Rgb::new(0, 255, 255));

    // You can use varying text sizes and fonts.
    let text = Text::new(
        "This is vexide.",
        Font::new(FontSize::from_fraction(2, 3), FontFamily::Proportional),
        [21, 84],
    );
    display.fill(&text, Rgb::new(255, 255, 255));

    // Font sizes can be created with a fraction or a float
    let size = FontSize::from_float(0.333).unwrap();
    println!("Font Size: {:?}", size);
    let size = FontSize::try_from(1.4).unwrap();
    println!("Font Size: {:?}", size);

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
