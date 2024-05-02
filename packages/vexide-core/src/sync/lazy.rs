use core::{cell::UnsafeCell, mem::MaybeUninit, ops::Deref};

use super::Once;

struct Data<T, I> {
    data: MaybeUninit<T>,
    init: MaybeUninit<I>,
}
pub struct LazyLock<T, I = fn() -> T> {
    data: UnsafeCell<Data<T, I>>,
    once: Once,
}
unsafe impl<T: Send + Sync, I: Send> Sync for LazyLock<T, I> {}
impl<T, I: FnOnce() -> T> LazyLock<T, I> {
    pub const fn new(init: I) -> Self {
        Self {
            data: UnsafeCell::new(Data {
                data: MaybeUninit::uninit(),
                init: MaybeUninit::new(init),
            }),
            once: Once::new(),
        }
    }

    fn lazy_init(&self) {
        let data = unsafe { &mut *self.data.get() };
        let init = unsafe { data.init.assume_init_read() };
        data.data = MaybeUninit::new((init)());
        unsafe {
            data.init.assume_init_drop();
        }
    }

    pub fn into_inner(self) -> Result<T, I> {
        let data = self.data.into_inner();
        match self.once.is_complete() {
            true => Ok(unsafe { data.data.assume_init() }),
            false => Err(unsafe { data.init.assume_init() }),
        }
    }

    /// The equivalent of the standard libraries [`LazyLock::force`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html#method.force).
    /// It has been renamed to get because it is the only way to asynchronously get the value.
    pub async fn get(&self) -> &T {
        self.once.call_once(|| self.lazy_init()).await;
        unsafe { &(*self.data.get()).data.assume_init_ref() }
    }

    fn get_blocking(&self) -> &T {
        self.once.call_once_blocking(|| self.lazy_init());
        unsafe { &(*self.data.get()).data.assume_init_ref() }
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
        self.get_blocking()
    }
}
