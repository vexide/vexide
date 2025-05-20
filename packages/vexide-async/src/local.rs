//! Task-local storage
//!
//! Task-local storage is a way to create global variables specific to the current task that live
//! for the entirety of the task's lifetime, almost like statics. Since they are local to the task,
//! they implement [`Send`] and [`Sync`], regardless of what the underlying data does or does not
//! implement.
//!
//! This is especially useful for interior mutability in globals. Normally, you would not be able
//! to store a [`RefCell`] inside of a static variable since it isn't safe in multitasked contexts.
//! Task-locals can eliminate this problem by making sure the variable never leaves the current
//! task.
//!
//! Task-locals can be declared using the [`task_local`] macro, which creates a [`LocalKey`] with
//! the same name that can be used to access the local.

use core::{
    alloc::Layout,
    cell::{Cell, RefCell},
    ptr,
    sync::atomic::{AtomicUsize, Ordering},
};

unsafe extern "C" {
    static mut __tdata_start: u8;
    static mut __tdata_end: u8;
}

static TLS_PTR: AtomicUsize = AtomicUsize::new(0);

fn tls_layout() -> Layout {
    const MAX_ALIGNMENT: usize = 16;

    Layout::from_size_align(
        unsafe { (&raw const __tdata_end).byte_offset_from_unsigned(&raw const __tdata_start) },
        MAX_ALIGNMENT,
    )
    .unwrap()
}

pub(crate) struct Tls {
    mem: *const (),
}

impl Tls {
    #[must_use]
    pub fn new_alloc() -> Self {
        let tls_layout = tls_layout();
        let mem = unsafe { alloc::alloc::alloc(tls_layout) };

        unsafe {
            ptr::copy_nonoverlapping(&raw const __tdata_start, mem, tls_layout.size());
        }

        Self {
            mem: mem as *const (),
        }
    }

    pub unsafe fn set_current_tls(&self) {
        TLS_PTR.store(self.mem as usize, Ordering::Relaxed);
    }
}

impl Drop for Tls {
    fn drop(&mut self) {
        unsafe {
            alloc::alloc::dealloc(self.mem.cast_mut().cast(), tls_layout());
        }
    }
}

/// A variable stored in task-local storage.
///
/// # Usage
///
/// The primary mode of accessing this is through the [`LocalKey::with`] method. For
/// [`LocalKey<RefCell<T>>`] and [`LocalKey<Cell<T>>`], additional convenience methods are added
/// that mirror the underlying [`RefCell<T>`] or [`Cell<T>`]'s methods.
///
/// # Examples
///
/// ```
/// task_local! {
///     static PHI: f64 = 1.61803;
///     static COUNTER: Cell<u32> = Cell::new(0);
///     static NAMES: RefCell<Vec<String>> = RefCell::new(Vec::new());
/// };
///
/// // LocalKey::with accepts a function and applies it to a reference, returning whatever value
/// // the function returned
/// let double_phi = PHI.with(|&phi| phi * 2.0);
/// assert_eq!(double_phi, 1.61803 * 2.0);
///
/// // We can use interior mutability
/// COUNTER.set(1);
/// assert_eq!(COUNTER.get(), 1);
///
/// NAMES.with_borrow_mut(|names| names.push(String::from("Johnny")));
/// NAMES.with_borrow(|names| assert_eq!(names.len(), 1));
///
/// use vexide::async_runtime::spawn;
///
/// // Creating another task
/// spawn(async {
///     // The locals of the previous task are completely different.
///     assert_eq!(COUNTER.get(), 0);
///     NAME.with_borrow(|names| assert_eq!(names.len(), 0));
/// }).await;
/// ```
pub struct LocalKey<T: 'static> {
    inner_static: &'static T,
}

unsafe impl<T> Sync for LocalKey<T> {}
unsafe impl<T> Send for LocalKey<T> {}

/// Declares task-local variables in [`LocalKey`]s of the same names.
///
/// # Syntax
///
/// ```
/// task_local! {
///     static PHI: f64 = 1.61803;
///     static COUNTER: Cell<u32> = Cell::new(0);
///     static NAMES: RefCell<Vec<String>> = RefCell::new(Vec::new());
/// };
/// ```
// allows matching `const` expressions
#[expect(edition_2024_expr_fragment_specifier)]
#[macro_export]
macro_rules! task_local {
    {
        $(#[$attr:meta])*
        $vis:vis static $name:ident: $type:ty = $init:expr;
    } => {
        $(#[$attr])*
        $vis static $name: LocalKey<$type> = {
            #[repr(transparent)]
            struct Opaque<T>(T);

            unsafe impl<T> Sync for Opaque<T> {}

            #[unsafe(link_section = ".tdata")]
            static INNER: Opaque<$type> = Opaque($init);

            unsafe {
                LocalKey::new(&INNER.0)
            }
        };
    };

    {
        $(#[$attr:meta])*
        $vis:vis static $name:ident: $type:ty = $init:expr;
        $($rest:tt)*
    } => {
        $crate::thread_local!($vis static $name: $type = $init;);
        $crate::thread_local!($($rest)*);
    }
}

impl<T: 'static> LocalKey<T> {
    /// # Safety
    ///
    /// `inner_static` must point to valid memory in the .tdata section.
    pub const unsafe fn new(inner_static: &'static T) -> Self {
        Self { inner_static }
    }

    fn offset(&'static self) -> usize {
        unsafe {
            ptr::from_ref(self.inner_static).byte_offset_from_unsigned(&raw const __tdata_start)
        }
    }

    /// Obtains a reference to the local and applies it to the function `f`, returning whatever `f`
    /// returned.
    ///
    /// # Examples
    ///
    /// ```
    /// task_local! {
    ///     static PHI: 1.61803;
    /// }
    ///
    /// let double_phi = PHI.with(|&phi| phi * 2.0);
    /// assert_eq!(double_phi, 1.61803 * 2.0);
    /// ```
    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let ptr = (self.offset() + TLS_PTR.load(Ordering::Relaxed)) as *const T;
        f(unsafe { &*ptr })
    }
}

impl<T: 'static> LocalKey<Cell<T>> {
    /// Returns a copy of the contained value.
    pub fn get(&'static self) -> T
    where
        T: Copy,
    {
        self.with(Cell::get)
    }

    /// Sets the contained value.
    pub fn set(&'static self, value: T) {
        self.with(|cell| cell.set(value));
    }

    /// Takes the value of contained value, leaving [`Default::default()`] in its place.
    pub fn take(&'static self) -> T
    where
        T: Default,
    {
        self.with(Cell::take)
    }

    /// Replaces the contained value with `value`, returning the old contained value.
    pub fn replace(&'static self, value: T) -> T {
        self.with(|cell| cell.replace(value))
    }
}

impl<T: 'static> LocalKey<RefCell<T>> {
    /// Immutably borrows from the [`RefCell`] and applies the obtained reference to `f`.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    pub fn with_borrow<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.with(|cell| f(&cell.borrow()))
    }

    /// Mutably borrows from the [`RefCell`] and applies the obtained reference to `f`.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn with_borrow_mut<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.with(|cell| f(&mut cell.borrow_mut()))
    }

    /// Sets the contained value.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn set(&'static self, value: T) {
        self.with_borrow_mut(|refmut| *refmut = value);
    }

    /// Takes the contained value, leaving [`Default::default()`] in its place.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn take(&'static self) -> T
    where
        T: Default,
    {
        self.with(RefCell::take)
    }

    /// Replaces the contained value with `value`, returning the old contained value.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    pub fn replace(&'static self, value: T) -> T {
        self.with(|cell| cell.replace(value))
    }
}
