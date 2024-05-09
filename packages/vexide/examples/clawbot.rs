//! Clawbot Control Example
//!
//! Demonstrates a program written for the V5 clawbot kit robot. This example is
//! partially based on jpearman's [`v5-drivecode`] repository.
//!
//! [`v5-drivecode`]: https://github.com/jpearman/v5-drivecode

#![no_main]
#![no_std]
#![feature(error_in_core)]

extern crate alloc;

use alloc::boxed::Box;
use core::{error::Error, time::Duration};

use vexide::prelude::*;

struct ClawBot {
    left_motor: Motor,
    right_motor: Motor,
    claw: Motor,
    arm: Motor,
    left_bumper: AdiDigitalIn,
    right_bumper: AdiDigitalIn,
    arm_limit_switch: AdiDigitalIn,
    controller: Controller,
}

impl CompetitionRobot for ClawBot {
    type Error = Box<dyn Error>;

    async fn autonomous(&mut self) -> Result<(), Self::Error> {
        self.left_motor
            .set_target(MotorControl::Position(Position::Rotations(10.0), 100))?;
        self.right_motor
            .set_target(MotorControl::Position(Position::Rotations(10.0), 100))?;

        loop {
            sleep(Duration::from_millis(10)).await;
        }
    }

    async fn driver(&mut self) -> Result<(), Self::Error> {
        loop {
            // Simple arcade drive
            let forward = self.controller.left_stick.y()? as f64;
            let turn = self.controller.right_stick.x()? as f64;
            let mut left = forward + turn;
            let mut right = forward - turn;

            // If we are pressing the bumpers, don't allow the motors to go in reverse
            if self.left_bumper.is_high()? || self.right_bumper.is_high()? {
                left = left.max(0.0);
                right = right.max(0.0);
            }

            self.left_motor.set_voltage(left * 12.0)?;
            self.right_motor.set_voltage(right * 12.0)?;

            if self.controller.right_trigger_1.is_pressed()? {
                self.arm.set_voltage(12.0)?;
            } else if self.controller.right_trigger_2.is_pressed()? {
                self.arm.set_voltage(-12.0)?;
            } else {
                self.arm.set_voltage(0.0)?;
            }

            if self.controller.left_trigger_1.is_pressed()? {
                self.claw.set_voltage(12.0)?;
            } else if self.controller.left_trigger_2.is_pressed()?
                && !self.arm_limit_switch.is_high()?
            {
                self.claw.set_voltage(-12.0)?;
            } else {
                self.claw.set_voltage(0.0)?;
            }

            sleep(Controller::UPDATE_RATE).await;
        }
    }
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
    ClawBot {
        left_motor: Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward),
        right_motor: Motor::new(peripherals.port_10, Gearset::Green, Direction::Reverse),
        claw: Motor::new(peripherals.port_3, Gearset::Green, Direction::Forward),
        arm: Motor::new(peripherals.port_8, Gearset::Green, Direction::Forward),
        left_bumper: AdiDigitalIn::new(peripherals.adi_a),
        right_bumper: AdiDigitalIn::new(peripherals.adi_b),
        arm_limit_switch: AdiDigitalIn::new(peripherals.adi_h),
        controller: peripherals.primary_controller,
    }
    .compete()
    .await
    .unwrap();
}
