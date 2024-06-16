#![no_std]
#![no_main]

extern crate alloc;

use vexide::prelude::*;

struct Robot;

impl Competition for Robot {
    async fn driver(&mut self) {
        loop {
            match self.controller.button_a.event().unwrap_or_default() {
                ButtonEvent::Press | ButtonEvent::Release => self.solenoid.toggle().unwrap(),
                _ => ()
            }

            sleep(Controller::UPDATE_INTERVAL).await;
        }
    }
}

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    Robot.compete().await;
}
