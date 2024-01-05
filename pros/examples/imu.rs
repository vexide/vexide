#![no_std]
#![no_main]

use pros::prelude::*;
use core::time::Duration;

#[derive(Default)]
pub struct Robot;

impl SyncRobot for Robot {
    fn opcontrol(&mut self) -> pros::Result {
        let imu = InertialSensor::new(1)?;

        imu.calibrate()?;

        loop {
            // let euler = imu.euler()?;

            // println!("Pitch: {} Roll: {} Yaw: {}", euler.pitch, euler.roll, euler.yaw);

            let delay = Duration::from_secs(1);

            println!("{:?}", imu.is_calibrating());
            pros::task::delay(delay);
            println!("{:?}", imu.rotation());
            pros::task::delay(delay);
            println!("{:?}", imu.heading());
            pros::task::delay(delay);
            println!("{:?}", imu.pitch());
            pros::task::delay(delay);
            println!("{:?}", imu.roll());
            pros::task::delay(delay);
            println!("{:?}", imu.yaw());
            pros::task::delay(delay);
            println!("{:?}", imu.status());
            pros::task::delay(delay);
            println!("{:?}", imu.quaternion());
            pros::task::delay(delay);
            println!("{:?}", imu.euler());
            pros::task::delay(delay);
            println!("{:?}", imu.gyro_rate());
            pros::task::delay(delay);
            println!("{:?}", imu.accel());
            pros::task::delay(delay);

            pros::task::delay(Duration::from_secs(1));
        }

        Ok(())
    }
}

sync_robot!(Robot);
