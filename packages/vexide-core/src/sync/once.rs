use core::{cell::UnsafeCell, error::Error, fmt::Debug, mem::MaybeUninit};

use super::mutex::Mutex;

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
    const ONCE_INCOMPLETE: bool = false;
    const ONCE_COMPLETE: bool = true;

    /// Create a new uncompleted Once
    pub const fn new() -> Self {
        Self {
            state: Mutex::new(false),
        }
    }

    /// Returns true if call_once has been run.
    pub fn is_complete(&self) -> bool {
        if let Some(state) = self.state.try_lock() {
            *state == Self::ONCE_COMPLETE
        } else {
            false
        }
    }

    /// Runs a closure if and only if this is the first time that call_once has been run.
    /// This is useful for making sure that expensive initialization is only run once.
    /// This will block if another task is running a different initialization routine.
    pub async fn call_once<F: FnOnce()>(&self, fun: F) {
        let mut state = self.state.lock().await;
        if *state == Self::ONCE_INCOMPLETE {
            fun();
            *state = true;
        }
    }

    pub(crate) fn call_once_blocking<F: FnOnce()>(&self, fun: F) {
        let mut state = self.state.lock_blocking();
        if *state == Self::ONCE_INCOMPLETE {
            fun();
            *state = true;
        }
    }
}
impl Debug for Once {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Once").finish_non_exhaustive()
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
    pub fn set(&self, data: T) -> Result<(), T> {
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
    pub fn try_insert(&self, data: T) -> Result<&T, (&T, T)> {
        match self.set(data) {
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
impl<T: Clone> Clone for OnceLock<T> {
    fn clone(&self) -> Self {
        let new = Self::new();
        if let Some(data) = self.get() {
            unsafe { new.set(data.clone()).unwrap_unchecked() };
        }
        new
    }
}
impl<T: Debug> Debug for OnceLock<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OnceLock")
            .field("data", &self.get())
            .finish_non_exhaustive()
    }
}
impl<T> Default for OnceLock<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Drop for OnceLock<T> {
    fn drop(&mut self) {
        if self.inner.is_complete() {
            unsafe { (*self.data.get()).assume_init_drop() }
        }
    }
}
impl<T> From<T> for OnceLock<T> {
    fn from(data: T) -> Self {
        let lock = Self::new();
        unsafe { lock.set(data).unwrap_unchecked() };
        lock
    }
}

impl<T: PartialEq> PartialEq for OnceLock<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}
impl<T: Eq> Eq for OnceLock<T> {}
