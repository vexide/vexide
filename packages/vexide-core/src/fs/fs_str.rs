use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};
use core::{fmt::Debug, mem::ManuallyDrop, ptr};

/// A type that allows safely displaying [`FsStr`]s that may contain non-UTF-8 data.
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
    /// use vexide::fs::FsStr;
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
    /// # Note
    ///
    /// As the encoding is unspecified, any sub-slice of bytes that is not valid UTF-8 should
    /// be treated as opaque and only comparable within the same Rust version built for the same
    /// target platform.  For example, sending the slice over the network or storing it in a file
    /// will likely result in incompatible byte slices.
    #[must_use]
    pub const fn as_encoded_bytes(&self) -> &[u8] {
        unsafe { &*(ptr::from_ref::<[i8]>(&self.inner) as *const [u8]) }
    }

    /// Copies this [`FsStr`] into an owned [`FsString`].
    ///
    /// # Examples
    ///
    /// ```
    /// let fs_str = FsStr::new("foo");
    /// let fs_string = fs_str.to_fs_string();
    /// assert_eq!(ofs_string.as_encoded_bytes(), fs_str.as_encoded_bytes());
    /// ```
    #[must_use]
    pub fn to_fs_string(&self) -> FsString {
        FsString {
            inner: self.inner.to_vec(),
        }
    }

    /// Converts an [`FsStr`] into a UTF-8 encoded string.
    ///
    /// Any non-UTF-8 sequences are replaced with the unicode
    /// [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD].
    ///
    /// [U+FFFD]: core::char::REPLACEMENT_CHARACTER
    #[must_use]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.as_encoded_bytes())
    }

    /// Checks whether or not the [`FsStr`] is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let fs_str = FsStr::new("");
    /// assert!(fs_str.is_empty());
    ///
    /// let fs_str = FsStr::new("foo");
    /// assert!(!fs_str.is_empty());
    /// ```
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    /// Returns the length of the [`FsStr`].
    ///
    /// # Examples
    ///
    /// ```
    /// let fs_str = FsStr::new("");
    /// assert_eq!(fs_str.len(), 0);
    ///
    /// let fs_str = FsStr::new("foo");
    /// assert_eq!(fs_str.len(), 3);
    /// ```
    #[must_use]
    pub const fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns a [`Display`] which can be used to display [`FsStr`]s that may contain non-UTF-8 data.
    /// This may perform lossy conversions. For an implementation that escapes the data, use [`Debug`].
    #[must_use]
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

/// A type that represents a mutable and owned VEXos filesystem string,
/// while being cheaply inter-convertible with Rust strings.
///
/// [`FsString`] is NOT null terminated. If you need to pass this to VEX SDK filesystem functions,
/// create a [`CStr`](core::ffi::CStr).
///
/// An [`FsString`] is to [`&FsStr`](FsStr) as [`String`] is to [`&str`](str);
/// the former are owned, while the latter are borrowed references.
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FsString {
    inner: Vec<i8>,
}
impl FsString {
    /// Allocates a new, empty, [`FsString`].
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Creates a new [`FsString`] from the raw encoded bytes.
    ///
    /// # Safety
    ///
    /// This function does not check if there are nul bytes withing or terminating the string.
    /// Passing an invalid [`FsString`] to VEX SDK functions can have unintended consequences,
    /// including undefined behavior.
    #[must_use]
    pub unsafe fn from_encoded_bytes_unchecked(bytes: Vec<u8>) -> Self {
        let mut bytes = ManuallyDrop::new(bytes);
        unsafe {
            let parts = (bytes.as_mut_ptr(), bytes.len(), bytes.capacity());
            let cast_bytes = Vec::from_raw_parts(parts.0.cast::<i8>(), parts.1, parts.2);
            Self { inner: cast_bytes }
        }
    }

    /// Borrows an [`FsString`] as an [`FsStr`].
    ///
    /// This is akin to taking a slice of the entire [`FsString`]
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // false-positive. can't be const dereffed
    pub fn as_fs_str(&self) -> &FsStr {
        self
    }

    /// Returns the raw encoded bytes of the [`FsString`]
    ///
    //TODO: VERIFY BEFORE DOC COMMENTING
    // The format of this data should be 7-bit ASCII which is the string format used in FAT32.
    #[must_use]
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

    /// Converts the [`FsString`] to a [`String`], or returns the original [`FsString`] on failure.
    ///
    /// # Errors
    ///
    /// This function will return the original string in the `Err` variant if the [`FsString`] was not valid UTF-8
    pub fn into_string(mut self) -> Result<String, FsString> {
        match core::str::from_utf8(self.as_fs_str().as_encoded_bytes()) {
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

    /// Extends the [`FsString`] with the given string.
    ///
    /// As an example, the stored data in this example will be equivalent to "foobarbaz":
    /// ```
    /// let mut foo = FsString::new();
    /// foo.push("foo");
    /// foo.push("bar");
    /// foo.push("baz");
    /// ```
    pub fn push<S: AsRef<FsStr>>(&mut self, s: S) {
        self.inner.extend_from_slice(&s.as_ref().inner);
    }

    /// Creates a new [`FsString`] with the given capacity.
    /// For more information on capacity, look at the [`Vec::with_capacity`] documentation.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
        }
    }

    /// Clears all data from the [`FsString`].
    /// This has no effect on the capacity of the [`FsString`]
    ///
    /// # Examples
    ///
    /// ```
    /// let mut string = FsString::with_capacity(100);
    /// string.push("hello :3");
    /// assert!(string.capacity() >= 100);
    /// string.clear();
    /// assert!(string.capacity() >= 100)
    /// ```
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Returns the capacity of the [`FsString`].
    /// The capacity of the string will be equivalent to the maximum number of characters unless special characters are used.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Reserves at least `additional` bytes in the [`FsString`].
    ///
    /// In order to reduce allocations, this function will often reserve more than `additional` bytes.
    /// For more information on the logic behind reserving bytes, see the [`Vec::reserve`] documentation.
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Attempts to reserve at least `additional` bytes in the [`FsString`].
    ///
    ///
    /// In order to reduce allocations, this function will often reserve more than `additional` bytes.
    /// For more information on the logic behind reserving bytes, see the [`Vec::reserve`] documentation.
    ///
    /// # Errors
    ///
    /// This function will error under the following conditions:
    ///
    /// - The capacity of the [`FsString`] has surpasses [`isize::MAX`] bytes
    /// - An allocation error occured while reserving space.
    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), alloc::collections::TryReserveError> {
        self.inner.try_reserve(additional)
    }

    /// Reserves `additional` bytes in the [`FsString`].
    ///
    /// # Note
    ///
    /// Reserving an exact amount of times can often negatively impact performace.
    /// you most likely want to use [`FsString::reserve`]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional);
    }

    /// Attempts to reserve `additional` bytes in the [`FsString`].
    ///
    /// # Note
    ///
    /// Reserving an exact amount of times can often negatively impact performace.
    /// you most likely want to use [`FsString::reserve`]
    ///
    /// # Errors
    /// This function will error under the following conditions:
    ///
    /// - The capacity of the [`FsString`] has surpasses [`isize::MAX`] bytes
    /// - An allocation error occured while reserving space.
    pub fn try_reserve_exact(
        &mut self,
        additional: usize,
    ) -> Result<(), alloc::collections::TryReserveError> {
        self.inner.try_reserve_exact(additional)
    }

    /// Shrinks the capacity of the [`FsString`] as much as possible.
    ///
    /// Depending on the allocator implementation, the [`FsString`] may still have extra capacity.
    /// See [`core::alloc::Allocator::shrink`] for more info.
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    /// Shrinks the capacity of the [`FsString`] to a lower bound.
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity);
    }

    /// Consumes and converts this [`FsString`] into a `Box<FsStr>`.
    /// Excess capacity is discarded.
    #[must_use]
    pub fn into_boxed_fs_str(self) -> Box<FsStr> {
        let raw = Box::into_raw(self.inner.into_boxed_slice()) as *mut FsStr;
        unsafe { Box::from_raw(raw) }
    }

    /// Consumes and leaks this [`FsString`] and converts it to a [`&FsStr`](FsStr).
    #[must_use]
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
    /// Copies any value implementing <code>[AsRef]&lt;[FsStr]&gt;</code>
    /// into a newly allocated [`FsString`].
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
