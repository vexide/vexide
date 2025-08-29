//! Minimal example of setting up program booting without the `#[vexide::main]` attribute macro.

use std::time::Duration;

use vexide_devices::{
    peripherals::Peripherals,
    smart::{
        motor::{Direction, Gearset},
        Motor,
    },
};
use vexide_startup::{CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

fn main() {
    // Setup the heap, zero bss, apply patches, etc...
    // SAFETY: Called once at program startup.
    unsafe {
        vexide_startup::startup();
    }

    println!("Hey");

    // Spin a motor
    let peripherals = Peripherals::take().unwrap();
    let mut m = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    m.set_voltage(12.0).unwrap();

    std::thread::sleep(Duration::from_secs(5));
}

// SAFETY: The code signature needs to be in this section so it may be found by VEXos.
#[unsafe(link_section = ".code_signature")]
#[used] // This is needed to prevent the linker from removing this object in release builds
static CODE_SIG: CodeSignature = CodeSignature::new(
    ProgramType::User,
    ProgramOwner::Partner,
    ProgramFlags::empty(),
);
