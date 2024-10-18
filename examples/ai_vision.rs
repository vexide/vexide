#![no_main]
#![no_std]

use core::time::Duration;

use vexide::prelude::*;
use vexide_devices::smart::ai_vision::{AiVisionColor, AiVisionSensor};

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let mut ai_vision = AiVisionSensor::new(peripherals.port_1, 1.0, 1.0);
    ai_vision.set_color(1, AiVisionColor { red: 55, green: 125, blue: 70, hue: 10.0, saturation: 0.2 }).unwrap();
    loop {
        println!("Vision Sensor: {:?}", ai_vision.num_objects().unwrap());

        sleep(Duration::from_millis(10)).await;
    }
}
