//! Path manipulation.
//!
//! This module provides the [`Path`] type for working with VEXos filesystem paths
//! abstractly. Paths are case sensitive.

use crate::fs::FsStr;

/// A slice of a path (akin to [`str`]).
///
/// More details about the overall approach can be found in the [module documentation](self).
#[repr(transparent)]
pub struct Path {
    pub(crate) inner: FsStr,
}

impl Path {
    /// Directly wraps a string slice as a `Path` slice.
    ///
    /// This is a cost-free conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::core::path::Path;
    ///
    /// Path::new("foo.txt");
    /// ```
    ///
    /// You can create `Path`s from `String`s, or even other `Path`s:
    ///
    /// ```
    /// use alloc::string::String;
    /// use vexide::core::path::Path;
    ///
    /// let string = String::from("foo.txt");
    /// let from_string = Path::new(&string);
    /// let from_path = Path::new(&from_string);
    /// assert_eq!(from_string, from_path);
    /// ```
    pub fn new<'a, P: AsRef<FsStr> + 'a>(path: P) -> &'a Self {
        unsafe { &*(core::ptr::from_ref::<FsStr>(path.as_ref()) as *const Path) }
    }

    /// Yields the underlying [`FsStr`] slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::core::{path::Path, fs::FsStr};
    ///
    /// let fs_str = Path::new("foo.txt").as_os_str();
    /// assert_eq!(os_str, FsStr::new("foo.txt"));
    /// ```
    #[must_use]
    pub const fn as_fs_str(&self) -> &FsStr {
        &self.inner
    }
}
impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        self
    }
}
impl AsRef<Path> for &str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}
