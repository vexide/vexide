//! Support for capturing stack backtraces.
//!
//! This module contains support for capturing a stack backtrace through the [`Backtrace`] and
//! [`BacktraceIter`] structs. Obtaining a backtrace gives you a list of every function that was
//! running at the time of its capture, with more recently called functions being ordered first.
//! Backtraces can be helpful to attach to errors since they allow you to determine the indirect
//! causes of a failure.
//!
//! # Capturing backtraces
//!
//! Calling [`Backtrace::capture`] gives you a list of the addresses of each function currently
//! running.
//!
//! ```
//! # use vexide::backtrace::Backtrace;
//! #
//! let backtrace = Backtrace::capture();
//! println!("{backtrace}");
//!
//! for frame in backtrace.frames() {
//!     println!("{frame:?}");
//! }
//! ```
//!
//! You can also use [`BacktraceIter::capture`] to similar effect. This a more lightweight API that
//! doesn't require memory allocation and allows you to cancel the capture. However, it only allows
//! you to access the iterator inside a callback:
//!
//! ```
//! # use vexide::backtrace::BacktraceIter;
//! #
//! // Only capture up to 5 backtrace frames.
//! let frames: Vec<_> = BacktraceIter::capture(|backtrace| backtrace.take(5).collect());
//!
//! // Print each frame as it's captured.
//! BacktraceIter::capture(|backtrace| {
//!     for frame in backtrace {
//!         println!("- {frame:?}");
//!     }
//! });
//! ```
//!
//! # Using the backtrace
//!
//! Each frame in backtrace is an address, which isn't particularly meaningful on its own but can be
//! combined with a built executable to become a file name, line number, and function name. Doing
//! this shows which functions were being run at the time of the backtrace's capture. Performing the
//! conversion requires a *symbolizer tool* such as `llvm-symbolizer` or `addr2line`.
//!
//! For instance, if you were shown the following backtrace, you could symbolize it as shown.
//!
//! ```txt
//! stack backtrace:
//!   0: 0x380217b
//!   1: 0x380209b
//! note: Use a symbolizer to convert stack frames to human-readable function names.
//! ```
//!
//! ```terminal
//! $ llvm-symbolizer -p -e ./target/armv7a-vex-v5/debug/program_name 0x380217b 0x380209b
//! my_function at src/main.rs:30:14
//!
//! main at src/main.rs:21:9
//! ```
//!
//! This result would indicate that at the time of the backtrace, the program was running line 30 of
//! `main.rs` (inside a function named `my_function`), and was running that code because of a
//! function call on line 21 of `main.rs` (inside `main`).
//!
//! # Platform Support
//!
//! The backtrace API is only functional when compiling for VEXos; using it on another platform will
//! result in an empty backtrace.

use alloc::{fmt, vec::Vec};
use core::{fmt::Display, marker::PhantomData};

/// A captured stack backtrace.
///
/// This type stores the backtrace of a captured stack at a certain point in time. The backtrace is
/// represented as a list of instruction pointers.
///
/// # Example
///
/// ```
/// use vexide::backtrace::Backtrace;
///
/// let backtrace = Backtrace::capture();
/// println!("{backtrace}");
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
    /// Backtraces will be empty when compiling to a non-VEXos platform.
    #[allow(clippy::inline_always)]
    #[inline(always)] // Inlining keeps this function from appearing in backtraces
    #[allow(clippy::missing_const_for_fn)]
    #[must_use = "this creates a backtrace but doesn't save it or log it anywhere"]
    pub fn capture() -> Self {
        BacktraceIter::capture(|bt| {
            let mut frames = Vec::new();
            for frame in bt {
                frames.push(frame);
            }
            Self { frames }
        })
    }

    /// Returns a slice of instruction pointers at every captured frame of the backtrace.
    #[must_use]
    pub const fn frames(&self) -> &[*const ()] {
        self.frames.as_slice()
    }
}

impl Display for Backtrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        format_backtrace(f, self.frames.iter().copied())
    }
}

/// An iterator over the frames of a stack backtrace.
///
/// This is a more lightweight alternative to [`Backtrace`]. Rather than allocating a list on the
/// heap containing every frame, the stack is walked lazily as [`next`](BacktraceIter::next)
/// is called. This makes it possible to capture frames without allocation or stop walking the stack
/// early.
///
/// Because the iterator inspects the active call stack, it cannot outlive the capture and can
/// only be used from within the closure passed to [`capture`](BacktraceIter::capture). As with
/// [`Backtrace`], the frames of the most recently called function are returned first.
///
/// # Example
///
/// ```
/// use vexide::backtrace::BacktraceIter;
///
/// // Print the five most recent frames.
/// BacktraceIter::capture(|backtrace| {
///     for frame in backtrace.take(5) {
///         println!("- {frame:?}");
///     }
/// });
/// ```
pub struct BacktraceIter<'a> {
    // Prevents "Unused lifetime" error on non-vexos targets.
    lifetime: PhantomData<&'a ()>,
    #[cfg(all(target_arch = "arm", target_os = "vexos"))]
    frame: Option<&'a arm::FrameRecord>,
}

impl Iterator for BacktraceIter<'_> {
    type Item = *const ();

    fn next(&mut self) -> Option<Self::Item> {
        #[cfg(target_os = "vexos")]
        if let Some(frame) = self.frame {
            let addr = frame.caller_address();
            self.frame = frame.next();
            return Some(addr as *const ());
        }

        None
    }
}

impl BacktraceIter<'_> {
    /// Captures a backtrace at the current point of execution, passing an iterator over its frames
    /// to the given closure.
    ///
    /// The iterator borrows from the current call stack and so cannot escape the closure. In
    /// exchange, capturing this way performs no heap allocation and lets you stop walking the stack
    /// early by simply not advancing the iterator any further.
    ///
    /// # Platform Support
    ///
    /// The iterator yields no frames when compiling to a non-VEXos platform.
    pub fn capture<R>(handler: impl FnOnce(BacktraceIter<'_>) -> R) -> R {
        cfg_select! {
            all(
                target_arch = "arm",
                target_os = "vexos",
            ) => {
                arm::capture_frame(|frame| {
                    handler(BacktraceIter {
                        lifetime: PhantomData,
                        frame: Some(frame),
                    })
                })
            }
            _ => handler(BacktraceIter {
                lifetime: PhantomData,
            })
        }
    }

    /// Capture a backtrace by walking the stack, starting at the given [AAPCS32 frame pointer].
    ///
    /// This function allows you to capture a backtrace from a different call stack, but does not
    /// ensure that the specified pointer or lifetime is valid. Generally, a valid frame pointer can
    /// be obtained by reading either `r7` in thumb mode or `r11` (aka `fp`) in ARM mode.
    ///
    /// # Safety
    ///
    /// `ptr` must either be null or an AAPCS32 frame pointer that's valid for the span of this
    /// type's lifetime.
    ///
    /// [AAPCS32 frame pointer]: https://github.com/ARM-software/abi-aa/blob/main/aapcs32/aapcs32.rst#6214the-frame-pointer
    #[must_use = "the backtrace isn't captured unless you step the iterator"]
    #[allow(
        clippy::missing_const_for_fn,
        reason = "performs non-const operations on some targets"
    )]
    pub unsafe fn from_frame_ptr(ptr: *const ()) -> Self {
        _ = ptr; // ptr is unused on non-VEXos targets.
        Self {
            lifetime: PhantomData,
            #[cfg(all(target_arch = "arm", target_os = "vexos"))]
            frame: unsafe { arm::FrameRecord::from_ptr(ptr.cast()) },
        }
    }

    /// Consumes the iterator, writing a human-readable representation of the backtrace to `dest`.
    ///
    /// This produces the same output as formatting a [`Backtrace`] with [`Display`].
    ///
    /// # Errors
    ///
    /// Returns an error if the underling write operation fails.
    ///
    /// [`Display`]: core::fmt::Display
    pub fn write_to(self, dest: &mut impl fmt::Write) -> fmt::Result {
        format_backtrace(dest, self)
    }
}

/// Writes a human-readable representation of a backtrace to `dest`.
fn format_backtrace(
    dest: &mut impl fmt::Write,
    frames: impl Iterator<Item = *const ()>,
) -> fmt::Result {
    writeln!(dest, "stack backtrace:")?;
    for (i, frame) in frames.enumerate() {
        writeln!(dest, "{i:>3}: 0x{:x}", frame as usize)?;
    }
    write!(
        dest,
        "note: Use a symbolizer to convert stack frames to human-readable function names."
    )
}

#[cfg(all(target_arch = "arm", target_os = "vexos"))]
mod arm {
    use core::{arch::asm, ptr};

    unsafe extern "C" {
        unsafe static __stack_top: u32;
        unsafe static __stack_bottom: u32;
    }

    /// The AAPCS32 frame record.
    ///
    /// Stores information about the state of the CPU upon entry to a function.
    #[repr(C)]
    pub struct FrameRecord {
        /// The caller's frame record.
        caller: *const FrameRecord,
        /// The value stored in LR upon entry to this function.
        lr: u32,
    }

    impl FrameRecord {
        /// Accesses a frame record using the given frame pointer.
        ///
        /// Returns `None` if the given pointer is null or out of range.
        ///
        /// # Safety
        ///
        /// `ptr` must either be null or a frame pointer that's valid for the span of the specified
        /// lifetime.
        #[must_use]
        pub unsafe fn from_ptr<'a>(ptr: *const Self) -> Option<&'a Self> {
            // Frame records are only ever stored on the stack.
            let stack = (&raw const __stack_bottom)..(&raw const __stack_top);
            if !stack.contains(&ptr.cast()) {
                return None;
            }

            // SAFETY: ptr is either a valid frame pointer or null.
            unsafe { ptr.as_ref() }
        }

        /// Returns the frame record of this frame's caller, or [`None`] if this is the last frame.
        #[must_use]
        pub fn next(&self) -> Option<&Self> {
            // The caller's stack frame should always be closer to the base of the stack.
            // (In other words, it should have a higher address.) This check works to prevent
            // any infinite loops.
            if self.caller <= ptr::from_ref(self) {
                return None;
            }

            // SAFETY: Since the call frame corresponding to this frame record is still active, its
            // caller must still be active as well, so `caller` is still valid.
            unsafe { Self::from_ptr(self.caller) }
        }

        /// Returns the address of the instruction that called into this frame.
        #[must_use]
        pub const fn caller_address(&self) -> u32 {
            const THUMB_BIT: u32 = 0b1;
            // Some instructions are bigger than this, but a symbolizer can still figure it out
            // if we give it an address in the middle of an instruction.
            const INSTR_SIZE: u32 = 2;

            // LR holds the instruction *after* the caller's address since that's where a function
            // would return to, but we care about which instruction actually called us. Go back 1
            // instruction to report that instead.
            (self.lr & !THUMB_BIT) - INSTR_SIZE
        }
    }

    /// Reads the active frame record and passes it to `handler`.
    ///
    /// The first record passed to `handler` is a synthetic frame record that represents the current
    /// function and acts as a starting point. Call [`FrameRecord::next`] to advance up the call
    /// stack.
    pub fn capture_frame<R>(handler: impl FnOnce(&FrameRecord) -> R) -> R {
        let caller: *const FrameRecord;
        let lr: u32;

        // SAFETY: By convention, r7 is the Thumb frame pointer and fp is the ARM frame pointer.
        // https://github.com/rust-lang/rust/blob/c397dae808f70caebab1fc4e11b3edf7e59f58c7/compiler/rustc_target/src/asm/arm.rs#L70
        unsafe {
            cfg_select! {
                target_feature = "thumb-mode" => asm!(
                    "mov {caller}, r7",
                    "mov {lr}, pc",
                    caller = out(reg) caller,
                    lr = out(reg) lr,
                ),
                _ => asm!(
                    "mov {caller}, fp",
                    "mov {lr}, pc",
                    caller = out(reg) caller,
                    lr = out(reg) lr,
                ),
            };
        }

        handler(&FrameRecord { caller, lr })
    }
}
