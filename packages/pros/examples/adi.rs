#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

#[derive(Debug, Default)]
struct ExampleRobot;

impl AsyncRobot for ExampleRobot {
    async fn opcontrol(&mut self) -> pros::Result {
        let peripherals = Peripherals::take().unwrap();

        let encoder_top_port = peripherals.adi_a;
        let encoder_bottom_port = peripherals.adi_b;

        let ultrasonic_ping_port = peripherals.adi_c;
        let ultrasonic_echo_port = peripherals.adi_d;

        let gyro_port = peripherals.adi_e;

        let mut encoder = AdiEncoder::new((encoder_top_port, encoder_bottom_port), false).unwrap();
        let ultrasonic = AdiUltrasonic::new((ultrasonic_ping_port, ultrasonic_echo_port)).unwrap();
        let mut gyro = AdiGyro::new(gyro_port, 1.0).unwrap();

        gyro.zero().unwrap();
        encoder.zero().unwrap();

        loop {
            println!("Encoder value: {:?}", encoder.value());
            println!("Ultrasonic value: {:?}", ultrasonic.value());

            pros::task::delay(Duration::from_millis(10));
        }
    }
}

async_robot!(ExampleRobot);
