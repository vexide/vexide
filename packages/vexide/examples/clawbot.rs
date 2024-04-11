//! Clawbot Control Example
//!
//! Demonstrates a program written for the V5 clawbot kit robot. This example is
//! partially based on jpearman's [`v5-drivecode`] repository.
//!
//! [`v5-drivecode`]: https://github.com/jpearman/v5-drivecode

#![no_main]
#![no_std]

use vexide::prelude::*;
use vexide_devices::PortError;

struct ClawBot {
    left_motor: Motor,
    right_motor: Motor,
    claw: Motor,
    arm: Motor,
}

impl Competition for Clawbot {
    type Error = PortError;

    fn driver(&mut self) -> Result<(), PortError> {
        loop {


			sleep(Controller::UPDATE_RATE).await;
		}
    }
}

#[vexide_startup::main]
async fn main(peripherals: Peripherals) {
    ClawBot {
        left_motor: Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward).unwrap(),
        right_motor: Motor::new(peripherals.port_10, Gearset::Green, Direction::Reverse).unwrap(),
        claw: Motor::new(peripherals.port_3, Gearset::Green, Direction::Forward).unwrap(),
        arm: Motor::new(peripherals.port_8, Gearset::Green, Direction::Forward).unwrap(),
    }.compete().await.unwrap();
}
