#![no_std]
#![no_main]

use core::time::Duration;

use pros::{prelude::*, task::delay};

#[derive(Debug, Default)]
struct ExampleRobot;

impl AsyncRobot for ExampleRobot {
    async fn opcontrol(&mut self) -> pros::Result {
        let handle = pros::async_runtime::spawn(async {
            for _ in 0..5 {
                println!("Hello from async!");
                sleep(Duration::from_millis(1000)).await;
            }
        });

        pros::async_runtime::block_on(handle);

        // Create a new motor plugged into port 2. The motor will brake when not moving.
        let motor = Motor::new(2, BrakeMode::Brake)?;
        motor.wait_until_stopped().await?;
        // Create a controller, specifically controller 1.
        let controller = Controller::Master;

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
            // Because this is a real FreeRTOS task this is not the sleep function used elsewhere in this example.
            // This sleep function will block the entire task, including the async executor! (There isn't one running here, but there is in the main task.)
            delay(Duration::from_millis(20));
        });

        loop {
            // Set the motors output with how far up or down the right joystick is pushed.
            // Set output takes a float from -1 to 1 that is scaled to -12 to 12 volts.
            motor.set_output(controller.state().joysticks.right.y)?;

            // println!("pid out {}", pid.update(10.0, motor.position().into_degrees() as f32));
            println!("Vision objs {}", vision.nth_largest_object(0)?.middle_x);

            // Once again, sleep.
            sleep(Duration::from_millis(20)).await;
        }
    }
}
async_robot!(ExampleRobot);

fn left_button_callback() {
    println!("Left button pressed!");
}
