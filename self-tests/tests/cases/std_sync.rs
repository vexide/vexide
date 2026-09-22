//! `std::sync` and `std::thread` blocking primitive tests.
//!
//! On this target these are all built on spinlocks that tick the VEXos scheduler through
//! `thread::yield_now`.

use std::{
    hint::spin_loop,
    sync::{Condvar, LazyLock, Mutex, Once, OnceLock, RwLock},
    thread,
    time::{Duration, Instant},
};

use libtest_mimic::Failed;
use vexide::{
    peripherals::Peripherals,
    smart::{SmartDeviceType, SmartPort},
};

/// Tests locking and unlocking a Rust ReentrantLock without contention.
pub async fn test_stdio_lock(_peripherals: Peripherals) -> Result<(), Failed> {
    for _ in 0..10_000 {
        _ = std::io::stdout().lock();
    }

    Ok(())
}

/// Tests locking and unlocking a Rust Mutex without contention.
pub async fn test_mutex_lock(_peripherals: Peripherals) -> Result<(), Failed> {
    let m = Mutex::new(());
    for _ in 0..10_000 {
        drop(m.lock()?);
    }

    Ok(())
}

/// Tests locking and unlocking a Rust RwLock without contention.
pub async fn test_rwlock_lock(_peripherals: Peripherals) -> Result<(), Failed> {
    let rw = RwLock::new(());
    for _ in 0..10_000 {
        drop(rw.write()?);
        let _r1 = rw.read()?;
        let _r2 = rw.read()?;
    }

    Ok(())
}

/// Tests initializing a Rust OnceLock and LazyLock.
pub async fn test_oncelock_lazylock(_peripherals: Peripherals) -> Result<(), Failed> {
    let once_lock = OnceLock::new();
    once_lock.get_or_init(|| 1);
    assert_eq!(once_lock.get(), Some(&1));

    let lazy_lock = LazyLock::new(|| 1);
    assert_eq!(*lazy_lock, 1);

    Ok(())
}

/// Tests invoking a Rust Once struct.
pub async fn test_call_once(_peripherals: Peripherals) -> Result<(), Failed> {
    let mut times = 0;

    let once = Once::new();
    once.call_once(|| times += 1);
    once.call_once(|| times += 1);

    assert_eq!(times, 1);

    Ok(())
}

// Rust std allows Condvars and thread parking to wake spuriously, but under VEX V5 this should not
// be the case as of the Rust version vexide is pinned to. This is not stable behavior but we can
// still use it to try to ensure things are mostly working properly.

/// Tests that Condvars under thumbv7a-vex-v5 time out properly when there are no events.
pub async fn test_condvar_timeout(_peripherals: Peripherals) -> Result<(), Failed> {
    let condvar = Condvar::new();
    let m = Mutex::new(());

    let now = Instant::now();

    let mlock = m.lock()?;
    let (_guard, result) = condvar
        .wait_timeout(mlock, Duration::from_millis(500))
        .unwrap();
    let after = Instant::now();

    assert!(result.timed_out());
    assert!((after - now) >= Duration::from_millis(500));

    Ok(())
}

/// Tests that `thread::park_timeout` unparks after the timeout.
pub async fn test_park_timeout(_peripherals: Peripherals) -> Result<(), Failed> {
    let now = Instant::now();
    thread::park_timeout(Duration::from_millis(500));
    let after = Instant::now();

    assert!((after - now) >= Duration::from_millis(500));

    Ok(())
}

/// Tests that `thread::sleep` and `thread::park_timeout` only yield on nonzero timeouts.
pub async fn test_duration_zero_no_yield(_peripherals: Peripherals) -> Result<(), Failed> {
    // We can detect if a yield happened by checking if the packet timestamp on the internal
    // ADI expander smart device has updated.
    // SAFETY: We do not hold any other references to this smart port.
    let adi = unsafe { SmartPort::new(22) };
    assert_eq!(adi.device_type(), Some(SmartDeviceType::Adi));

    let before = adi.timestamp().unwrap();

    // Wait 15ms to ensure there is a new packet ready. Packets are supposed to arrive every 10ms.
    while before.elapsed() < Duration::from_millis(15) {
        spin_loop();
    }

    thread::sleep(Duration::ZERO);
    // If we did yield to VEXos, the ADI expander should have a new packet.
    assert_eq!(adi.timestamp(), Some(before));

    thread::park_timeout(Duration::ZERO);
    assert_eq!(adi.timestamp(), Some(before));

    thread::yield_now();
    assert_ne!(adi.timestamp(), Some(before));

    Ok(())
}
