#![no_main]
#![no_std]
#![feature(never_type)]

extern crate alloc;

use alloc::boxed::Box;
use core::time::Duration;

use vexide::prelude::*;
use vexide_devices::screen::Rect;

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let mut screen = peripherals.screen;

    println!("Hi");
    screen.stroke(&Rect::new((0, 0), (30, 30)), Rgb::RED);

    screen.fill(&(30, 30), Rgb::BLUE);

    loop {
        sleep(Duration::from_millis(10)).await;
    }
}
