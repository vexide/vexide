#![no_std]
#![no_main]

use pros::prelude::*;

#[no_mangle]
pub extern "C" fn opcontrol() {
    // Create a new motor plugged into port zero. The motor will brake when not moving.
    let motor = pros::motor::Motor::new(0, pros::motor::BrakeMode::Brake).unwrap();
    // Create a controller, specifically controller 1.
    let controller = pros::controller::Controller::new(pros::controller::ControllerId::Master);

    // Create a copy of the motor to be moved into the print task.
    let motor_copy = motor.clone();

    // Spawn a new task that will print whether or not the motor is stopped constantly.
    // Priority changes how much time the operating system spends on the task, this can usually be set to default.
    // Stack depth is how large the stack that is given to the task is, again it can usually be left at default.
    // Name is an optional string usefull for debugging.
    pros::multitasking::Task::spawn(
        move || loop {
            println!("Motor stopped? {}", motor_copy.get_state().stopped);

            // Sleep the task as to not steal processing time from the OS.
            // This should always be done in any loop, including loops in the main task.
            pros::multitasking::sleep(core::time::Duration::from_millis(20));
        },
        pros::multitasking::TaskPriority::Default,
        pros::multitasking::TaskStackDepth::Default,
        Some("Print Task"),
    );

    loop {
        // Set the motors output with how far up or down the right joystick is pushed.
        // Set raw takes in an i8 which is scaled to -12 to 12 volts.
        motor.set_raw_output(controller.state().joysticks.right.y);

        // Once again, sleep.
        pros::multitasking::sleep(core::time::Duration::from_millis(20));
    }
}