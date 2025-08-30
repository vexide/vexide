//! Minimal example of setting up a program without the `#[vexide::main]` attribute macro.

use std::time::Duration;

use vexide_devices::{
    peripherals::Peripherals,
    smart::motor::{Direction, Gearset, Motor},
};
use vexide_startup::{CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

fn main() {
    // Setup the heap, zero bss, apply patches, etc...
    // SAFETY: Called once at program startup.
    unsafe {
        vexide_startup::startup();
    }

    println!("Hey");

    // Try to spin a motor
    let peripherals = Peripherals::take().unwrap();
    let mut m = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    _ = m.set_voltage(12.0);

    std::thread::sleep(Duration::from_secs(5));
}
