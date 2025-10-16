use vexide::{
    devices::smart::ai_vision::{AiVisionColor, AiVisionSensor},
    prelude::*,
};

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ai_vision
        .set_color(
            1,
            AiVisionColor {
                rgb: Rgb::new(55, 125, 70),
                hue_range: 10.0,
                saturation_range: 0.2,
            },
        )
        .unwrap();

    loop {
        println!("Vision Sensor: {:?}", ai_vision.object_count().unwrap());

        sleep(AiVisionSensor::UPDATE_INTERVAL).await;
    }
}
