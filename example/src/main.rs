#![no_std]
#![no_main]

use core::time::Duration;
use pros::prelude::*;

#[derive(Debug, Default)]
struct ExampleRobot;

#[robot]
impl Robot for ExampleRobot {
    fn opcontrol(&mut self) -> pros::Result {
        // Create a new motor plugged into port 2. The motor will brake when not moving.
        let motor = Motor::new(2, BrakeMode::Brake)?;
        // Create a controller, specifically controller 1.
        let controller = Controller::new(ControllerId::Master);

        let mut vision = VisionSensor::new(9, VisionZeroPoint::Center)?;
        vision.set_led(LedMode::On(Rgb::new(0, 0, 255)));

        pros::lcd::buttons::register(left_button_callback, Button::Left);

        // Spawn a new task that will print whether or not the motor is stopped constantly.
        spawn(move || loop {
            println!(
                "Motor stopped? {}",
                motor.get_state().unwrap_or_default().stopped
            );

            // Sleep the task as to not steal processing time from the OS.
            // This should always be done in any loop, including loops in the main task.
            sleep(Duration::from_millis(20));
        });

        loop {
            // Set the motors output with how far up or down the right joystick is pushed.
            // Set output takes a float from -1 to 1 that is scaled to -12 to 12 volts.
            motor.set_output(controller.state().joysticks.right.y)?;

            // println!("pid out {}", pid.update(10.0, motor.position().into_degrees() as f32));
            println!("Vision objs {}", vision.nth_largest_object(0)?.middle_x);

            // Once again, sleep.
            sleep(Duration::from_millis(20));
        }
    }
}

fn left_button_callback() {
    println!("Left button pressed!");
}
