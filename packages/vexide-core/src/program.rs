//! Functions for modifying the state of the current
//! user program.

use core::{convert::Infallible, fmt::Debug, time::Duration};

use bitflags::bitflags;
use vex_sdk::{vexSerialWriteFree, vexSystemExitRequest, vexTasksRun};

use crate::{io, time::Instant};

/// A that can be implemented for arbitrary return types in the main function.
pub trait Termination {
    /// Run specific termination logic.
    /// Unlike in the standard library, this function does not return a status code.
    fn report(self);
}
impl Termination for () {
    fn report(self) {}
}
impl Termination for ! {
    fn report(self) {}
}
impl Termination for Infallible {
    fn report(self) {}
}
impl<T: Termination, E: Debug> Termination for Result<T, E> {
    fn report(self) {
        match self {
            Ok(t) => t.report(),
            Err(e) => {
                io::println!("Error: {e:?}");
                exit();
            }
        }
    }
}

/// Identifies the type of binary to VEXos.
#[repr(u32)]
#[non_exhaustive]
pub enum ProgramType {
    /// User program binary.
    User = 0,
}

/// The owner (originator) of the user program
#[repr(u32)]
pub enum ProgramOwner {
    /// Program is a system binary.
    System = 0,

    /// Program originated from VEX.
    Vex = 1,

    /// Program originated from a partner developer.
    Partner = 2,
}

bitflags! {
    /// Program Flags
    ///
    /// These bitflags are part of the [`CodeSignature`] that determine some small
    /// aspects of program behavior when running under VEXos. This struct contains
    /// the flags with publicly documented behavior.
    #[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
    pub struct ProgramFlags: u32 {
        /// Default graphics colors (foreground/background) will be inverted.
        const INVERT_DEFAULT_GRAPHICS = 1 << 0;

        /// VEXos scheduler simple tasks will be killed when the program requests exit.
        const KILL_TASKS_ON_EXIT = 1 << 1;

        /// Default graphics colors (foreground/background) will invert based on
        /// the selected system theme.
        const THEMED_DEFAULT_GRAPHICS = 1 << 2;
    }
}

/// Program Code Signature
///
/// The first 16 bytes of a VEX user code binary contain a user code signature,
/// containing some basic metadata and startup flags about the program. This
/// signature must be at the start of the binary for booting to occur.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct CodeSignature(vex_sdk::vcodesig, [u32; 4]);

impl CodeSignature {
    /// Creates a new signature given a program type, owner, and flags.
    pub const fn new(
        program_type: ProgramType,
        owner: ProgramOwner,
        flags: ProgramFlags
    ) -> Self {

        Self(vex_sdk::vcodesig {
            magic: vex_sdk::V5_SIG_MAGIC,
            r#type: program_type as _,
            owner: owner as _,
            options: flags.bits(),
        }, [0; 4])
    }
}


/// Exits the program using vexSystemExitRequest.
/// This function will not instantly exit the program,
/// but will instead wait 3ms to force the serial buffer to flush.
pub fn exit() -> ! {
    let exit_time = Instant::now();
    const FLUSH_TIMEOUT: Duration = Duration::from_millis(15);
    unsafe {
        // Force the serial buffer to flush
        while exit_time.elapsed() < FLUSH_TIMEOUT {
            // If the buffer has been fully flushed, exit the loop
            if vexSerialWriteFree(io::STDIO_CHANNEL) == (io::Stdout::INTERNAL_BUFFER_SIZE as i32) {
                break;
            }
            vexTasksRun();
        }
        // Exit the program
        // Everything after this point is unreachable.
        vexSystemExitRequest();
    }

    // unreachable.
    loop {
        core::hint::spin_loop();
    }
}
