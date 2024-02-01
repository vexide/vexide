#![no_std]
#![no_main]

use core::fmt::Write;

use pros::prelude::*;

#[derive(Default)]
pub struct Robot;

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        println!("basic example");

        Ok(())
    }
}
async_robot!(Robot);
