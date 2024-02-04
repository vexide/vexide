//! Synchronization types for FreeRTOS tasks.
//!
//! Types implemented here are specifically designed to mimic the standard library.

use core::{cell::UnsafeCell, fmt::Debug, mem};

use pros_core::error::take_errno;

/// The basic mutex type.
/// Mutexes are used to share variables between tasks safely.
pub struct Mutex<T> {
    pros_mutex: pros_sys::mutex_t,
    data: Option<UnsafeCell<T>>,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    pub fn new(data: T) -> Self {
        let pros_mutex = unsafe { pros_sys::mutex_create() };

        Self {
            pros_mutex,
            data: Some(UnsafeCell::new(data)),
        }
    }

    /// Locks the mutex so that it cannot be locked in another task at the same time.
    /// Blocks the current task until the lock is acquired.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        if !unsafe { pros_sys::mutex_take(self.pros_mutex, pros_sys::TIMEOUT_MAX) } {
            panic!("Mutex lock failed: {}", take_errno());
        }

        MutexGuard { mutex: self }
    }

    /// Attempts to acquire this lock. This function does not block.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        let success = unsafe { pros_sys::mutex_take(self.pros_mutex, 0) };
        success.then(|| MutexGuard::new(self))
    }

    /// Consumes the mutex and returns the inner data.
    pub fn into_inner(mut self) -> T {
        let data = mem::take(&mut self.data).unwrap();
        data.into_inner()
    }

    /// Gets a mutable reference to the inner data.
    pub fn get_mut(&mut self) -> &mut T {
        self.data.as_mut().unwrap().get_mut()
    }
}

impl<T> Drop for Mutex<T> {
    fn drop(&mut self) {
        unsafe {
            pros_sys::mutex_delete(self.pros_mutex);
        }
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
        unsafe { &*self.mutex.data.as_ref().unwrap().get() }
    }
}

impl<T> core::ops::DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.as_ref().unwrap().get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            pros_sys::mutex_give(self.mutex.pros_mutex);
        }
    }
}
