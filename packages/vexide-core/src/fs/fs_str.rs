use core::ptr;

/// Borrowed reference to an VEXos filesystem OS string.
///
/// This type represents a borrowed reference to a string in VEXos's preferred
/// representation for filesystem paths.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FsStr {
    inner: [i8],
}
impl FsStr {
    /// Coerces into an `OsStr` slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::core::fs::FsStr;
    ///
    /// let fs_str = FsStr::new("foo");
    /// ```
    pub fn new<'a, S: AsRef<Self> + 'a>(string: S) -> &'a Self {
        unsafe { &*ptr::from_ref::<Self>(string.as_ref()) }
    }

    /// Converts a slice of bytes to an FS string slice without checking that the string contains
    /// valid `FsStr`-encoded data.
    ///
    /// # Safety
    ///
    /// As the encoding is unspecified, callers must pass in bytes that originated as a mixture of
    /// validated UTF-8 and bytes from [`OsStr::as_encoded_bytes`] from within the same Rust version
    /// built for the same target platform.
    ///
    /// Due to the encoding being self-synchronizing, the bytes from [`OsStr::as_encoded_bytes`] can be
    /// split either immediately before or immediately after any valid non-empty UTF-8 substring.
    #[must_use]
    pub const unsafe fn from_encoded_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { &*(ptr::from_ref::<[u8]>(bytes) as *const Self) }
    }

    pub(crate) const fn as_ptr(&self) -> *const i8 {
        self.inner.as_ptr()
    }

    /// Converts an FS string slice to a byte slice. To convert the byte slice back into an FS
    /// string slice, use the [`OsStr::from_encoded_bytes_unchecked`] function.
    ///
    /// Note: As the encoding is unspecified, any sub-slice of bytes that is not valid UTF-8 should
    /// be treated as opaque and only comparable within the same Rust version built for the same
    /// target platform.  For example, sending the slice over the network or storing it in a file
    /// will likely result in incompatible byte slices.
    #[must_use]
    pub const fn as_encoded_bytes(&self) -> &[u8] {
        unsafe { &*(ptr::from_ref::<[i8]>(&self.inner) as *const [u8]) }
    }
}

impl AsRef<FsStr> for str {
    fn as_ref(&self) -> &FsStr {
        unsafe {
            FsStr::from_encoded_bytes_unchecked(self.as_bytes())
        }
    }
}
