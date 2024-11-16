//! Panic handler implementation for [`vexide`](https://crates.io/crates/vexide)
//!
//! Supports capturing and printing backtraces to aid in debugging.
//!
//! If the `display_panics` feature is enabled, it will also display the panic message on the V5 Brain display.

#![no_std]

extern crate alloc;

#[allow(unused_imports)]
use alloc::{
    boxed::Box,
    string::{String, ToString},
};
#[allow(unused_imports)]
use core::{cell::UnsafeCell, fmt::Write};

use vexide_core::{backtrace::Backtrace, println};
#[cfg(feature = "display_panics")]
use vexide_devices::{
    display::{Display, Rect, Text, TextSize},
    math::Point2,
};

#[cfg(target_arch = "wasm32")]
extern "C" {
    /// Prints a backtrace to the debug console
    fn sim_log_backtrace();
}

/// Draw an error box to the display.
///
/// This function is internally used by the vexide panic handler for displaying
/// panic messages graphically before exiting.
#[cfg(feature = "display_panics")]
fn draw_error(display: &mut Display, msg: &str, backtrace: &Backtrace) {
    const ERROR_BOX_MARGIN: i16 = 16;
    const ERROR_BOX_PADDING: i16 = 16;
    const LINE_HEIGHT: i16 = 20;
    const LINE_MAX_WIDTH: usize = 52;

    fn draw_text(screen: &mut Display, buffer: &str, line: i16) {
        screen.fill(
            &Text::new(
                buffer,
                TextSize::Small,
                Point2 {
                    x: ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                    y: ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * LINE_HEIGHT),
                },
            ),
            (255, 255, 255),
        );
    }

    display.set_render_mode(vexide_devices::display::RenderMode::Immediate);

    let error_box_rect = Rect::new(
        Point2 {
            x: ERROR_BOX_MARGIN,
            y: ERROR_BOX_MARGIN,
        },
        Point2 {
            x: Display::HORIZONTAL_RESOLUTION - ERROR_BOX_MARGIN,
            y: Display::VERTICAL_RESOLUTION - ERROR_BOX_MARGIN,
        },
    );

    display.fill(&error_box_rect, (255, 0, 0));
    display.stroke(&error_box_rect, (255, 255, 255));

    let mut buffer = String::new();
    let mut line: i16 = 0;

    for (i, character) in msg.char_indices() {
        if !character.is_ascii_control() {
            buffer.push(character);
        }

        if character == '\n' || ((buffer.len() % LINE_MAX_WIDTH == 0) && (i > 0)) {
            draw_text(display, &buffer, line);
            line += 1;
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        draw_text(display, &buffer, line);

        line += 1;
    }

    line += 1;
    draw_text(display, "stack backtrace:", line);
    line += 1;

    if !backtrace.frames.is_empty() {
        const ROW_LENGTH: usize = 3;
        for (col, frames) in backtrace.frames.chunks(ROW_LENGTH).enumerate() {
            let mut msg = String::new();
            for (row, frame) in frames.iter().enumerate() {
                write!(msg, "{:>3}: {:?}    ", col * ROW_LENGTH + row, frame).unwrap();
            }
            draw_text(display, msg.trim_end(), line);
            line += 1;
        }
    }
}

/// The default panic handler.
///
/// This function is called when a panic occurs and no custom panic hook is set,
/// but you can also use it as a fallback in your custom panic hook to print the
/// panic message to the screen and stdout.
///
/// # Examples
///
/// ```
/// # use vexide_panic::{default_panic_hook, set_hook};
/// #
/// set_hook(|info| {
///     // Do something bespoke with the panic info
///     // ...
///
///     // Then, call the default panic hook to show the message on the screen
///     default_panic_hook(info);
/// });
/// ```
pub fn default_panic_hook(info: &core::panic::PanicInfo<'_>) {
    println!("{info}");

    let backtrace = Backtrace::capture();
    #[cfg(not(target_arch = "wasm32"))]
    if !backtrace.frames.is_empty() {
        println!("{backtrace}");
    }

    #[cfg(feature = "display_panics")]
    draw_error(
        unsafe { &mut Display::new() },
        &info.to_string(),
        &backtrace,
    );

    #[cfg(target_arch = "wasm32")]
    unsafe {
        sim_log_backtrace();
    }
}

/// The panic hook type
///
/// This mirrors the one available in the standard library.
enum Hook {
    Default,
    Custom(Box<dyn Fn(&core::panic::PanicInfo<'_>)>),
}

/// A word-for-word copy of the Rust `std` impl
impl Hook {
    #[inline]
    fn into_box(self) -> Box<dyn Fn(&core::panic::PanicInfo<'_>)> {
        match self {
            Hook::Default => Box::new(default_panic_hook),
            Hook::Custom(hook) => hook,
        }
    }
}

/// A newtype wrapper over the hook type so we can implement `Sync`
/// This is only safe because we're single-threaded
struct HookWrapper(UnsafeCell<Hook>);
unsafe impl Sync for HookWrapper {}

static HOOK: HookWrapper = HookWrapper(UnsafeCell::new(Hook::Default));

/// Registers a custom panic hook, replacing the current one if any.
///
/// This can be used to, for example, output a different message to the screen
/// than the default one shown when the `display_panics` feature is enabled, or
/// to log the panic message to a log file or other output (you will need to use
/// `unsafe` to get peripheral access).
///
/// **Note**: Just like in the standard library, a custom panic hook _does
/// override_ the default panic hook. In `vexide`'s case, this means that the
/// error _will not automatically print to the screen or console_ when you set
/// a custom panic hook. You will need to either do that yourself in the custom
/// panic hook, or call [`default_panic_hook`] from your custom panic hook.
///
/// # Examples
///
/// ```
/// use vexide_panic::set_panic_hook;
///
/// set_hook(|info| {
///     // Do something with the panic info
///     // This is pretty useless since vexide already does this
///     println!("{:?}", info);
/// });
/// ```
pub fn set_hook<F>(hook: F)
where
    F: Fn(&core::panic::PanicInfo<'_>) + 'static,
{
    // SAFETY: We're expected to uphold the invariant "Ensure that the access
    // is unique (no active references, mutable or not) when casting to &mut T
    // We only obtain a reference to the `UnsafeCell` in this function and the
    // panic handler, and since this function never panics (please make sure!)
    // it should be safe.
    // Also, V5 is single-threaded so we don't need to worry about `Sync` issues
    unsafe {
        HOOK.0.get().write(Hook::Custom(Box::new(hook)));
    }
}

/// Unregisters the current panic hook, if any, and returns it, replacing it
/// with the default panic hook.
///
/// The default panic hook will remain registered if no custom hook was set.
pub fn take_hook() -> Box<dyn Fn(&core::panic::PanicInfo<'_>)> {
    // SAFETY: We're expected to uphold the invariant "Ensure that the access
    // is unique (no active references, mutable or not) when casting to &mut T
    // We only obtain a reference to the `UnsafeCell` in this function and the
    // panic handler, and since this function never panics it should be safe.
    unsafe { HOOK.0.get().replace(Hook::Default).into_box() }
}

#[panic_handler]
/// The panic handler for vexide.
pub fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    // SAFETY: We're expected to uphold the invariant "ensure that there are no
    // mutations or mutable aliases going on when casting to &T"
    // This is upheld by the fact that we only ever write to the `UnsafeCell` in
    // the `set_panic_hook` function, which should never panic while writing.
    //
    // We release a reference to the `UnsafeCell` to safe code because we will
    // never take a mutable reference again.
    let panic_hook = unsafe { HOOK.0.get().read().into_box() };
    panic_hook(info);

    #[cfg(not(feature = "display_panics"))]
    vexide_core::program::exit();
    // unreachable without display_panics
    #[cfg(feature = "display_panics")]
    loop {
        unsafe {
            // Flush the serial buffer so that the panic message is printed
            vex_sdk::vexTasksRun();
        }
    }
}
