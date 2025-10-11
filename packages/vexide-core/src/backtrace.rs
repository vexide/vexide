//! Support for capturing stack backtraces.
//!
//! This module contains the support for capturing a stack backtrace through
//! the [`Backtrace`] type. Backtraces are helpful to attach to errors,
//! containing information that can be used to get a chain of where an error
//! was created.
//!
//! # Platform Support
//!
//! The [`Backtrace`] API is only functional on the `armv7a-vex-v5` platform
//! target. At the moment, this target only platform that vexide supports,
//! however this may change in the future.
//!
//! Additionally, backtraces will be unsupported if vexide is compiled without
//! the `backtrace` feature.

use alloc::vec::Vec;
use core::fmt::Display;

#[cfg(all(target_os = "vexos", feature = "backtrace"))]
use vex_libunwind::{registers, UnwindContext, UnwindCursor, UnwindError};

/// A captured stack backtrace.
///
/// This type stores the backtrace of a captured stack at a certain point in
/// time. The backtrace is represented as a list of instruction pointers.
///
/// # Platform Support
///
/// The [`Backtrace`] API is only functional on the `armv7a-vex-v5` platform
/// target. At the moment, this target only platform that vexide supports,
/// however this may change in the future.
///
/// Additionally, backtraces will be unsupported if vexide is compiled without
/// the `backtrace` feature.
///
/// # Example
///
/// ```
/// let backtrace = Backtrace::capture();
/// println!("{backtrace}");
/// ```
///
/// # Symbolication
///
/// The number stored in each frame is not particularly meaningful to humans on its own.
/// Using a tool such as `llvm-symbolizer` or `addr2line`, it can be turned into
/// a function name and line number to show what functions were being run at
/// the time of the backtrace's capture.
///
/// ```terminal
/// $ llvm-symbolizer -p -e ./target/armv7a-vex-v5/debug/program_name 0x380217b 0x380209b
/// my_function at /path/to/project/src/main.rs:30:14
///
/// main at /path/to/project/src/main.rs:21:9
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backtrace {
    /// The instruction pointers of each frame in the backtrace.
    frames: Vec<*const ()>,
}

impl Backtrace {
    /// Captures a backtrace at the current point of execution.
    ///
    /// If a backtrace could not be captured, an empty backtrace is returned.
    ///
    /// # Platform Support
    ///
    /// Backtraces will be empty on non-vex targets (e.g. WebAssembly) or when
    /// the `backtrace` feature is disabled.
    #[allow(clippy::inline_always)]
    #[inline(always)] // Inlining keeps this function from appearing in backtraces
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn capture() -> Self {
        #[cfg(all(target_os = "vexos", feature = "backtrace"))]
        return Self::try_capture().unwrap_or(Self { frames: Vec::new() });

        #[cfg(not(all(target_os = "vexos", feature = "backtrace")))]
        return Self { frames: Vec::new() };
    }

    /// Returns a slice of instruction pointers at every captured frame of the backtrace.
    #[must_use]
    pub const fn frames(&self) -> &[*const ()] {
        self.frames.as_slice()
    }

    /// Captures a backtrace at the current point of execution,
    /// returning an error if the backtrace fails to capture.
    ///
    /// # Platform Support
    ///
    /// See [`Backtrace::capture`].
    ///
    /// # Errors
    ///
    /// This function errors when the program's unwind info is corrupted.
    #[inline(never)] // Make sure there's always a frame to remove
    #[cfg(all(target_os = "vexos", feature = "backtrace"))]
    fn try_capture() -> Result<Self, UnwindError> {
        UnwindContext::capture(|context| {
            let mut cursor = UnwindCursor::new(&context)?;
            let mut frames = Vec::new();

            // Procedure based on mini_backtrace crate.
            loop {
                let mut instruction_pointer = cursor.register(registers::UNW_REG_IP)?;

                // Adjust IP to point inside the function — this improves symbolization quality.
                if !cursor.is_signal_frame()? {
                    instruction_pointer -= 1;
                }

                frames.push(instruction_pointer as *const ());

                // Step to the next frame, break if there is none.
                if !cursor.step()? {
                    break;
                }
            }

            Ok(Self { frames })
        })
    }
}

impl Display for Backtrace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "stack backtrace:")?;
        for (i, frame) in self.frames.iter().enumerate() {
            writeln!(f, "{i:>3}: 0x{:x}", *frame as usize)?;
        }
        write!(
            f,
            "note: Use a symbolizer to convert stack frames to human-readable function names."
        )?;
        Ok(())
    }
}
