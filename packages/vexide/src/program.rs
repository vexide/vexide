//! Functions for modifying the state of the current
//! user program.

use core::{convert::Infallible, fmt::Debug, time::Duration};

use vex_sdk::{vexSerialWriteFree, vexSystemExitRequest, vexTasksRun, vcodesig};
use bitflags::bitflags;

use no_std_io::io::Write;

use crate::{io::{self, stdout}, time::Instant};

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
                write!(stdout(), "Error: {e:?}").unwrap();
                exit();
            }
        }
    }
}

/// Program Code Signature
///
/// The first 16 bytes of a VEX user code binary contain a user code signature,
/// containing some basic metadata and startup flags about the program. This
/// signature must be at the start of the binary for booting to occur.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct CodeSignature {
    /// Magic bytes. Should be "VXV5" little-endian.
    pub magic: u32,

    /// Program type
    pub program_type: ProgramType,

    /// Program originator
    pub owner: ProgramOwner,

    /// Program flags
    pub flags: ProgramFlags,
}

impl CodeSignature {
    /// Creates a new signature given a program type, owner, and flags.
    pub const fn new(program_type: ProgramType, owner: ProgramOwner, flags: ProgramFlags) -> Self {
        Self {
            magic: vex_sdk::V5_SIG_MAGIC,
            program_type,
            owner,
            flags,
        }
    }
}

// TODO: This impl should probably be TryFrom and not have unreachables
impl From<vcodesig> for CodeSignature {
    fn from(vcodesig: vcodesig) -> Self {
        Self {
            magic: vcodesig.magic,
            program_type: match vcodesig.r#type {
                0 => ProgramType::User,
                _ => unreachable!("Encountered unknown program type in code signature!"),
            },
            owner: match vcodesig.owner {
                0 => ProgramOwner::System,
                1 => ProgramOwner::Vex,
                2 => ProgramOwner::Partner,
                _ => unreachable!("Encountered unknown program owner in code signature!"),
            },
            flags: ProgramFlags::from_bits_retain(vcodesig.options),
        }
    }
}

/// Identifies the type of binary to VEXos.
#[repr(u32)]
#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub enum ProgramType {
    /// User program binary.
    User = 0,
}

/// The owner (originator) of the user program
#[repr(u32)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
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
    #[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
    pub struct ProgramFlags: u32 {
        /// Default graphics colors will be inverted.
        const INVERT_DEFAULT_GRAPHICS = 1 << 0;

        /// VEXos scheduler simple tasks will be killed when the program requests exit.
        const KILL_TASKS_ON_EXIT = 1 << 1;

        /// Default graphics colors will invert based on the selected system theme.
        const THEMED_DEFAULT_GRAPHICS = 1 << 2;
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
            if vexSerialWriteFree(io::STDIO_CHANNEL) == (crate::io::Stdout::INTERNAL_BUFFER_SIZE as i32) {
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
