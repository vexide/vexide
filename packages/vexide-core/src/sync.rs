//! Synchronization types for FreeRTOS tasks.
//!
//! Types implemented here are specifically designed to mimic the standard library.

use core::{
    cell::UnsafeCell,
    error::Error,
    fmt::Debug,
    future::Future,
    mem::MaybeUninit,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering},
};

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
/// This is different from a [`Mutex`] because it allows for multiple readers at the same time.
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

const ONCE_COMPLETE: bool = true;
const ONCE_INCOMPLETE: bool = false;

/// A synchronization primitive which can be used to run a one-time global
/// initialization. Useful for one-time initialization for FFI or related
/// functionality. This type can only be constructed with [`Once::new()`].
///
/// # Examples
///
/// ```
/// use vexide::core::sync::Once;
///
/// static START: Once = Once::new();
///
/// START.call_once(|| {
///     // run initialization here
/// });
/// ```
pub struct Once {
    state: Mutex<bool>,
}
impl Once {
    /// Create a new uncompleted Once
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(false),
        }
    }

    /// Returns true if call_once has been run.
    pub fn is_complete(&self) -> bool {
        if let Some(state) = self.state.try_lock() {
            *state == ONCE_COMPLETE
        } else {
            false
        }
    }

    /// Runs a closure if and only if this is the first time that call_once has been run.
    /// This is useful for making sure that expensive initialization is only run once.
    /// This will block if another task is running a different initialization routine.
    pub async fn call_once<F: FnOnce()>(&self, fun: F) {
        let mut state = self.state.lock().await;
        if *state == ONCE_INCOMPLETE {
            fun();
            *state = true;
        }
    }
}

/// A synchronization primitive which can be used to run initialization code once.
/// This type is thread safe and can be used in statics.
/// All functions that can block are async.
pub struct OnceLock<T> {
    inner: Once,
    data: UnsafeCell<MaybeUninit<T>>,
}
unsafe impl<T: Send> Send for OnceLock<T> {}
unsafe impl<T: Send + Sync> Sync for OnceLock<T> {}
impl<T> OnceLock<T> {
    /// Creates a new uninitialized [`OnceLock`].
    pub const fn new() -> Self {
        Self {
            inner: Once::new(),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Get a reference to the data in the [`OnceLock`] if it has been initialized.
    pub fn get(&self) -> Option<&T> {
        if self.inner.is_complete() {
            Some(unsafe { &*(*self.data.get()).as_ptr() })
        } else {
            None
        }
    }

    /// Get a mutable reference to the data in the [`OnceLock`] if it has been initialized.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.inner.is_complete() {
            Some(unsafe { &mut *(*self.data.get()).as_mut_ptr() })
        } else {
            None
        }
    }

    /// Attempt to set the data in the [`OnceLock`] if it has not been initialized.
    /// If already initialized, the data is returned in an [`Err`](Result::Err)` variant.
    pub async fn set(&self, data: T) -> Result<(), T> {
        if self.inner.is_complete() {
            return Err(data);
        }

        Ok(())
    }

    /// Consumes the [`OnceLock`] and returns the inner data if it has been initialized.
    pub fn into_inner(self) -> Option<T> {
        if self.inner.is_complete() {
            Some(unsafe { (*self.data.get()).as_ptr().read() })
        } else {
            None
        }
    }

    /// Move the data out of the [`OnceLock`] if it has been initialized.
    /// This will leave the [`OnceLock`] in an uninitialized state.
    pub fn take(&mut self) -> Option<T> {
        let data = if self.inner.is_complete() {
            Some(unsafe { (*self.data.get()).as_ptr().read() })
        } else {
            None
        };
        self.inner = Once::new();
        *self.data.get_mut() = MaybeUninit::uninit();
        debug_assert!(!self.inner.is_complete());
        data
    }

    /// Attempt to set the data in the [`OnceLock`] if it has not been initialized.
    /// This is similar to [`OnceLock::set`] but always returns the data in the [`OnceLock`].
    pub async fn try_insert(&self, data: T) -> Result<&T, (&T, T)> {
        match self.set(data).await {
            Ok(()) => Ok(self.get().unwrap()),
            Err(data) => Err((self.get().unwrap(), data)),
        }
    }

    /// Get or initialize the data in the [`OnceLock`].
    /// This function will always return the value stored.
    pub async fn get_or_init(&self, init: impl FnOnce() -> T) -> &T {
        if let Some(data) = self.get() {
            return data;
        }
        self.init(init).await;
        unsafe { &*(*self.data.get()).as_ptr() }
    }

    /// Get or try to initialize the data in the [`OnceLock`].
    /// If the initialization function is run and returns an error, the error is returned and no value is set.
    pub async fn get_or_try_init<E: Error>(
        &self,
        init: impl FnOnce() -> Result<T, E>,
    ) -> Result<&T, E> {
        if let Some(data) = self.get() {
            return Ok(data);
        }
        self.try_init(init).await?;
        debug_assert!(self.inner.is_complete());
        Ok(unsafe { &*(*self.data.get()).as_ptr() })
    }

    async fn init(&self, init: impl FnOnce() -> T) {
        self.inner
            .call_once(|| unsafe {
                (*self.data.get()).write(init());
            })
            .await;
        debug_assert!(self.inner.is_complete());
    }

    async fn try_init<E: Error>(&self, init: impl FnOnce() -> Result<T, E>) -> Result<(), E> {
        match init() {
            Ok(data) => {
                self.inner
                    .call_once(|| unsafe {
                        (*self.data.get()).write(data);
                    })
                    .await;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
