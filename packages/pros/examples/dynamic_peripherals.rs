#![no_std]
#![no_main]

use pros::prelude::*;

pub struct Robot {
    peripherals: DynamicPeripherals,
}
impl Robot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            peripherals: DynamicPeripherals::new(peripherals),
        }
    }
}
impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> Result {
        let motor = Motor::new(
            self.peripherals.take_smart_port(10).unwrap(),
            Gearset::Green,
            false,
        )?;
        motor.wait_until_stopped().await?;
        Ok(())
    }
}
async_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
