#![no_std]
#![no_main]

use pros::prelude::*;

#[derive(Default)]
pub struct Robot;

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        if pros::battery::capacity()? < 20.0 {
            println!("WARNING: Battery Low!");
        }

        if pros::battery::temperature()? > 999.0 {
            println!("WARNING: Battery is on fire!");
        }

        Ok(())
    }
}
async_robot!(Robot);
