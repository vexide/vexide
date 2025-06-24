use core::{
    cell::UnsafeCell,
    fmt::Debug,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use super::Once;

union Data<T, I> {
    data: ManuallyDrop<T>,
    init: ManuallyDrop<I>,
}

/// A thread-safe value which is initialized on first access.
///
/// This type is a thread-safe [`LazyCell`](core::cell::LazyCell), and can be used in statics.
///
/// # Differences from `std::sync::LazyLock`
///
/// There are two possible edge cases that can cause different behavior in this type
/// compared to its `std` counterpart:
///
/// - If the type is lazily initialized from within its own initialization function, a panic
///   will occur rather than an infinite deadlock.
/// - By extension, if the initialization function uses `block_on` to execute `async` code
///   and another task attempts to access the underlying value of this type (through
///   either dereferencing or using [`LazyLock::force`]) before the initialization function has
///   returned a value, this function will panic rather than block. As such, you should generally
///   avoid using `block_on` when creating a value inside of a `LazyLock`.
///
///
///
/// These two differences allow us to implement `LazyLock` without an actual lock at all, since
/// we guarantee exclusive access after initialization due to the V5 being single-threaded.
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
        let mut data = unsafe { core::ptr::read(&raw const self.data).into_inner() };
        match self.once.is_completed() {
            true => Ok(unsafe { ManuallyDrop::take(&mut data.data) }),
            false => Err(unsafe { ManuallyDrop::take(&mut data.init) }),
        }
    }

    /// # Safety
    ///
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

    /// Forces the evaluation of this lazy value and returns a reference to result.
    /// This is equivalent to the `Deref` impl, but is explicit.
    ///
    /// # Panics
    ///
    /// This method will panic under two possible edge-cases:
    ///
    /// - It is called recursively from within its own initialization function.
    /// - The initialization function uses `block_on` to execute `async` code,
    ///   and another task attempts to access the underlying value of this type
    ///   in the middle of it lazily initializing.
    ///
    /// This behavior differs from the standard library, which would normally either
    /// block the current thread or deadlock forever. Since the V5 brain is a
    /// single-core system, it was determined that panicking is a more acceptable
    /// compromise than an unrecoverable deadlock.
    pub fn force(&self) -> &T {
        self.once
            .try_call_once(|| unsafe { self.lazy_init() })
            .unwrap();
        unsafe { &(*self.data.get()).data }
    }

    /// Forces the evaluation of this lazy value and returns a mutable reference to
    /// the result. This is equivalent to the `DerefMut` impl, but is explicit.
    ///
    /// # Panics
    ///
    /// This method will panic under two possible edge-cases:
    ///
    /// - It is called recursively from within its own initialization function.
    /// - The initialization function uses `block_on` to execute `async` code,
    ///   and another task attempts to access the underlying value of this type
    ///   in the middle of it lazily initializing.
    ///
    /// This behavior differs from the standard library, which would normally either
    /// block the current thread or deadlock forever. Since the V5 brain is a
    /// single-core system, it was determined that panicking is a more acceptable
    /// compromise than an unrecoverable deadlock.
    pub fn force_mut(&mut self) -> &mut T {
        self.once
            .try_call_once(|| unsafe { self.lazy_init() })
            .unwrap();
        unsafe { &mut (*self.data.get()).data }
    }
}

impl<T, I: FnOnce() -> T> Deref for LazyLock<T, I> {
    type Target = T;

    /// Dereferences the value.
    ///
    /// # Panics
    ///
    /// This method will panic under two possible edge-cases:
    ///
    /// - It is called recursively from within its own initialization function.
    /// - The initialization function uses `block_on` to execute `async` code,
    ///   and another task attempts to access the underlying value of this type
    ///   in the middle of it lazily initializing.
    ///
    /// This behavior differs from the standard library, which would normally either
    /// block the current thread or deadlock forever. Since the V5 brain is a
    /// single-core system, it was determined that panicking is a more acceptable
    /// compromise than an unrecoverable deadlock.
    fn deref(&self) -> &Self::Target {
        self.force()
    }
}

impl<T, I: FnOnce() -> T> DerefMut for LazyLock<T, I> {
    /// Mutably dereferences the value.
    ///
    /// # Panics
    ///
    /// This method will panic under two possible edge-cases:
    ///
    /// - It is called recursively from within its own initialization function.
    /// - The initialization function uses `block_on` to execute `async` code,
    ///   and another task attempts to access the underlying value of this type
    ///   in the middle of it lazily initializing.
    ///
    /// This behavior differs from the standard library, which would normally either
    /// block the current thread or deadlock forever. Since the V5 brain is a
    /// single-core system, it was determined that panicking is a more acceptable
    /// compromise than an unrecoverable deadlock.
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.force_mut()
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
