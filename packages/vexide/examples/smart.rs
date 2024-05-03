#![no_std]
#![no_main]

use core::time::Duration;

use vexide::prelude::*;

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let controller = peripherals.primary_controller;
    // Create two new motors on smart ports 1 and 10.
    let mut left_motor =
        Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    let mut right_motor =
        Motor::new(peripherals.port_10, Gearset::Green, Direction::Forward);

    // Create a new inertial sensor (IMU) on smart port 6.
    // We don't have to handle a result because this constructor is infallible.
    let mut imu = InertialSensor::new(peripherals.port_6);
    // Calibrate the IMU.
    imu.calibrate().await.unwrap();

    // Create a new radio link on smart port 15 with the id "example".
    let mut link = RadioLink::open(
        peripherals.port_15,
        "example",
        vexide_devices::smart::link::LinkType::Manager,
    )
    .unwrap();
    // Send a message over vexlink.
    // We dont have to flush because VEXOs does that immediately.
    link.write(b"Hello, world!").unwrap();

    // Create a new distance sensor on smart port 16.
    // This constructor is infallible.
    let distance = DistanceSensor::new(peripherals.port_16);

    loop {
        // Simple tank drive
        let left = controller.left_stick.y().unwrap() as f64;
        let right = controller.right_stick.y().unwrap() as f64;
        left_motor.set_voltage(12.0 * left).unwrap();
        right_motor.set_voltage(12.0 * right).unwrap();

        println!("IMU Euler angles: {:?}", imu.euler().unwrap());
        println!(
            "Distance: {}. Confidence: {}",
            distance.distance().unwrap(),
            distance.distance_confidence().unwrap()
        );

        // Don't hog the CPU
        sleep(Duration::from_millis(5)).await;
    }
}
