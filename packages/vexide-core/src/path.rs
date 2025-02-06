//! Path manipulation.
//!
//! This module provides the [`Path`] type for working with VEXos filesystem paths
//! abstractly. Paths are case sensitive.

use alloc::string::String;
use core::{borrow::Borrow, ops::Deref};

use crate::fs::{FsStr, FsString};

/// A slice of a path (akin to [`str`]).
///
/// More details about the overall approach can be found in the [module documentation](self).
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// An owned, mutable path akin to a `String
///
/// This type implements `Deref` to [`Path`],
/// menaing all methods on a [`Path`] can be used on a [`PathBuf`].
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathBuf {
    inner: FsString,
}
//TODO: reimplement std's funcs
impl PathBuf {
    /// Allocates a new, empty, path.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            inner: FsString::new(),
        }
    }

    fn as_path(&self) -> &Path {
        Path::new(self.inner.as_fs_str())
    }
}

impl From<String> for PathBuf {
    fn from(value: String) -> Self {
        Self {
            inner: FsString::from(value),
        }
    }
}
impl Deref for PathBuf {
    type Target = Path;

    fn deref(&self) -> &Path {
        self.as_path()
    }
}

impl Borrow<Path> for PathBuf {
    fn borrow(&self) -> &Path {
        self.as_path()
    }
}
impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}
