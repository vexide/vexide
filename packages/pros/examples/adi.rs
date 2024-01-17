#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

#[derive(Debug, Default)]
struct ExampleRobot;

impl AsyncRobot for ExampleRobot {
    async fn opcontrol(&mut self) -> pros::Result {
        let encoder_top_port = unsafe { AdiPort::new(1, None) };
        let encoder_bottom_port = unsafe { AdiPort::new(2, None) };

        let ultrasonic_ping_port = unsafe { AdiPort::new(3, None) };
        let ultrasonic_echo_port = unsafe { AdiPort::new(4, None) };

        let gyro_port = unsafe { AdiPOrt::new(5, None) };

        let encoder = AdiEncoder::new((encoder_top_port, encoder_bottom_port), false).unwrap();
        let ultrasonic = AdiUltrasonic::new((ultrasonic_ping_port, ultrasonic_echo_port)).unwrap();
        let gyro = AdiGyro::new(gyro_port, 1.0).unwrap();

        gyro.zero().unwrap();
        encoder.zero().unwrap();
        ultrasonic.zero().unwrap();

        loop {
            println!("Encoder value: {:?}", encoder.value());
            println!("Ultrasonic value: {:?}", ultrasonic.value());

            pros::task::delay(Duration::from_millis(10));
        }
    }
}

async_robot!(ExampleRobot);
