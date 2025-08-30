//! Minimal example  of setting up a vexide program without
//! the `#[vexide::main]` attribute macro or async runtime.
//!
//! There are a few reasons why you might want this over using
//! a normal Rust binary - notably, going through `vexide_startup`
//! provides support for differential uploading, implementations
//! for `vex_sdk`, graphical panics with backtraces, and a more
//! efficient/smaller allocator.

use std::time::Duration;

use vexide_devices::{
    peripherals::Peripherals,
    smart::motor::{Direction, Gearset, Motor},
};
use vexide_startup::{CodeSignature, ProgramFlags, ProgramOwner, ProgramType};

/// The code signature encodes some metadata about this program
/// at startup to VEXos.
///
/// `vexide-startup` allows you to customize your program's code
/// signature by placing data into the `.code_signature` section of
/// your binary.
// SAFETY: The code signature needs to be in this section so it may be found by VEXos.
#[unsafe(link_section = ".code_signature")]
#[used] // This is needed to prevent the linker from removing this object in release builds
static CODE_SIG: CodeSignature = CodeSignature::new(
    ProgramType::User,
    ProgramOwner::Partner,
    ProgramFlags::empty(),
);

fn main() {
    // Setup the heap, zero bss, apply patches, etc...
    // SAFETY: Called once at program startup.
    unsafe {
        vexide_startup::startup();
    }

    println!("Hey");

    // Try to spin a motor.
    let peripherals = Peripherals::take().unwrap();
    let mut m = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    _ = m.set_voltage(12.0);

    // Wait 5 seconds then exit.
    std::thread::sleep(Duration::from_secs(5));
}
