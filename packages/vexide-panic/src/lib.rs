//! Panic handler for [`vexide`](https://crates.io/crates/vexide).
//!
//! Supports capturing and printing backtraces to aid in debugging.
//!
//! If the `display_panics` feature is enabled, it will also display the panic message on the V5 Brain display.

#![no_std]

extern crate alloc;

use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use core::sync::atomic::{AtomicBool, Ordering};
use core::fmt::Write;

use vexide_core::{backtrace::Backtrace, println, sync::Mutex};
#[cfg(feature = "display_panics")]
use vexide_devices::{
    display::{Display, Font, FontFamily, FontSize, Rect, Text},
    math::Point2,
};

static FIRST_PANIC: AtomicBool = AtomicBool::new(true);

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
        screen.draw_text(
            &Text::new(
                buffer,
                Font::new(FontSize::SMALL, FontFamily::Monospace),
                Point2 {
                    x: ERROR_BOX_MARGIN + ERROR_BOX_PADDING,
                    y: ERROR_BOX_MARGIN + ERROR_BOX_PADDING + (line * LINE_HEIGHT),
                },
            ),
            (255, 255, 255),
            None,
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
/// It will print the panic message to the serial connection, and if the
/// `display_panics` feature is enabled, it will also display the panic message
/// on the V5 Brain display.
///
/// Note that if `display_panics` is not enabled, this function will not return.
/// It will immediately exit the program after printing the panic message. If
/// you do not want this behavior, you should use your own
///
/// # Examples
///
/// ```
/// # use vexide_panic::{default_panic_hook, set_hook};
/// #
/// set_hook(|info| {
///     // Do something interesting with the panic info, like printing it to the
///     // controller screen so the driver knows something has gone wrong.
///     // ...
///
///     // Then, call the default panic hook to show the message on the screen
///     default_panic_hook(info);
/// });
/// ```
pub fn default_panic_hook(info: &core::panic::PanicInfo<'_>) {
    println!("{info}");

    let backtrace = Backtrace::capture();

    #[cfg(feature = "display_panics")]
    draw_error(
        &mut unsafe { Display::new() },
        &info.to_string(),
        &backtrace,
    );

    if !backtrace.frames.is_empty() {
        println!("{backtrace}");
    }

    #[cfg(not(feature = "display_panics"))]
    vexide_core::program::exit();
}

/// The panic hook type
///
/// This mirrors the one available in the standard library.
enum Hook {
    Default,
    Custom(Box<dyn Fn(&core::panic::PanicInfo<'_>) + Send>),
}

/// A word-for-word copy of the Rust `std` impl
impl Hook {
    #[inline]
    fn into_box(self) -> Box<dyn Fn(&core::panic::PanicInfo<'_>) + Send> {
        match self {
            Hook::Default => Box::new(default_panic_hook),
            Hook::Custom(hook) => hook,
        }
    }
}

static HOOK: Mutex<Hook> = Mutex::new(Hook::Default);

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
/// panic hook, or call [`default_panic_hook`] from your hook.
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
    F: Fn(&core::panic::PanicInfo<'_>) + Send + 'static,
{
    // Try to lock the mutex. This should always succeed since the mutex is only
    // locked when the program panics and by the set_hook and take_hook
    // functions, which don't panic while holding a lock.
    let mut guard = HOOK
        .try_lock()
        .expect("failed to set custom panic hook (mutex poisoned or locked)");
    // If we used a simple assignment, like
    // *guard = Hook::Custom(Box::new(hook));
    // the old value will be dropped. Since the old value is `dyn`, it
    // could have arbitrary side effects in its destructor, *including
    // panicking*. We need to avoid panicking here, since the panic
    // handler would not be able to lock the mutex, which is kind of
    // important.
    // Don't do anything that could panic until guard is dropped
    let old_handler = core::mem::replace(&mut *guard, Hook::Custom(Box::new(hook))).into_box();
    // Drop the guard first to avoid a deadlock
    core::mem::drop(guard);
    // Now we can drop the old handler
    core::mem::drop(old_handler); // This could panic
}

/// Unregisters the current panic hook, if any, and returns it, replacing it
/// with the default panic hook.
///
/// The default panic hook will remain registered if no custom hook was set.
pub fn take_hook() -> Box<dyn Fn(&core::panic::PanicInfo<'_>) + Send> {
    // Try to lock the mutex. This should always succeed since the mutex is only
    // locked when the program panics and by the set_hook and take_hook
    // functions, which don't panic while holding a lock.
    let mut guard = HOOK
        .try_lock()
        .expect("failed to set custom panic hook (mutex locked)");
    // Don't do anything that could panic until guard is dropped
    let old_hook = core::mem::replace(&mut *guard, Hook::Default).into_box();
    core::mem::drop(guard);
    old_hook
}

/// The panic handler for vexide.
#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    // This can only occur if the panic handler itself has panicked (which can
    // happen in hooks or if println!() fails), resulting in a potential stack
    // overflow. In this instance, something is likely very wrong, so it's better
    // to just abort rather than recursively panicking.
    if !FIRST_PANIC.swap(false, Ordering::Relaxed) {
        vexide_core::program::exit();
    }

    // Try to lock the HOOK mutex. If we can't, we'll just use the default panic
    // handler, since it's probably not good to panic in the panic handler and
    // leave the user clueless about what happened.
    //
    // We should be able to lock the mutex, since we know that the mutex is only
    // otherwise locked in `take_hook` and `set_hook`, which don't panic.

    // Allow if_let_rescope lint since we actually prefer the Rust 2024 rescope
    // in this case.
    // Formerly, in the `else` branch, the lock would be held to the end of the
    // block, but it doesn't in the 2024 edition of Rust. That behavior is
    // actually preferable here.
    #[allow(if_let_rescope)]
    if let Some(mut guard) = HOOK.try_lock() {
        let hook = core::mem::replace(&mut *guard, Hook::Default);
        // Drop the guard first to avoid preventing set_hook or take_hook from
        // getting a hook
        core::mem::drop(guard);
        (hook.into_box())(info);
    } else {
        // Since this is in theory unreachable, if it is reached, let's ask the
        // user to file a bug report.
        // FIXME: use eprintln once armv7a-vex-v5 support in Rust is merged
        println!("Panic handler hook mutex was locked, so the default panic hook will be used. This should never happen.");
        println!("If you see this, please consider filing a bug: https://github.com/vexide/vexide/issues/new");
        default_panic_hook(info);
    }

    // enter into an endless loop if the panic hook didn't exit the program
    loop {
        unsafe {
            // Flush the serial buffer so that the panic message is printed
            vex_sdk::vexTasksRun();
        }
    }
}
