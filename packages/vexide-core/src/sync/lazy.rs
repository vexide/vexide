use core::{cell::UnsafeCell, fmt::Debug, mem::ManuallyDrop, ops::Deref};

use super::Once;

union Data<T, I> {
    data: ManuallyDrop<T>,
    init: ManuallyDrop<I>,
}
/// A thread safe lazy initialized value.
pub struct LazyLock<T, I = fn() -> T> {
    data: UnsafeCell<Data<T, I>>,
    once: Once,
}
unsafe impl<T: Send + Sync, I: Send> Sync for LazyLock<T, I> {}
impl<T, I: FnOnce() -> T> LazyLock<T, I> {
    /// Creates a new [`LazyLock`] with the given initializer.
    pub const fn new(init: I) -> Self {
        Self {
            data: UnsafeCell::new(Data {
                init: ManuallyDrop::new(init),
            }),
            once: Once::new(),
        }
    }

    /// Consume the [`LazyLock`] and return the inner value if it has been initialized.
    ///
    /// # Errors
    ///
    /// If the inner value has not been initialized, this function returns an error
    /// containing the initializer function.
    pub fn into_inner(self) -> Result<T, I> {
        let mut data = unsafe { core::ptr::read(&self.data).into_inner() };
        match self.once.is_completed() {
            true => Ok(unsafe { ManuallyDrop::take(&mut data.data) }),
            false => Err(unsafe { ManuallyDrop::take(&mut data.init) }),
        }
    }

    /// # Safety
    /// Caller must ensure this function is only called once.
    unsafe fn lazy_init(&self) {
        let initializer = unsafe { ManuallyDrop::take(&mut (*self.data.get()).init) };
        let initialized_data = initializer();
        unsafe {
            self.data.get().write(Data {
                data: ManuallyDrop::new(initialized_data),
            });
        }
    }

    /// The equivalent of the standard libraries [`LazyLock::force`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html#method.force).
    /// It has been renamed to get because it is the only way to asynchronously get the value.
    pub async fn get(&self) -> &T {
        self.once.call_once(|| unsafe { self.lazy_init() }).await;
        unsafe { &(*self.data.get()).data }
    }
}
impl<T: Default> Default for LazyLock<T> {
    fn default() -> Self {
        Self {
            data: UnsafeCell::new(Data {
                init: ManuallyDrop::new(T::default),
            }),
            once: Once::new(),
        }
    }
}
impl<T: Debug, I> Debug for LazyLock<T, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut struct_ = f.debug_struct("LazyLock");
        if self.once.is_completed() {
            struct_.field("data", unsafe { &(*self.data.get()).data });
        } else {
            struct_.field("data", &"Uninitialized");
        }
        struct_.finish_non_exhaustive()
    }
}
impl<T, I> Drop for LazyLock<T, I> {
    fn drop(&mut self) {
        match self.once.is_completed() {
            true => unsafe {
                ManuallyDrop::drop(&mut (*self.data.get()).data);
            },
            false => unsafe {
                ManuallyDrop::drop(&mut (*self.data.get()).init);
            },
        }
    }
}
