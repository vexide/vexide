#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use core::time::Duration;

use pros::{
    core::sync::Mutex,
    devices::{
        smart::vision::{LedMode, VisionZeroPoint},
        Controller,
    },
    prelude::*,
};

struct ExampleRobot {
    motor: Arc<Mutex<Motor>>,
    vision: VisionSensor,
}
impl ExampleRobot {
    pub fn new(peripherals: Peripherals) -> Self {
        Self {
            motor: Arc::new(Mutex::new(
                Motor::new(peripherals.port_2, Gearset::Green, Direction::Forward).unwrap(),
            )),
            vision: VisionSensor::new(peripherals.port_9, VisionZeroPoint::Center).unwrap(),
        }
    }
}

impl AsyncRobot for ExampleRobot {
    async fn opcontrol(&mut self) -> Result {
        let handle = pros::async_runtime::spawn(async {
            for _ in 0..5 {
                println!("Hello from async!");
                sleep(Duration::from_millis(1000)).await;
            }
        });

        handle.await;

        // Create a controller, specifically controller 1.
        let controller = Controller::Master;

        self.vision.set_led(LedMode::On(Rgb::new(0, 0, 255)));

        // Spawn a new task that will print whether or not the motor is stopped constantly.
        pros_core::task::spawn({
            let motor = Arc::clone(&self.motor); // Obtain a shared reference to our motor to safely share between tasks.

            move || loop {
                println!("Motor stopped? {}", motor.lock().velocity() < 2);

                // Sleep the task as to not steal processing time from the OS.
                // This should always be done in any loop, including loops in the main task.
                // Because this is a real FreeRTOS task this is not the sleep function used elsewhere in this example.
                // This sleep function will block the entire task, including the async executor! (There isn't one running here, but there is in the main task.)
                delay(Duration::from_millis(Motor::DATA_READ_RATE));
            }
        });

        loop {
            // Set the motors output with how far up or down the right joystick is pushed.
            // Set output takes a float from -1 to 1 that is scaled to -12 to 12 volts.
            self.motor
                .lock()
                .set_voltage(Motor::MAX_VOLTAGE * controller.state()?.joysticks.right.y)?;

            // println!("pid out {}", pid.update(10.0, motor.position().into_degrees() as f32));
            println!(
                "Vision objs {}",
                self.vision.nth_largest_object(0)?.middle_x
            );

            // Once again, sleep.
            sleep(Duration::from_millis(20)).await;
        }
    }
}
async_robot!(
    ExampleRobot,
    ExampleRobot::new(Peripherals::take().unwrap())
);
