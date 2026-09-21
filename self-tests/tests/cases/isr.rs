//! Interrupt service routine tests.
//!
//! These check that a user-installed Armv7-A IRQ handler can run alongside the main execution
//! context and synchronize with it using `std::sync`, as described in the `thumbv7a-vex-v5`
//! platform support docs.

use std::{
    hint::spin_loop,
    sync::{
        Mutex,
        atomic::{AtomicU32, AtomicU64, Ordering},
    },
    thread::{self, Thread},
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

/// A `Mutex` shared with an ISR should hand out the lock to one side at a time, and `try_lock`
/// should back off rather than deadlock when the main context already holds it.
pub async fn test_irq_mutex_try_lock(_peripherals: Peripherals) -> Result<(), Failed> {
    static DATA: Mutex<u64> = Mutex::new(0);

    let mut data_last = 0;

    vectors::with_irq(
        || {
            if let Ok(mut data) = DATA.try_lock() {
                *data += 1;
            }
        },
        || {
            // Spin long enough to be sure several interrupts arrive. Interrupts increment `DATA`,
            // so it must never be seen going backwards or torn.
            let deadline = unsafe { vexSystemTimeGet() } + 50;
            while unsafe { vexSystemTimeGet() } < deadline {
                spin_loop();
                let data = *DATA.lock().unwrap();
                assert!(data >= data_last);
                data_last = data;
            }
        },
    );

    assert_ne!(data_last, 0);

    Ok(())
}

/// Unparking the main thread from an ISR should wake it back up.
pub async fn test_irq_thread_unpark(_peripherals: Peripherals) -> Result<(), Failed> {
    /// How long the ISR waits before it starts unparking.
    const DELAY: u32 = 50;
    /// Bails out if the ISR never manages to unpark us.
    const TIMEOUT: Duration = Duration::from_millis(500);

    static THREAD: Mutex<Option<Thread>> = Mutex::new(None);
    static WAKEUP_TIME: AtomicU32 = AtomicU32::new(0);

    let begin = unsafe { vexSystemTimeGet() };
    WAKEUP_TIME.store(begin + DELAY, Ordering::SeqCst);
    *THREAD.lock().unwrap() = Some(thread::current());

    vectors::with_irq(
        || {
            // Start spamming unpark once the delay has elapsed.
            if unsafe { vexSystemTimeGet() } < WAKEUP_TIME.load(Ordering::SeqCst) {
                return;
            }

            if let Ok(thrd) = THREAD.try_lock()
                && let Some(thrd) = &*thrd
            {
                thrd.unpark();
            }
        },
        || thread::park_timeout(TIMEOUT),
    );

    let elapsed = unsafe { vexSystemTimeGet() } - begin;
    *THREAD.lock().unwrap() = None;

    assert!(
        elapsed >= DELAY,
        "woke up after {elapsed}ms, before the ISR started unparking"
    );
    assert!(
        Duration::from_millis(u64::from(elapsed)) < TIMEOUT,
        "timed out after {elapsed}ms without being unparked"
    );

    Ok(())
}
