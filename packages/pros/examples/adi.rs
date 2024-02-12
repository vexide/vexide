#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

struct ExampleRobot {
    encoder: AdiEncoder,
    ultrasonic: AdiUltrasonic,
    gyro: AdiGyro,
}
impl ExampleRobot {
    pub fn new(peripherals: Peripherals) -> Self {
        Self {
            encoder: AdiEncoder::new((peripherals.adi_a, peripherals.adi_b), false).unwrap(),
            ultrasonic: AdiUltrasonic::new((peripherals.adi_c, peripherals.adi_d)).unwrap(),
            gyro: AdiGyro::new(peripherals.adi_e, 1.0).unwrap(),
        }
    }
}

impl AsyncRobot for ExampleRobot {
    async fn opcontrol(&mut self) -> Result {
        self.gyro.zero()?;
        self.encoder.zero()?;

        loop {
            println!("Encoder position: {:?}", self.encoder.position());
            println!("Ultrasonic distance: {:?}", self.ultrasonic.distance());

            delay(Duration::from_millis(10));
        }
    }
}

async_robot!(
    ExampleRobot,
    ExampleRobot::new(Peripherals::take().unwrap())
);
