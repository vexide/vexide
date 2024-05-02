use core::{cell::UnsafeCell, fmt::Debug, mem::MaybeUninit, ops::Deref};

use super::Once;

struct Data<T, I> {
    data: MaybeUninit<T>,
    init: MaybeUninit<I>,
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
                data: MaybeUninit::uninit(),
                init: MaybeUninit::new(init),
            }),
            once: Once::new(),
        }
    }

    /// Consume the [`LazyLock`] and return the inner value if it has been initialized.
    pub fn into_inner(self) -> Result<T, I> {
        let data = self.data.into_inner();
        match self.once.is_complete() {
            true => Ok(unsafe { data.data.assume_init() }),
            false => Err(unsafe { data.init.assume_init() }),
        }
    }

    /// # Safety
    /// Caller must ensure this function is only called once.
    unsafe fn lazy_init(&self) {
        let data = unsafe { &mut *self.data.get() };
        let init = unsafe { data.init.assume_init_read() };
        data.data = MaybeUninit::new((init)());
        unsafe {
            data.init.assume_init_drop();
        }
    }

    /// The equivalent of the standard libraries [`LazyLock::force`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html#method.force).
    /// It has been renamed to get because it is the only way to asynchronously get the value.
    pub async fn get(&self) -> &T {
        self.once.call_once(|| unsafe { self.lazy_init() }).await;
        unsafe { &(*self.data.get()).data.assume_init_ref() }
    }

    fn force(&self) -> &T {
        self.once.call_once_blocking(|| unsafe { self.lazy_init() });
        unsafe { (*self.data.get()).data.assume_init_ref() }
    }
}
impl<T, I: Default> Default for LazyLock<T, I> {
    fn default() -> Self {
        Self {
            data: UnsafeCell::new(Data {
                data: MaybeUninit::uninit(),
                init: MaybeUninit::new(I::default()),
            }),
            once: Once::new(),
        }
    }
}
impl<T, I: FnOnce() -> T> Deref for LazyLock<T, I> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.force()
    }
}
impl<T: Debug, I> Debug for LazyLock<T, I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut struct_ = f.debug_struct("LazyLock");
        if self.once.is_complete() {
            struct_.field("data", unsafe { (*self.data.get()).data.assume_init_ref() });
        } else {
            struct_.field("data", &"Uninitialized");
        }
        struct_.finish()
    }
}
