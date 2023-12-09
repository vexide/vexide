use core::{cell::RefCell, ptr::NonNull, sync::atomic::AtomicU32};

use alloc::{boxed::Box, collections::BTreeMap};

use spin::Once;

use super::current;

static mut INDEX: AtomicU32 = AtomicU32::new(0);

// Unsafe because you can change the thread local storage while it is being read.
// This requires you to leak val so that you can be sure it lives the entire task.
unsafe fn task_local_storage_set<T>(task: pros_sys::task_t, val: &'static T, index: u32) {
    // Yes, we transmute val. This is the intended use of this function.
    pros_sys::vTaskSetThreadLocalStoragePointer(task, index as _, (val as *const T).cast());
}

// Unsafe because we can't check if the type is the same as the one that was set.
unsafe fn task_local_storage_get<T>(task: pros_sys::task_t, index: u32) -> Option<&'static T> {
    let val = pros_sys::pvTaskGetThreadLocalStoragePointer(task, index as _);
    val.cast::<T>().as_ref()
}

struct ThreadLocalStorage {
    pub data: BTreeMap<usize, NonNull<()>>,
}

pub struct LocalKey<T: 'static> {
    index: Once<usize>,
    init: fn() -> T,
}

impl<T: 'static> LocalKey<T> {
    pub const fn new(init: fn() -> T) -> Self {
        Self {
            index: Once::new(),
            init,
        }
    }

    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&'static T) -> R,
    {
        let index = *self.index.call_once(|| unsafe {
            INDEX.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as usize
        });
        let current = current();

        // Get the thread local storage for this task.
        // Creating it if it doesn't exist.
        let storage = unsafe {
            task_local_storage_get(current.task, 0).unwrap_or_else(|| {
                let storage = Box::leak(Box::new(RefCell::new(ThreadLocalStorage {
                    data: BTreeMap::new(),
                })));
                task_local_storage_set(current.task, storage, 0);
                storage
            })
        };

        {
            if let Some(val) = storage.borrow_mut().data.get(&index) {
                return f(unsafe { val.cast().as_ref() });
            }
        }

        let val = Box::leak(Box::new((self.init)()));
        storage.borrow_mut().data.insert(index, NonNull::new((val as *mut T).cast::<()>()).unwrap());
        f(val)
    }
}

#[macro_export]
macro_rules! os_task_local {
    ($($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty = $init:expr;)*) => {
        $(
        $(#[$attr])*
        $vis static $name: $crate::task::local::LocalKey<$t> = $crate::task::local::LocalKey::new(|| $init);
        )*
    };
}
