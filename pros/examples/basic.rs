#![no_std]
#![no_main]

use pros::prelude::*;

#[derive(Default)]
pub struct Robot;
#[async_trait]
impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        println!("basic example");

        Ok(())
    }
}
async_robot!(Robot);
