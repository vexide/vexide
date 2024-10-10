#![no_std]
#![no_main]

use vexide::prelude::*;

struct Robot {
    controller: Controller,
    solenoid: AdiSolenoid,
}

impl Compete for Robot {
    async fn driver(&mut self) {
        loop {
            let controller_state = self.controller.state().unwrap_or_default();

            // Toggle the solenoid if button A got pressed.
            if controller_state.button_a.is_now_pressed() {
                self.solenoid.toggle().ok();
            }

            sleep(Controller::UPDATE_INTERVAL).await;
        }
    }
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
    Robot {
        controller: peripherals.primary_controller,
        solenoid: AdiSolenoid::new(peripherals.adi_a),
    }.compete().await;
}
