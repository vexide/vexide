//! Support for capturing stack backtraces
//!
//! This module contains the support for capturing a stack backtrace through
//! the [`Backtrace`] type. Backtraces are helpful to attach to errors,
//! containing information that can be used to get a chain of where an error
//! was created.

use alloc::vec::Vec;
use core::{ffi::c_void, fmt::Display};

#[cfg(all(target_arch = "arm", feature = "backtraces"))]
use vex_libunwind::*;

/// A captured stack backtrace.
///
/// This type stores the backtrace of a captured stack at a certain point in
/// time. The backtrace is represented as a list of instruction pointers.
///
/// ```
/// let backtrace = Backtrace::capture();
/// println!("{backtrace}");
/// ```
///
/// ## Symbolication
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
    pub frames: Vec<*const c_void>,
}

impl Backtrace {
    /// Captures a backtrace at the current point of execution.
    ///
    /// If a backtrace could not be captured, an empty backtrace is returned.
    ///
    /// ## Platform Support
    ///
    /// Backtraces will be empty on non-armv7a targets (e.g. WebAssembly) or when
    /// the `unwind` feature is disabled.
    #[inline(always)] // Inlining keeps this function from appearing in backtraces
    #[allow(clippy::missing_const_for_fn)]
    pub fn capture() -> Self {
        #[cfg(all(target_arch = "arm", feature = "backtraces"))]
        return Self::try_capture().unwrap_or(Self { frames: Vec::new() });

        #[cfg(not(all(target_arch = "arm", feature = "backtraces")))]
        return Self { frames: Vec::new() };
    }

    /// Captures a backtrace at the current point of execution,
    /// returning an error if the backtrace fails to capture.
    #[inline(never)] // Make sure there's alawys a frame to remove
    #[cfg(all(target_arch = "arm", feature = "backtraces"))]
    pub fn try_capture() -> Result<Self, UnwindError> {
        let context = UnwindContext::new()?;
        let mut cursor = UnwindCursor::new(&context)?;

        let mut frames = Vec::new();

        // Procedure based on mini_backtrace crate.

        // Step once before taking the backtrace to skip the current frame.
        while cursor.step()? {
            let mut instruction_pointer = cursor.register(registers::UNW_REG_IP)?;

            // Adjust IP to point inside the function — this improves symbolization quality.
            if !cursor.is_signal_frame()? {
                instruction_pointer -= 1;
            }

            frames.push(instruction_pointer as *const c_void);
        }

        Ok(Self { frames })
    }
}

impl Display for Backtrace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "stack backtrace:")?;
        for (i, frame) in self.frames.iter().enumerate() {
            writeln!(f, "{i:>3}: {:?}", frame)?;
        }
        write!(
            f,
            "note: Use a symbolizer to convert stack frames to human-readable function names."
        )?;
        Ok(())
    }
}
