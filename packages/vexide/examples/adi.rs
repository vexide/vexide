use vexide::prelude::*;

#[vexide::main]
async fn main(peripherals: Peripherals) {
    // Create a potentiometer on triport A. We'll assume the potentiometer is the newer V2 potentiometer
    // rather than the legacy cortex-era one.
    let potentiometer = AdiPotentiometer::new(peripherals.adi_a, PotentiometerType::V2);

    // Create a line tracker on triport B.
    let line_tracker = AdiLineTracker::new(peripherals.adi_b);

    // Create an ultrasonic range finder on triport C.
    let range_finder = AdiRangeFinder::new((peripherals.adi_c, peripherals.adi_d)).unwrap();

    loop {
        // Print out the sensor values to stdout every 10ms (the update rate of ADI devices).
        println!(
            "Potentiometer Angle: {}\nLine Tracker Reflectivity: {}%\nUltrasonic Distance: {}\n",
            potentiometer.angle().unwrap(),
            line_tracker.reflectivity().unwrap() * 100.0,
            range_finder.distance().unwrap()
        );

        // All ADI devices only update at 10ms, so we'll yield back to the async executor to
        // not hog all the CPU while looping.
        sleep(vexide::devices::adi::ADI_UPDATE_INTERVAL).await;
    }
}
