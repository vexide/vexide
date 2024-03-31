#![no_std]
#![no_main]

use core::time::Duration;

use pros::prelude::*;

pub struct Robot {
    encoder: AdiEncoder,
}
impl Robot {
    fn new(peripherals: Peripherals) -> Self {
        // Create an expander on smart port 1
        let expander = AdiExpander::new(peripherals.port_1);

        Self {
            // Create an encoder on the expander's A and B ports.
            encoder: AdiEncoder::new((expander.adi_a, expander.adi_b), false).unwrap(),
        }
    }
}

impl AsyncRobot for Robot {
    async fn opcontrol(&mut self) -> Result {
        // Read from the encoder every second.
        loop {
            println!("Encoder position: {}", self.encoder.position()?);

            delay(Duration::from_secs(1));
        }
    }
}
async_robot!(Robot, Robot::new(Peripherals::take().unwrap()));
