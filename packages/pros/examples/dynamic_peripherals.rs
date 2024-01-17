#![no_std]
#![no_main]

use pros::{
    peripherals::{DynamicPeripherals, Peripherals},
    prelude::*,
};

pub struct Robot {
    peripherals: DynamicPeripherals,
}
impl Robot {
    fn new() -> Self {
        Self {
            peripherals: Peripherals::take().unwrap().into(),
        }
    }
}
impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        let motor = Motor::new(
            self.peripherals.take_smart_port(10).unwrap(),
            BrakeMode::Brake,
        )?;
        motor.wait_until_stopped().await?;
        Ok(())
    }
}
async_robot!(Robot, Robot::new());
