#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

pub struct Robot {
    imu: InertialSensor,
}
impl Robot {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            imu: InertialSensor::new(peripherals.port_1),
        }
    }
}

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> Result {
        self.imu.calibrate().await?;

        loop {
            let euler = self.imu.euler()?;

            println!(
                "Pitch: {} Roll: {} Yaw: {}",
                euler.pitch, euler.roll, euler.yaw
            );

            delay(Duration::from_secs(1));
        }
    }
}

async_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
