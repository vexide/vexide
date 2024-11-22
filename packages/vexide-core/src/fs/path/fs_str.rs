use core::ptr;

use alloc::{borrow::ToOwned, boxed::Box};

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FsStr {
    inner: [i8],
}
impl FsStr {
    pub fn new<'a, S: AsRef<Self> + 'a>(string: S) -> &'a Self {
        unsafe { &*ptr::from_ref::<Self>(string.as_ref()) }
    }

    pub const fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { &*(ptr::from_ref::<[u8]>(bytes) as *const Self) }
    }

    pub const fn as_ptr(&self) -> *const i8 {
        self.inner.as_ptr()
    }

    pub const fn as_encoded_bytes(&self) -> &[u8] {
        unsafe { &*(ptr::from_ref::<[i8]>(&self.inner) as *const [u8]) }
    }
}

impl AsRef<FsStr> for str {
    fn as_ref(&self) -> &FsStr {
        FsStr::from_bytes_unchecked(self.as_bytes())
    }
}
impl AsRef<FsStr> for [u8] {
    fn as_ref(&self) -> &FsStr {
        FsStr::from_bytes_unchecked(self)
    }
}
impl AsRef<FsStr> for [i8] {
    fn as_ref(&self) -> &FsStr {
        unsafe { &*(ptr::from_ref::<[i8]>(self) as *const FsStr) }
    }
}
