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
        panic!("AAAAAAHHHHHHH!");
        Ok(())
    }
}
async_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
