//! Synchronization types for FreeRTOS tasks.
//!
//! Types implemented here are specifically designed to mimic the standard library.

use core::{cell::UnsafeCell, fmt::Debug, sync::atomic::{ AtomicU8, Ordering}};

struct MutexState(AtomicU8);
impl MutexState {
    const fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    /// Returns true if the lock was acquired.
    fn try_lock(&self) -> bool {
        self.0.compare_exchange(0, 1, Ordering::Acquire, Ordering::Acquire).is_ok()
    }

    fn unlock(&self) {
        self.0.store(0, Ordering::Release);
    }

    fn poison(&self) {
        self.0.store(2, Ordering::Release);
    }

    fn is_poisoned(&self) -> bool {
        self.0.load(Ordering::Acquire) == 2
    }

    fn is_locked(&self) -> bool {
        self.0.load(Ordering::Acquire) == 1
    }

    fn is_unlocked(&self) -> bool {
        self.0.load(Ordering::Acquire) == 0
    }
}

/// The basic mutex type.
/// Mutexes are used to share variables between tasks safely.
pub struct Mutex<T> {
    state: MutexState,
    data: UnsafeCell<T>,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    pub const fn new(data: T) -> Self {
        Self {
            state: MutexState::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Locks the mutex so that it cannot be locked in another task at the same time.
    /// Blocks the current task until the lock is acquired.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        while !self.state.try_lock() {
            if self.state.is_poisoned() {
                panic!("Mutex poisoned");
            }
        }

        MutexGuard::new(self)
    }

    /// Attempts to acquire this lock. This function does not block.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.state.try_lock() {
            Some(MutexGuard { mutex: self })
        } else {
            None
        }
    }

    /// Consumes the mutex and returns the inner data.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Gets a mutable reference to the inner data.
    pub fn get_mut(&mut self) -> &mut T {
        self.data.get_mut()
    }
}

impl<T> Debug for Mutex<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        struct Placeholder;
        impl Debug for Placeholder {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("<locked>")
            }
        }

        let mut d = f.debug_struct("Mutex");
        match self.try_lock() {
            Some(guard) => d.field("data", &&*guard),
            None => d.field("data", &Placeholder),
        };
        d.finish_non_exhaustive()
    }
}

impl<T> Default for Mutex<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// Allows the user to access the data from a locked mutex.
/// Dereference to get the inner data.
#[derive(Debug)]
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> MutexGuard<'a, T> {
    const fn new(mutex: &'a Mutex<T>) -> Self {
        Self { mutex }
    }
}

impl<T> core::ops::Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> core::ops::DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.state.unlock();
    }
}
