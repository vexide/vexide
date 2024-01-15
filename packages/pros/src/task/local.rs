//! A custom TLS implementation that allows for more than 5 entries in TLS.
//!
//! FreeRTOS task locals have a hard limit of entries.
//! The custom implementation used here stores a pointer to a custom TLS struct inside the first slot of FreeRTOS TLS.
//! This sacrifices a bit of speed for the ability to have as many entries as memory allows.
//!
//! [`LocalKey`]s can be created with the [`os_task_local!`](crate::os_task_local!) macro.
//! ## Example
//! ```rust
//! os_task_local! {
//!     static FOO: u32 = 0;
//!     static BAR: String = String::from("Hello, world!");
//! }
//! ```

use alloc::{boxed::Box, collections::BTreeMap};
use core::{cell::RefCell, ptr::NonNull, sync::atomic::AtomicU32};

use spin::Once;

use super::current;

/// A semaphore that makes sure that each [`LocalKey`] has a unique index into TLS.
static INDEX: AtomicU32 = AtomicU32::new(0);

/// Set a value in OS TLS.
/// This requires you to leak val so that you can be sure it lives as long as the task.
/// # Safety
/// Unsafe because you can change the thread local storage while it is being read.
unsafe fn thread_local_storage_set<T>(task: pros_sys::task_t, val: &'static T, index: u32) {
    // Yes, we transmute val. This is the intended use of this function.
    pros_sys::vTaskSetThreadLocalStoragePointer(task, index as _, (val as *const T).cast());
}

/// Get a value from OS TLS.
/// # Safety
/// Unsafe because we can't check if the type is the same as the one that was set.
unsafe fn thread_local_storage_get<T>(task: pros_sys::task_t, index: u32) -> Option<&'static T> {
    let val = pros_sys::pvTaskGetThreadLocalStoragePointer(task, index as _);
    val.cast::<T>().as_ref()
}

/// Get or create the [`ThreadLocalStorage`] for the current task.
fn fetch_storage() -> &'static RefCell<ThreadLocalStorage> {
    let current = current();

    // Get the thread local storage for this task.
    // Creating it if it doesn't exist.
    // This is safe as long as index 0 of the freeRTOS TLS is never set to any other type.
    unsafe {
        thread_local_storage_get(current.task, 0).unwrap_or_else(|| {
            let storage = Box::leak(Box::new(RefCell::new(ThreadLocalStorage {
                data: BTreeMap::new(),
            })));
            thread_local_storage_set(current.task, storage, 0);
            storage
        })
    }
}

/// A custom thread local storage implementation.
/// This itself is stored inside real OS TLS, it allows for more than 5 entries in TLS.
/// [`LocalKey`]s store their data inside this struct.
struct ThreadLocalStorage {
    pub data: BTreeMap<usize, NonNull<()>>,
}

/// A TLS key that owns its data.
/// Can be created with the [`os_task_local`](crate::os_task_local!) macro.
pub struct LocalKey<T: 'static> {
    index: Once<usize>,
    init: fn() -> T,
}

impl<T: 'static> LocalKey<T> {
    /// Creates a new local key that lazily initializes its data.
    /// init is called to initialize the data when it is first accessed from a new thread.
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            index: Once::new(),
            init,
        }
    }

    /// Get the index of this key, or get the next one if it has never been created before.
    fn index(&'static self) -> &usize {
        self.index
            .call_once(|| INDEX.fetch_add(1, core::sync::atomic::Ordering::Relaxed) as _)
    }

    /// Sets or initializes the value of this key.
    /// Does not run the initializer.
    pub fn set(&'static self, val: T) {
        let storage = fetch_storage();
        let index = *self.index();

        let val = Box::leak(Box::new(val));

        storage
            .borrow_mut()
            .data
            .insert(index, NonNull::new((val as *mut T).cast()).unwrap());
    }

    /// Passes a reference to the value of this key to the given closure.
    /// If the value has not been initialized yet, it will be initialized.
    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&'static T) -> R,
    {
        let storage = fetch_storage();
        let index = *self.index();

        // Make sure that the borrow is dropped if the if does not execute.
        // This shouldn't be necessary, but caution is good.
        {
            if let Some(val) = storage.borrow_mut().data.get(&index) {
                return f(unsafe { val.cast().as_ref() });
            }
        }

        let val = Box::leak(Box::new((self.init)()));
        storage
            .borrow_mut()
            .data
            .insert(index, NonNull::new((val as *mut T).cast::<()>()).unwrap());
        f(val)
    }
}

/// Create new [`LocalKey`]\(s)
/// # Example
/// ```rust
/// os_task_local! {
///     static FOO: u32 = 0;
///     static BAR: String = String::new();
/// }
#[macro_export]
macro_rules! os_task_local {
    ($($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr;)*) => {
        $(
        $(#[$attr])*
        $vis static $name: $crate::task::local::LocalKey<$t> = $crate::task::local::LocalKey::new(|| $init);
        )*
    };
}
