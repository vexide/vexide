/// The basic mutex type.
/// Mutexes are used to share variables between tasks safely.
pub struct Mutex<T> {
    pros_mutex: pros_sys::mutex_t,
    data: core::cell::UnsafeCell<T>,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    pub fn new(data: T) -> Self {
        let pros_mutex = unsafe { pros_sys::mutex_create() };

        Self {
            pros_mutex,
            data: core::cell::UnsafeCell::new(data),
        }
    }

    /// Locks the mutex so that it cannot be locked in another task at the same time.
    /// Blocks the current task untill the lock is acquired.
    pub fn lock(&self) -> MutexGuard<T> {
        unsafe {
            pros_sys::mutex_take(self.pros_mutex, pros_sys::TIMEOUT_MAX);
        }
        MutexGuard { mutex: self }
    }
}

impl<T> Drop for Mutex<T> {
    fn drop(&mut self) {
        unsafe {
            pros_sys::mutex_delete(self.pros_mutex);
        }
    }
}

/// Allows the user to access the data from a locked mutex.
/// Dereference to get the inner data.
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
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
        unsafe {
            pros_sys::mutex_give(self.mutex.pros_mutex);
        }
    }
}
