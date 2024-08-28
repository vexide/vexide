use core::{
    cell::UnsafeCell,
    fmt::Debug,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
};

use futures_core::Future;

struct RwLockState {
    lock_status: AtomicU8,
    reader_count: AtomicUsize,
}
impl RwLockState {
    const UNLOCKED: u8 = 0;
    const LOCKED_EXCLUSIVE: u8 = 1;
    const LOCKED_SHARED: u8 = 2;

    const fn new() -> Self {
        Self {
            lock_status: AtomicU8::new(Self::UNLOCKED),
            reader_count: AtomicUsize::new(0),
        }
    }

    fn try_lock_exclusive(&self) -> bool {
        self.lock_status
            .compare_exchange(
                Self::UNLOCKED,
                Self::LOCKED_EXCLUSIVE,
                Ordering::Acquire,
                Ordering::Acquire,
            )
            .is_ok()
    }

    fn try_lock_shared(&self) -> bool {
        let state = self.lock_status.load(Ordering::Acquire);
        if state == Self::LOCKED_EXCLUSIVE {
            return false;
        }
        self.lock_status
            .store(Self::LOCKED_SHARED, Ordering::Release);
        self.reader_count.fetch_add(1, Ordering::Acquire);
        true
    }

    fn try_unlock(&self) -> bool {
        let reader_count = self.reader_count.load(Ordering::Acquire);
        if reader_count > 1 {
            self.reader_count.store(reader_count - 1, Ordering::Release);
            false
        } else {
            self.lock_status.store(Self::UNLOCKED, Ordering::Release);
            self.reader_count.store(0, Ordering::Release);
            true
        }
    }
}

/// Allows for gaining immutable access to the data in an [`RwLock`]`.
/// Multiple readers can access the data at the same time.
pub struct RwLockReadGuard<'a, T> {
    lock: &'a RwLock<T>,
}
impl<'a, T> core::ops::Deref for RwLockReadGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}
impl<T> Drop for RwLockReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.try_unlock();
    }
}

/// A future that resolves to a read guard.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct RwLockReadFuture<'a, T> {
    lock: &'a RwLock<T>,
}
impl<'a, T> Future for RwLockReadFuture<'a, T> {
    type Output = RwLockReadGuard<'a, T>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if critical_section::with(|_| self.lock.state.try_lock_shared()) {
            core::task::Poll::Ready(RwLockReadGuard { lock: self.lock })
        } else {
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    }
}

/// Allows for gaining mutable access to the data in an [`RwLock`]`.
/// Only one writer can access the data at a time.
pub struct RwLockWriteGuard<'a, T> {
    lock: &'a RwLock<T>,
}
impl<'a, T> core::ops::Deref for RwLockWriteGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}
impl<'a, T> core::ops::DerefMut for RwLockWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}
impl<T> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.try_unlock();
    }
}

/// A future that resolves to a write guard.
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct RwLockWriteFuture<'a, T> {
    lock: &'a RwLock<T>,
}
impl<'a, T> Future for RwLockWriteFuture<'a, T> {
    type Output = RwLockWriteGuard<'a, T>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        if critical_section::with(|_| self.lock.state.try_lock_exclusive()) {
            core::task::Poll::Ready(RwLockWriteGuard { lock: self.lock })
        } else {
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    }
}

/// A reader-writer lock synchronization primitive.
/// This type allows multiple readers or one writer at a time.
///
/// This is different from a [`Mutex`](super::Mutex) because it allows for multiple readers at the same time.
pub struct RwLock<T> {
    state: RwLockState,
    data: UnsafeCell<T>,
}
impl<T> RwLock<T> {
    /// Creates a new reader-writer lock.
    pub const fn new(data: T) -> Self {
        Self {
            state: RwLockState::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Obtains a read lock on the data.
    /// Multiple read locks can be held at the same time.
    pub const fn read(&self) -> RwLockReadFuture<'_, T> {
        RwLockReadFuture { lock: self }
    }

    /// Obtains a write lock on the data.
    /// Only one write lock can be held at a time.
    pub const fn write(&self) -> RwLockWriteFuture<'_, T> {
        RwLockWriteFuture { lock: self }
    }

    /// Attempt to gain a read lock on the data.
    pub fn try_read(&self) -> Option<RwLockReadGuard<'_, T>> {
        if critical_section::with(|_| self.state.try_lock_shared()) {
            Some(RwLockReadGuard { lock: self })
        } else {
            None
        }
    }

    /// Attempt to gain a write lock on the data.
    pub fn try_write(&self) -> Option<RwLockWriteGuard<'_, T>> {
        if critical_section::with(|_| self.state.try_lock_exclusive()) {
            Some(RwLockWriteGuard { lock: self })
        } else {
            None
        }
    }

    /// Get mutable access to the data stored in the read-write lock.
    /// This doesn't require a lock to be acquired because the borrow checker guarentees exclusive access because self is a mutable reference.
    pub fn get_mut(&mut self) -> &mut T {
        //SAFETY: This is safe because we have exclusive access to the data thanks to taking a mutable reference to self.
        unsafe { &mut *self.data.get() }
    }

    /// Consumes the read-write lock and returns the inner data.
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: Debug> Debug for RwLock<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug = f.debug_struct("RwLock");
        match self.try_read() {
            Some(data) => debug.field("data", &*data),
            None => debug.field("data", &format_args!("<locked>")),
        };
        debug.finish_non_exhaustive()
    }
}
impl<T: Default> Default for RwLock<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
impl<T> From<T> for RwLock<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}
