#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

#[derive(Default)]
pub struct Robot;

impl SyncRobot for Robot {
    fn opcontrol(&mut self) -> pros::Result {
        let peripherals = Peripherals::take().unwrap();
        let sensor = OpticalSensor::new(peripherals.smart_1, true)?;

        loop {
            println!(
				"-----\nHue: {}\nSaturation: {}\nBrightess: {}\nLast Gesture Direction: {:?}\n-----\n",
				sensor.hue()?,
				sensor.saturation()?,
				sensor.brightness()?,
				sensor.last_gesture_direction()?
			);

            pros::task::delay(Duration::from_millis(10));
        }
    }
}

sync_robot!(Robot);
