#![no_main]
#![no_std]

use vexide::{prelude::*, startup::banner_themes::THEME_SYNTHWAVE};

#[vexide::main(banner(theme = THEME_SYNTHWAVE))]
async fn main(_peripherals: Peripherals) {
    println!("This program has a synthwave banner!");
}
