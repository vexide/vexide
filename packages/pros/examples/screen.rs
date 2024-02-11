#![no_std]
#![no_main]

use core::fmt::Write;

use pros::prelude::*;

pub struct Robot {
    screen: Screen,
}

impl Robot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            screen: peripherals.screen,
        }
    }
}

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        self.screen.fill(&Rect::new(0, 0, 20, 20), Rgb::RED)?;
        self.screen.stroke(&Circle::new(25, 25, 20), Rgb::BLUE)?;

        writeln!(self.screen, "Hello, world.")?;

        Ok(())
    }
}
async_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
