#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

#[derive(Default)]
pub struct Robot;

impl SyncRobot for Robot {
    fn opcontrol(&mut self) -> pros::Result {
        let imu = InertialSensor::new(1)?;

        imu.calibrate()?;

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

sync_robot!(Robot);
