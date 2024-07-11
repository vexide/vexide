#[repr(transparent)]
pub struct FsStr {
    inner: [i8],
}
impl FsStr {
    pub fn new<'a, S: AsRef<Self> + 'a>(string: S) -> &'a Self {
        unsafe { &*(string.as_ref() as *const Self) }
    }

    pub const fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { &*(bytes as *const [u8] as *const Self) }
    }

    pub const fn as_ptr(&self) -> *const i8 {
        self.inner.as_ptr()
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
        unsafe { &*(self as *const [i8] as *const FsStr) }
    }
}
