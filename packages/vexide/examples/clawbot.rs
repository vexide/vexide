//! Clawbot Control Example
//!
//! Demonstrates a program written for the V5 clawbot kit robot. This example is
//! partially based on jpearman's [`v5-drivecode`] repository.
//!
//! [`v5-drivecode`]: https://github.com/jpearman/v5-drivecode

#![no_main]
#![no_std]

extern crate alloc;

use core::time::Duration;

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

impl Compete for ClawBot {
    async fn autonomous(&mut self) {
        // Basic example autonomous that moves the drivetrain 10 revolutions forwards.
        self.left_motor
            .set_target(MotorControl::Position(
                Position::from_revolutions(10.0),
                100,
            ))
            .ok();
        self.right_motor
            .set_target(MotorControl::Position(
                Position::from_revolutions(10.0),
                100,
            ))
            .ok();

        loop {
            sleep(Duration::from_millis(10)).await;
        }
    }

    async fn driver(&mut self) {
        loop {
            // Bumper State
            let left_bumper_pressed = self.left_bumper.is_high().unwrap_or_default();
            let right_bumper_pressed = self.right_bumper.is_high().unwrap_or_default();

            // Limit Switch State
            let limit_switch_pressed = self.arm_limit_switch.is_high().unwrap_or_default();

            // Controller Buttons
            let r1_pressed = self
                .controller
                .right_trigger_1
                .is_pressed()
                .unwrap_or_default();
            let r2_pressed = self
                .controller
                .right_trigger_2
                .is_pressed()
                .unwrap_or_default();
            let l1_pressed = self
                .controller
                .left_trigger_1
                .is_pressed()
                .unwrap_or_default();
            let l2_pressed = self
                .controller
                .left_trigger_2
                .is_pressed()
                .unwrap_or_default();

            // Simple arcade drive
            let forward = self.controller.left_stick.y().unwrap_or_default() as f64;
            let turn = self.controller.right_stick.x().unwrap_or_default() as f64;
            let mut left_voltage = (forward + turn) * Motor::MAX_VOLTAGE;
            let mut right_voltage = (forward - turn) * Motor::MAX_VOLTAGE;

            // If we are pressing the bumpers, don't allow the motors to go in reverse
            if left_bumper_pressed || right_bumper_pressed {
                left_voltage = left_voltage.max(0.0);
                right_voltage = right_voltage.max(0.0);
            }

            // Set the drive motors to our arcade control values.
            self.left_motor.set_voltage(left_voltage).ok();
            self.right_motor.set_voltage(right_voltage).ok();

            // Arm control using the R1 and R2 buttons on the controller.
            if r1_pressed {
                self.arm.set_voltage(12.0).ok();
            } else if r2_pressed {
                self.arm.set_voltage(-12.0).ok();
            } else {
                self.arm.brake(BrakeMode::Hold).ok();
            }

            // Claw control using the L1 and L2 buttons on the controller.
            if l1_pressed {
                self.claw.set_voltage(12.0).ok();
            } else if l2_pressed && !limit_switch_pressed {
                self.claw.set_voltage(-12.0).ok();
            } else {
                self.arm.brake(BrakeMode::Hold).ok();
            }

            // Sleep some time, since we're limited by how fast the controller updates.
            //
            // Also need to give some CPU time for other tasks to run.
            sleep(Controller::UPDATE_INTERVAL).await;
        }
    }
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
    // Configuring devices and handing off control to the [`Competition`] API.
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
    .await;
}
