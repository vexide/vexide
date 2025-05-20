use core::{
    alloc::Layout,
    arch::asm,
    cell::{Cell, RefCell},
    ptr,
};

unsafe extern "C" {
    static mut __tdata_start: u8;
    static mut __tdata_end: u8;
}

fn get_tls_ptr() -> *const () {
    let ptr: *const ();
    // Safety: This is atomic and the environment is single-threaded
    unsafe {
        asm!("mrc p15 0, {}, c13, c0, 2", out(reg) ptr);
    }
    ptr
}

unsafe fn set_tls_ptr(ptr: *const ()) {
    unsafe {
        asm!("mrc p15, 0, {}, c13, c0, 2", in(reg) ptr);
    }
}

fn tls_layout() -> Layout {
    Layout::from_size_align(
        unsafe { (&raw const __tdata_end).byte_offset_from_unsigned(&raw const __tdata_start) },
        16,
    )
    .unwrap()
}

pub(crate) struct TlsReg(());

impl TlsReg {
    pub const unsafe fn new() -> Self {
        Self(())
    }

    #[expect(clippy::unused_self)]
    pub unsafe fn set(&mut self, tls: &Tls) {
        unsafe {
            set_tls_ptr(tls.mem);
        }
    }
}

pub struct Tls {
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
}

impl Drop for Tls {
    fn drop(&mut self) {
        unsafe {
            alloc::alloc::dealloc(self.mem.cast_mut().cast(), tls_layout());
        }
    }
}

pub struct Key<T: 'static> {
    inner_static: &'static T,
}

unsafe impl<T> Sync for Key<T> {}
unsafe impl<T> Send for Key<T> {}

// allows matching `const` expressions
#[expect(edition_2024_expr_fragment_specifier)]
#[macro_export]
macro_rules! thread_local {
    {
        $(#[$attr:meta])*
        $vis:vis static $name:ident: $type:ty = $init:expr;
    } => {
        $(#[$attr])*
        $vis static $name: Key<$type> = {
            #[repr(transparent)]
            struct Opaque<T>(T);

            unsafe impl<T> Sync for Opaque<T> {}

            #[unsafe(link_section = ".tdata")]
            static INNER: Opaque<$type> = Opaque($init);

            unsafe {
                Key::new(&INNER.0)
            }
        };
    };

    {
        $(#[$attr:meta])*
        $vis:vis static $name:ident: $type:ty = $init:expr_2021;
        $($rest:tt)*
    } => {
        $crate::thread_local!($vis static $name: $type = $init;);
        $crate::thread_local!($($rest)*);
    }
}

impl<T: 'static> Key<T> {
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

    pub fn with<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let ptr = unsafe { get_tls_ptr().cast::<T>().byte_add(self.offset()) };

        f(unsafe { &*ptr })
    }
}

impl<T: 'static> Key<Cell<T>> {
    pub fn get(&'static self) -> T
    where
        T: Copy,
    {
        self.with(Cell::get)
    }

    pub fn set(&'static self, value: T) {
        self.with(|cell| cell.set(value));
    }

    pub fn take(&'static self) -> T
    where
        T: Default,
    {
        self.with(Cell::take)
    }

    pub fn replace(&'static self, value: T) -> T {
        self.with(|cell| cell.replace(value))
    }
}

impl<T: 'static> Key<RefCell<T>> {
    pub fn with_borrow<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        self.with(|cell| f(&cell.borrow()))
    }

    pub fn with_borrow_mut<F, R>(&'static self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        self.with(|cell| f(&mut cell.borrow_mut()))
    }

    pub fn set(&'static self, value: T) {
        self.with_borrow_mut(|refmut| *refmut = value);
    }

    pub fn take(&'static self) -> T
    where
        T: Default,
    {
        self.with(RefCell::take)
    }

    pub fn replace(&'static self, value: T) -> T {
        self.with(|cell| cell.replace(value))
    }
}
