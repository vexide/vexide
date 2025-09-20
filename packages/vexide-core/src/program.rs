//! User program state.

use core::ptr::NonNull;

use vex_sdk::vexSystemLinkAddrGet;
use bitflags::bitflags;

/// Identifies the type of binary to VEXos.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum ProgramType {
    /// User program binary.
    User = 0,
}

/// The owner (originator) of the user program
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    /// Program Startup Options
    ///
    /// These bitflags are part of the [`CodeSignature`] and determine some small
    /// aspects of program behavior when running under VEXos. This struct contains
    /// the flags with publicly documented behavior.
    #[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
    pub struct ProgramOptions: u32 {
        /// Inverts the background color to pure white.
        const INVERT_DEFAULT_GRAPHICS = 1 << 0;

        /// VEXos scheduler simple tasks will be killed when the program requests exit.
        const KILL_TASKS_ON_EXIT = 1 << 1;

        /// If VEXos is using the Light theme, inverts the background color to pure white.
        const THEMED_DEFAULT_GRAPHICS = 1 << 2;
    }
}

/// Program Code Signature
///
/// The first 16 bytes of a VEX user program contains a code signature header,
/// which has some basic metadata and startup flags for the program. This
/// signature must be at the start of the binary for VExos to recognize our
/// binary as a program.
///
/// A static instance of this type can be passed to the `code_sig` argument of the
/// `#[vexide::main]` macro to override the default code signature, or may be placed
/// into the `.code_signature` linker section if not using the macro.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct CodeSignature(vex_sdk::vcodesig, [u32; 4]);

impl CodeSignature {
    /// Creates a new signature given a program type, owner, and flags.
    #[must_use]
    pub const fn new(
        program_type: ProgramType,
        owner: ProgramOwner,
        options: ProgramOptions,
    ) -> Self {
        Self(
            vex_sdk::vcodesig {
                magic: vex_sdk::V5_SIG_MAGIC,
                r#type: program_type as _,
                owner: owner as _,
                options: options.bits(),
            },
            [0; 4],
        )
    }

    pub const fn owner(&self) -> ProgramOwner {
        match self.0.owner {
            0 => ProgramOwner::System,
            1 => ProgramOwner::Vex,
            2 => ProgramOwner::Partner,
            _ => unreachable!(),
        }
    }

    pub const fn program_type(&self) -> ProgramType {
        match self.0.r#type {
            0 => ProgramType::User,
            _ => unreachable!(),
        }
    }

    pub const fn options(&self) -> ProgramOptions {
        ProgramOptions::from_bits_retain(self.0.options)
    }
}
