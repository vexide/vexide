#![no_std]
#![no_main]

use vexide::prelude::*;

/// 6-motor drivetrain robot with tank controls.
struct Robot {
    controller: Controller,
    left_motors: [Motor; 3],
    right_motors: [Motor; 3],
}

impl Compete for Robot {
    async fn driver(&mut self) {
        loop {
            let controller_state = self.controller.state().unwrap_or_default();

            // Move the left motors using the left stick's y-axis
            for motor in self.left_motors.iter_mut() {
                motor
                    .set_voltage(controller_state.left_stick.y() * Motor::V5_MAX_VOLTAGE)
                    .ok();
            }

            // Move the right motors using the left stick's y-axis
            for motor in self.right_motors.iter_mut() {
                motor
                    .set_voltage(controller_state.right_stick.y() * Motor::V5_MAX_VOLTAGE)
                    .ok();
            }

            sleep(Controller::UPDATE_INTERVAL).await;
        }
    }
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
    Robot {
        controller: peripherals.primary_controller,
        left_motors: [
            Motor::new(peripherals.port_1, Gearset::Blue, Direction::Reverse),
            Motor::new(peripherals.port_2, Gearset::Blue, Direction::Reverse),
            Motor::new(peripherals.port_3, Gearset::Blue, Direction::Forward),
        ],
        right_motors: [
            Motor::new(peripherals.port_4, Gearset::Blue, Direction::Forward),
            Motor::new(peripherals.port_5, Gearset::Blue, Direction::Forward),
            Motor::new(peripherals.port_6, Gearset::Blue, Direction::Reverse),
        ],
    }
    .compete()
    .await;
}
