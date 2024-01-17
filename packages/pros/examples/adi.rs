#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

#[derive(Debug, Default)]
struct ExampleRobot;

impl AsyncRobot for ExampleRobot {
    async fn opcontrol(&mut self) -> pros::Result {
        let encoder_top = unsafe { AdiPort::new(1, None) };
        let encoder_bottom = unsafe { AdiPort::new(2, None) };

        let ultrasonic_ping = unsafe { AdiPort::new(3, None) };
        let ultrasonic_echo = unsafe { AdiPort::new(4, None) };

        let encoder = AdiEncoder::new((encoder_top, encoder_bottom), false).unwrap();
        let ultrasonic = AdiUltrasonic::new((ultrasonic_ping, ultrasonic_echo)).unwrap();

        encoder.zero().unwrap();
        ultrasonic.zero().unwrap();

        loop {
            println!("Encoder value: {}", encoder.value());
            println!("Ultrasonic value: {}", ultrasonic.value());

            pros::task::delay(Duration::from_millis(10));
        }
    }
}

async_robot!(ExampleRobot);
