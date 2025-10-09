use vexide::prelude::*;

#[vexide::main]
async fn main(peripherals: Peripherals) {
    let sensor = DistanceSensor::new(peripherals.port_1);

    match sensor.object() {
        Ok(object) => match object {
            Some(data) => println!("Found an object! {data:?}"),
            None => println!("No object found."),
        },
        Err(error) => println!("An error occurred. {error}"),
    }
}
