//! Interrupt service routine tests.
//!
//! These check that a user-installed Armv7-A IRQ handler can run alongside the main execution
//! context and synchronize with it using atomics. The `thumbv7a-vex-v5` platform support docs
//! treat exception handlers like UNIX signal handlers, so they must not use `std::sync`,
//! `thread_local!`, or `thread::current`.

use std::{
    sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    thread,
    time::Duration,
};

use libtest_mimic::Failed;
use vex_sdk::vexSystemTimeGet;
use vexide::peripherals::Peripherals;

mod vectors {
    use std::{
        arch::{asm, global_asm},
        mem, ptr,
        sync::atomic::{AtomicPtr, Ordering},
    };

    // This is based off the setup in vexide-startup, so see that for more information. Every
    // exception other than IRQ is forwarded to vexide's own handler.
    global_asm!(
        include_str!("vectors.s"),
        irq_handler = sym irq_handler,
    );

    static IRQ_CALLBACK: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

    extern "C" fn irq_handler() {
        let ptr = IRQ_CALLBACK.load(Ordering::Acquire);
        if ptr.is_null() {
            return;
        }

        // SAFETY: `IRQ_CALLBACK` only ever holds a `fn()` written by `with_irq`.
        let callback = unsafe { mem::transmute::<*mut (), fn()>(ptr) };
        callback();

        // Note: normal IRQ handling is handled by the selftest_irq function in `vectors.s`.
    }

    /// Points `VBAR` at the given vector table.
    ///
    /// The symbol must name a 32-byte aligned Armv7-A vector table.
    macro_rules! set_vbar {
        ($table:literal) => {
            // SAFETY: Both tables handle every exception the CPU can raise.
            unsafe {
                asm!(
                    concat!("movw r0, #:lower16:", $table),
                    concat!("movt r0, #:upper16:", $table),
                    // Set VBAR; see <https://developer.arm.com/documentation/ddi0601/2025-06/AArch32-Registers/VBAR--Vector-Base-Address-Register>
                    "mcr p15, 0, r0, c12, c0, 0",
                    "isb",
                    out("r0") _,
                    options(nostack, preserves_flags)
                );
            }
        };
    }

    /// Runs `body` with `callback` installed as the IRQ handler.
    ///
    /// vexide's vector table is put back before returning. Note that this does not happen if
    /// `body` panics, since this target aborts on panic.
    pub fn with_irq<R>(callback: fn(), body: impl FnOnce() -> R) -> R {
        IRQ_CALLBACK.store(callback as *mut (), Ordering::Release);
        set_vbar!("selftest_vector_table");

        let result = body();

        set_vbar!("vector_table");
        IRQ_CALLBACK.store(ptr::null_mut(), Ordering::Release);

        result
    }
}

/// Interrupts should keep firing while a user vector table is installed.
pub async fn test_irqs_received(_peripherals: Peripherals) -> Result<(), Failed> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    vectors::with_irq(
        || {
            COUNTER.fetch_add(1, Ordering::Relaxed);
        },
        || thread::sleep(Duration::from_millis(10)),
    );

    assert_ne!(COUNTER.load(Ordering::Relaxed), 0);

    Ok(())
}

/// An ISR should be able to wake the main context by setting an atomic flag that the main context
/// polls while yielding to VEXos.
pub async fn test_irq_atomic_flag_wakes_main(_peripherals: Peripherals) -> Result<(), Failed> {
    /// How long the ISR waits before it sets the flag.
    const DELAY: u32 = 50;
    /// Bails out if the ISR never sets the flag.
    const TIMEOUT: u32 = 500;

    static WAKEUP_TIME: AtomicU32 = AtomicU32::new(0);
    static FLAG: AtomicBool = AtomicBool::new(false);

    FLAG.store(false, Ordering::SeqCst);
    let begin = unsafe { vexSystemTimeGet() };
    WAKEUP_TIME.store(begin + DELAY, Ordering::SeqCst);

    let success = vectors::with_irq(
        || {
            // This ISR might set the flag multiple times, but it shouldn't be an issue since we
            // always assign the same value - we just need to know that it ran at least once.
            if unsafe { vexSystemTimeGet() } >= WAKEUP_TIME.load(Ordering::SeqCst) {
                FLAG.store(true, Ordering::Release);
            }
        },
        || {
            loop {
                if FLAG.load(Ordering::Acquire) {
                    return true;
                }
                if unsafe { vexSystemTimeGet() } - begin >= TIMEOUT {
                    return false;
                }
                thread::yield_now();
            }
        },
    );

    let elapsed = unsafe { vexSystemTimeGet() } - begin;

    assert!(
        elapsed >= DELAY,
        "woke up after {elapsed}ms, before the ISR set the flag"
    );
    assert!(
        success,
        "timed out after {elapsed}ms without the flag being set"
    );

    Ok(())
}
