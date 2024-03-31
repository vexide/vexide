#![no_std]
#![no_main]

use pros::{devices::battery, prelude::*};

#[derive(Default)]
pub struct Robot;

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> Result {
        if battery::capacity()? < 20.0 {
            println!("Battery is low!");
        } else if battery::temperature()? > 999.0 {
            println!("Battery has exploded!");
        }

        Ok(())
    }
}
async_robot!(Robot);
