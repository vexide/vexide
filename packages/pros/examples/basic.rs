#![no_std]
#![no_main]

use pros::prelude::*;
use core::fmt::Write;

pub struct Robot {
    screen: Screen,
}

impl Robot {
    pub fn new(peripherals: Peripherals) -> Self {
        Self {
            screen: peripherals.screen,
        }
    }
}

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        let mut i = 1;

        loop {
            self.screen.write_fmt(format_args!("{i}\n"))?;
            i += 1;
            pros::task::delay(core::time::Duration::from_millis(100));
        }

        // self.screen.write_str("Dolor sit")?;

        Ok(())
    }
}
async_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
