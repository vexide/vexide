use core::{
    cell::UnsafeCell,
    fmt::Debug,
    sync::atomic::{AtomicU8, Ordering},
};

use futures_core::Future;
use lock_api::RawMutex as _;

struct MutexState(AtomicU8);
impl MutexState {
    const fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    /// Returns true if the lock was acquired.
    fn try_lock(&self) -> bool {
        self.0
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Acquire)
            .is_ok()
    }

    fn unlock(&self) {
        self.0.store(0, Ordering::Release);
    }
}

/// A raw mutex type built on top of the critical section.
pub struct RawMutex {
    state: MutexState,
}
impl RawMutex {
    /// Creates a new raw mutex.
    pub const fn new() -> Self {
        Self {
            state: MutexState::new(),
        }
    }
}
unsafe impl lock_api::RawMutex for RawMutex {
    // Allow this because we can't get around it
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self::new();

    type GuardMarker = lock_api::GuardSend;

    fn lock(&self) {
        critical_section::with(|_| {
            while !self.state.try_lock() {
                core::hint::spin_loop();
            }
        })
    }

    fn try_lock(&self) -> bool {
        critical_section::with(|_| self.state.try_lock())
    }

    unsafe fn unlock(&self) {
        critical_section::with(|_| {
            self.state.unlock();
        })
    }
}

/// A future that resolves to a mutex guard.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct MutexLockFuture<'a, T> {
    mutex: &'a Mutex<T>,
}
impl<'a, T> Future for MutexLockFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        _: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if self.mutex.raw.try_lock() {
            core::task::Poll::Ready(MutexGuard::new(self.mutex))
        } else {
            core::task::Poll::Pending
        }
    }
}

/// The basic mutex type.
/// Mutexes are used to share variables between tasks safely.
pub struct Mutex<T> {
    raw: RawMutex,
    data: UnsafeCell<T>,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    pub const fn new(data: T) -> Self {
        Self {
            raw: RawMutex::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Locks the mutex so that it cannot be locked in another task at the same time.
    /// Blocks the current task until the lock is acquired.
    pub const fn lock(&self) -> MutexLockFuture<'_, T> {
        MutexLockFuture { mutex: self }
    }

    /// Used internally to lock the mutex in a blocking fashion.
    /// This is neccessary because a mutex may be created internally before the executor is ready to be initialized.
    pub(crate) fn lock_blocking(&self) -> MutexGuard<'_, T> {
        self.raw.lock();
        MutexGuard::new(self)
    }

    /// Attempts to acquire this lock. This function does not block.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.raw.try_lock() {
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

    pub(crate) unsafe fn unlock(&self) {
        // SAFETY: caller must ensure that this is safe
        unsafe {
            self.mutex.raw.unlock();
        }
    }

    pub(crate) fn relock(self) -> MutexLockFuture<'a, T> {
        let lock = self.mutex.lock();
        drop(self);
        lock
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
        unsafe { self.mutex.raw.unlock() };
    }
}
