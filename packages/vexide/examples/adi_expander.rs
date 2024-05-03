#![no_main]
#![no_std]

use vexide::prelude::*;

#[vexide::main]
async fn main(peripherals: Peripherals) {
    // Create an ADI expander on smart port 1.
    let expander = AdiExpander::new(peripherals.port_1);

    // Create a potentiometer on the expander.
    // The AdiExpander api is almost identical to that of Peripherals.
    // AdiPorts can be moved out of the struct to create ADI devices.
    let potentiometer = AdiPotentiometer::new(expander.adi_a, PotentiometerType::V2);

    loop {
        // Print out the sensor values to stdout every 10ms (the update rate of ADI devices).
        println!("Potentiometer Angle: {}", potentiometer.angle().unwrap(),);

        sleep(vexide::devices::adi::ADI_UPDATE_INTERVAL).await;
    }
}
