#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

#[derive(Default)]
pub struct Robot;

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> pros::Result {
        let peripherals = Peripherals::take().unwrap();
        let mut imu = InertialSensor::new(peripherals.smart_1);

        imu.calibrate().await?;

        loop {
            let euler = imu.euler()?;

            println!(
                "Pitch: {} Roll: {} Yaw: {}",
                euler.pitch, euler.roll, euler.yaw
            );

            pros::task::delay(Duration::from_secs(1));
        }
    }
}

async_robot!(Robot);
