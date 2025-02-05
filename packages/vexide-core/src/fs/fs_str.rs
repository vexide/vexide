use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};
use core::{fmt::Debug, mem::ManuallyDrop, ptr};

pub struct Display<'a> {
    fs_str: &'a FsStr,
}
impl Debug for Display<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.fs_str, f)
    }
}
impl alloc::fmt::Display for Display<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.fs_str, f)
    }
}

/// Borrowed reference to an VEXos filesystem OS string.
///
/// This type represents a borrowed reference to a string in VEXos's preferred
/// representation for filesystem paths.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FsStr {
    inner: [i8],
}
impl FsStr {
    pub(crate) const unsafe fn from_inner(inner: &[i8]) -> &Self {
        unsafe { &*(ptr::from_ref(inner) as *const Self) }
    }
    pub(crate) const unsafe fn from_inner_mut(inner: &mut [i8]) -> &mut Self {
        unsafe { &mut *(ptr::from_mut(inner) as *mut Self) }
    }

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
    /// validated UTF-8 and bytes from [`FsStr::as_encoded_bytes`] from within the same Rust version
    /// built for the same target platform.
    ///
    /// Due to the encoding being self-synchronizing, the bytes from [`FsStr::as_encoded_bytes`] can be
    /// split either immediately before or immediately after any valid non-empty UTF-8 substring.
    #[must_use]
    pub const unsafe fn from_encoded_bytes_unchecked(bytes: &[u8]) -> &Self {
        unsafe { &*(ptr::from_ref::<[u8]>(bytes) as *const Self) }
    }

    pub(crate) const fn as_ptr(&self) -> *const i8 {
        self.inner.as_ptr()
    }

    /// Converts an FS string slice to a byte slice. To convert the byte slice back into an FS
    /// string slice, use the [`FsStr::from_encoded_bytes_unchecked`] function.
    ///
    /// Note: As the encoding is unspecified, any sub-slice of bytes that is not valid UTF-8 should
    /// be treated as opaque and only comparable within the same Rust version built for the same
    /// target platform.  For example, sending the slice over the network or storing it in a file
    /// will likely result in incompatible byte slices.
    #[must_use]
    pub const fn as_encoded_bytes(&self) -> &[u8] {
        unsafe { &*(ptr::from_ref::<[i8]>(&self.inner) as *const [u8]) }
    }

    pub fn to_fs_string(&self) -> FsString {
        FsString {
            inner: self.inner.to_vec(),
        }
    }

    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_encoded_bytes())
    }
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    pub const fn display(&self) -> Display<'_> {
        Display { fs_str: self }
    }
}
impl Debug for FsStr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.as_encoded_bytes().utf8_chunks(), f)
    }
}
impl AsRef<FsStr> for FsStr {
    fn as_ref(&self) -> &FsStr {
        self
    }
}

impl AsRef<FsStr> for str {
    fn as_ref(&self) -> &FsStr {
        unsafe { FsStr::from_encoded_bytes_unchecked(self.as_bytes()) }
    }
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FsString {
    inner: Vec<i8>,
}
impl FsString {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub unsafe fn from_encoded_bytes_unchecked(bytes: Vec<u8>) -> Self {
        let mut bytes = ManuallyDrop::new(bytes);
        unsafe {
            let parts = (bytes.as_mut_ptr(), bytes.len(), bytes.capacity());
            let cast_bytes = Vec::from_raw_parts(parts.0.cast::<i8>(), parts.1, parts.2);
            Self { inner: cast_bytes }
        }
    }

    pub fn as_fs_str(&self) -> &FsStr {
        self
    }

    pub fn into_encoded_bytes(mut self) -> Vec<u8> {
        unsafe {
            let parts = (
                self.inner.as_mut_ptr(),
                self.inner.len(),
                self.inner.capacity(),
            );

            Vec::from_raw_parts(parts.0.cast::<u8>(), parts.1, parts.2)
        }
    }

    pub fn into_string(mut self) -> Result<String, FsString> {
        match core::str::from_utf8(&self.as_fs_str().as_encoded_bytes()) {
            Ok(_) => Ok(unsafe {
                let parts = (
                    self.inner.as_mut_ptr(),
                    self.inner.len(),
                    self.inner.capacity(),
                );
                let cast_bytes = Vec::from_raw_parts(parts.0.cast::<u8>(), parts.1, parts.2);
                String::from_utf8_unchecked(cast_bytes)
            }),
            Err(_) => Err(self),
        }
    }

    pub fn push<S: AsRef<FsStr>>(&mut self, s: S) {
        self.inner.extend_from_slice(&s.as_ref().inner);
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), alloc::collections::TryReserveError> {
        self.inner.try_reserve(additional)
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional);
    }

    pub fn try_reserve_exact(
        &mut self,
        additional: usize,
    ) -> Result<(), alloc::collections::TryReserveError> {
        self.inner.try_reserve_exact(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity);
    }

    pub fn into_boxed_fs_str(self) -> Box<FsStr> {
        let raw = Box::into_raw(self.inner.into_boxed_slice()) as *mut FsStr;
        unsafe { Box::from_raw(raw) }
    }

    pub fn leak<'a>(self) -> &'a mut FsStr {
        unsafe { &mut *(core::ptr::from_mut(self.inner.leak()) as *mut FsStr) }
    }
}
impl Debug for FsString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.as_encoded_bytes().utf8_chunks(), f)
    }
}

impl From<String> for FsString {
    fn from(value: String) -> Self {
        unsafe { Self::from_encoded_bytes_unchecked(value.into_bytes()) }
    }
}

impl<T: ?Sized + AsRef<FsStr>> From<&T> for FsString {
    /// Copies any value implementing <code>[AsRef]&lt;[OsStr]&gt;</code>
    /// into a newly allocated [`OsString`].
    fn from(s: &T) -> FsString {
        s.as_ref().to_fs_string()
    }
}

impl core::ops::Index<core::ops::RangeFull> for FsString {
    type Output = FsStr;

    fn index(&self, _index: core::ops::RangeFull) -> &FsStr {
        let me = unsafe { FsStr::from_inner(self.inner.as_slice()) };
        me
    }
}

impl core::ops::IndexMut<core::ops::RangeFull> for FsString {
    fn index_mut(&mut self, _index: core::ops::RangeFull) -> &mut FsStr {
        unsafe { FsStr::from_inner_mut(self.inner.as_mut_slice()) }
    }
}

impl core::ops::Deref for FsString {
    type Target = FsStr;

    fn deref(&self) -> &FsStr {
        &self[..]
    }
}
impl core::ops::DerefMut for FsString {
    fn deref_mut(&mut self) -> &mut FsStr {
        &mut self[..]
    }
}
