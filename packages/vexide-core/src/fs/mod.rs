//! Filesystem manipulation operations.
//!
//! This module contains basic methods to manipulate the contents of the brain's
//! micro SDCard. All methods in this module represent VEXos filesystem operations.
//!
//! # VEXos Limitations
//!
//! While this module largely mimicks Rust's `std::fs` API, there are several major
//! limitations in the VEXos filesystem. This module only provides a small subset of
//! what would normally be expected in a typical Rust enviornment. Notably:
//!
//! - Files cannot be opened as read and write at the same time (only one). To read a
//!   file that you’ve written to, you’ll need to drop your written file descriptor and
//!   reopen it as readonly.
//! - Files can be created, but not deleted or renamed.
//! - Directories cannot be created or enumerated from the Brain, only top-level files.

use alloc::{boxed::Box, ffi::CString, format, string::String, vec, vec::Vec};

use no_std_io::io::{Read, Seek, Write};

use crate::{
    io,
    path::{Path, PathBuf},
};

mod fs_str;

pub use fs_str::{Display, FsStr, FsString};

/// Options and flags which can be used to configure how a file is opened.
///
/// This builder exposes the ability to configure how a [`File`] is opened and
/// what operations are permitted on the open file. The [`File::open`] and
/// [`File::create`] methods are aliases for commonly used options using this
/// builder.
///
/// Generally speaking, when using `OpenOptions`, you'll first call
/// [`OpenOptions::new`], then chain calls to methods to set each option, then
/// call [`OpenOptions::open`], passing the path of the file you're trying to
/// open. This will give you a [`io::Result`] with a [`File`] inside that you
/// can further operate on.
///
/// # Limitations
///
/// - Files MUST be opened in either `read` XOR `write` mode.
/// - VEXos does not allow you to open a file configured as `read` and `write`
///   at the same time. Doing so will return an error with `File::open`. This is
///   a fundamental limtiation of the OS.
///
/// # Examples
///
/// Opening a file to read:
///
/// ```no_run
/// use vexide::fs::OpenOptions;
///
/// let file = OpenOptions::new().read(true).open("foo.txt");
/// ```
///
/// Opening a file for writing, as well as creating it if it doesn't exist:
///
/// ```no_run
/// use vexide::fs::OpenOptions;
///
/// let file = OpenOptions::new()
///             .write(true)
///             .create(true)
///             .open("foo.txt");
/// ```
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create_new: bool,
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    ///
    /// All options are initially set to `false`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs::OpenOptions;
    ///
    /// let mut options = OpenOptions::new();
    /// let file = options.read(true).open("foo.txt");
    /// ```
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub const fn new() -> OpenOptions {
        OpenOptions {
            read: false,
            write: false,
            append: false,
            truncate: false,
            create_new: false,
        }
    }

    /// Sets the option for read access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `read`-able if opened.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).open("foo.txt");
    /// ```
    pub const fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// Sets the option for write access.
    ///
    /// This option, when true, will indicate that the file should be
    /// `write`-able if opened.
    ///
    /// If the file already exists, any write calls on it will overwrite its
    /// contents, without truncating it.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).open("foo.txt");
    /// ```
    pub const fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// Sets the option for the append mode.
    ///
    /// This option, when true, means that writes will append to a file instead
    /// of overwriting previous contents.
    /// Note that setting `.write(true).append(true)` has the same effect as
    /// setting only `.append(true)`.
    ///
    /// Append mode guarantees that writes will be positioned at the current end of file,
    /// even when there are other processes or threads appending to the same file. This is
    /// unlike <code>[seek]\([SeekFrom]::[End]\(0))</code> followed by `write()`, which
    /// has a race between seeking and writing during which another writer can write, with
    /// our `write()` overwriting their data.
    ///
    /// Keep in mind that this does not necessarily guarantee that data appended by
    /// different processes or threads does not interleave. The amount of data accepted a
    /// single `write()` call depends on the operating system and file system. A
    /// successful `write()` is allowed to write only part of the given data, so even if
    /// you're careful to provide the whole message in a single call to `write()`, there
    /// is no guarantee that it will be written out in full. If you rely on the filesystem
    /// accepting the message in a single write, make sure that all data that belongs
    /// together is written in one operation. This can be done by concatenating strings
    /// before passing them to [`write()`].
    ///
    /// [SeekFrom]: io::SeekFrom
    /// [Start]: io::SeekFrom::End
    /// [End]: io::SeekFrom::End
    /// [Seek]: io::Seek::seek
    ///
    /// ## Note
    ///
    /// This function doesn't create the file if it doesn't exist. Use the
    /// [`OpenOptions::create`] method to do so.
    ///
    /// [`write()`]: Write::write "io::Write::write"
    /// [`flush()`]: Write::flush "io::Write::flush"
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().append(true).open("foo.txt");
    /// ```
    pub const fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// Sets the option for truncating a previous file.
    ///
    /// If a file is successfully opened with this option set it will truncate
    /// the file to 0 length if it already exists.
    ///
    /// The file must be opened with write access for truncate to work.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).truncate(true).open("foo.txt");
    /// ```
    pub const fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// Sets the option to create a new file, or open it if it already exists.
    ///
    /// In order for the file to be created, [`OpenOptions::write`] or
    /// [`OpenOptions::append`] access must be used.
    ///
    /// See also [`write()`][self::write] for a simple function to create a file
    /// with some given data.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true).create(true).open("foo.txt");
    /// ```
    pub const fn create(&mut self, create: bool) -> &mut Self {
        self.write = create;
        self
    }

    /// Sets the option to create a new file, failing if it already exists.
    ///
    /// No file is allowed to exist at the target location. In this way, if the call succeeds,
    /// the file returned is guaranteed to be new. If a file exists at the target location,
    /// creating a new file will fail with [`AlreadyExists`] or another error based on the
    /// situation. See [`OpenOptions::open`] for a non-exhaustive list of likely errors.
    ///
    /// If `.create_new(true)` is set, [`.create()`] and [`.truncate()`] are
    /// ignored.
    ///
    /// The file must be opened with write or append access in order to create
    /// a new file.
    ///
    /// [`.create()`]: OpenOptions::create
    /// [`.truncate()`]: OpenOptions::truncate
    /// [`AlreadyExists`]: io::ErrorKind::AlreadyExists
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().write(true)
    ///                              .create_new(true)
    ///                              .open("foo.txt");
    /// ```
    pub const fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    /// Opens a file at `path` with the options specified by `self`.
    ///
    /// # Errors
    ///
    /// This function will return an error under a number of different
    /// circumstances. Some of these error conditions are listed here, together
    /// with their [`io::ErrorKind`]. The mapping to [`io::ErrorKind`]s is not
    /// part of the compatibility contract of the function.
    ///
    /// * [`NotFound`]: The specified file does not exist and neither `create`
    ///   or `create_new` is set.
    /// * [`AlreadyExists`]: `create_new` was specified and the file already
    ///   exists.
    /// * [`InvalidInput`]: Invalid combinations of open options (read/write
    ///   access both specified, truncate without write access, no access mode
    ///   set, etc.).
    ///
    /// The following errors don't match any existing [`io::ErrorKind`] at the moment:
    /// * Filesystem-level errors: full disk, write permission
    ///   requested on a read-only file system, exceeded disk quota, too many
    ///   open files, too long filename.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs::OpenOptions;
    ///
    /// let file = OpenOptions::new().read(true).open("foo.txt");
    /// ```
    ///
    /// [`AlreadyExists`]: io::ErrorKind::AlreadyExists
    /// [`InvalidInput`]: io::ErrorKind::InvalidInput
    /// [`NotFound`]: io::ErrorKind::NotFound
    /// [`PermissionDenied`]: io::ErrorKind::PermissionDenied
    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        // Mount sdcard volume as FAT filesystem
        map_fresult(unsafe { vex_sdk::vexFileMountSD() })?;

        let path = path.as_ref();

        let path = CString::new(path.as_fs_str().as_encoded_bytes()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Path contained a null byte")
        })?;

        if self.write && self.read {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Files cannot be opened with read and write access",
            ));
        }
        if self.create_new {
            let file_exists = unsafe { vex_sdk::vexFileStatus(path.as_ptr()) };
            if file_exists != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "File already exists",
                ));
            }
        }

        let file = if self.read && !self.write {
            // The second argument to this function is ignored.
            // Open in read only mode
            unsafe { vex_sdk::vexFileOpen(path.as_ptr(), c"".as_ptr()) }
        } else if self.write && self.append {
            // Open in read/write and append mode
            unsafe { vex_sdk::vexFileOpenWrite(path.as_ptr()) }
        } else if self.write && self.truncate {
            // Open in read/write mode
            unsafe { vex_sdk::vexFileOpenCreate(path.as_ptr()) }
        } else if self.write {
            // Open in read/write and overwrite mode
            unsafe {
                // Open in read/write and append mode
                let fd = vex_sdk::vexFileOpenWrite(path.as_ptr());
                // Seek to beginning of the file
                vex_sdk::vexFileSeek(fd, 0, 0);

                fd
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Files cannot be opened without read or write access",
            ));
        };

        if file.is_null() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not open file",
            ))
        } else {
            Ok(File {
                fd: file,
                write: self.write,
            })
        }
    }
}

/// A structure representing a type of file with accessors for each file type.
/// It is returned by [`Metadata::file_type`] method.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct FileType {
    is_dir: bool,
}

impl FileType {
    /// Tests whether this file type represents a directory. The
    /// result is mutually exclusive to the results of [`is_file`];
    /// only one of these tests may pass.
    ///
    /// [`is_file`]: FileType::is_file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    /// let file_type = metadata.file_type();
    ///
    /// assert_eq!(file_type.is_dir(), false);
    /// ```
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        self.is_dir
    }

    /// Tests whether this file type represents a regular file. The
    /// result is mutually exclusive to the results of [`is_dir`];
    /// only one of these tests may pass.
    ///
    /// When the goal is simply to read from (or write to) the source, the most
    /// reliable way to test the source can be read (or written to) is to open
    /// it. See [`File::open`] or [`OpenOptions::open`] for more information.
    ///
    /// [`is_dir`]: FileType::is_dir
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    /// let file_type = metadata.file_type();
    ///
    /// assert_eq!(file_type.is_file(), true);
    /// ```
    #[must_use]
    pub const fn is_file(&self) -> bool {
        !self.is_dir
    }
}

/// Metadata information about a file.
///
/// This structure is returned from the [`metadata`] function or method
/// and represents known metadata about a file such as its size and type.
#[derive(Clone)]
pub struct Metadata {
    file_type: FileType,
    size: u64,
}

impl Metadata {
    fn from_fd(fd: *mut vex_sdk::FIL) -> io::Result<Self> {
        let size = unsafe { vex_sdk::vexFileSize(fd) };

        if size >= 0 {
            Ok(Self {
                size: size as u64,
                file_type: FileType { is_dir: false },
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to get file size",
            ))
        }
    }

    fn from_path(path: &Path) -> io::Result<Self> {
        let c_path = CString::new(path.as_fs_str().as_encoded_bytes()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Path contained a null byte")
        })?;

        let file_type = unsafe { vex_sdk::vexFileStatus(c_path.as_ptr()) };
        let is_dir = file_type == 3;

        // We can't get the size if its a directory because we cant open it as a file
        if is_dir {
            Ok(Self {
                size: 0,
                file_type: FileType { is_dir: true },
            })
        } else {
            let mut opts = OpenOptions::new();
            opts.read(true);
            let file = opts.open(path)?;
            let fd = file.fd;

            Self::from_fd(fd)
        }
    }

    /// Returns the file type for this metadata.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// fn main() -> std::io::Result<()> {
    ///     use vexide::fs;
    ///
    ///     let metadata = fs::metadata("foo.txt")?;
    ///
    ///     println!("{:?}", metadata.file_type());
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub const fn file_type(&self) -> FileType {
        self.file_type
    }

    /// Tests whether this file type represents a directory. The
    /// result is mutually exclusive to the results of [`is_file`];
    /// only one of these tests may pass.
    ///
    /// [`is_file`]: FileType::is_file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// assert!(!metadata.is_dir());
    /// ```
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        self.file_type.is_dir
    }

    /// Tests whether this file type represents a regular file. The
    /// result is mutually exclusive to the results of [`is_dir`];
    /// only one of these tests may pass.
    ///
    /// When the goal is simply to read from (or write to) the source, the most
    /// reliable way to test the source can be read (or written to) is to open
    /// it. See [`File::open`] or [`OpenOptions::open`] for more information.
    ///
    /// [`is_dir`]: FileType::is_dir
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// assert!(metadata.is_file());
    /// ```
    #[must_use]
    pub const fn is_file(&self) -> bool {
        !self.file_type.is_dir
    }

    /// Returns the size of the file, in bytes, this metadata is for.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::fs;
    ///
    /// let metadata = fs::metadata("foo.txt")?;
    ///
    /// assert_eq!(0, metadata.len());
    /// ```
    #[allow(clippy::len_without_is_empty)]
    #[must_use]
    pub fn len(&self) -> Option<u64> {
        self.file_type.is_dir.then_some(self.size)
    }
}

/// Represents a file in the file system.
pub struct File {
    fd: *mut vex_sdk::FIL,
    write: bool,
}
impl File {
    fn flush(&self) {
        unsafe {
            vex_sdk::vexFileSync(self.fd);
        }
    }

    fn tell(&self) -> io::Result<u64> {
        let position = unsafe { vex_sdk::vexFileTell(self.fd) };
        position.try_into().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to get current location in file",
            )
        })
    }

    /// Attempts to open a file in read-only mode.
    ///
    /// See the [`OpenOptions::open`] method for more details.
    ///
    /// If you only need to read the entire file contents, consider
    /// [`fs::read()`][self::read] or [`fs::read_to_string()`][self::read_to_string]
    /// instead.
    ///
    /// # Errors
    ///
    /// This function will return an error if `path` does not already exist.
    /// Other errors may also be returned according to [`OpenOptions::open`].
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        OpenOptions::new().read(true).open(path.as_ref())
    }

    /// Opens a file in write-only mode.
    ///
    /// This function will create a file if it does not exist,
    /// and will truncate it if it does.
    ///
    /// Depending on the platform, this function may fail if the
    /// full directory path does not exist.
    /// See the [`OpenOptions::open`] function for more details.
    ///
    /// See also [`fs::write()`][self::write] for a simple function to
    /// create a file with some given data.
    ///
    /// # Errors
    ///
    /// See [`OpenOptions::open`].
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path.as_ref())
    }

    /// Creates a new file in read-write mode; error if the file exists.
    ///
    /// This function will create a file if it does not exist, or return an error if it does. This
    /// way, if the call succeeds, the file returned is guaranteed to be new.
    /// If a file exists at the target location, creating a new file will fail with [`AlreadyExists`]
    /// or another error based on the situation. See [`OpenOptions::open`] for a
    /// non-exhaustive list of likely errors.
    ///
    /// This option is useful because it is atomic. Otherwise between checking whether a file
    /// exists and creating a new one, the file may have been created by another process (a TOCTOU
    /// race condition / attack).
    ///
    /// This can also be written using
    /// `File::options().read(true).write(true).create_new(true).open(...)`.
    ///
    /// [`AlreadyExists`]: crate::io::ErrorKind::AlreadyExists
    ///
    /// # Errors
    ///
    /// See [`OpenOptions::open`].
    pub fn create_new<P: AsRef<Path>>(path: P) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path.as_ref())
    }

    /// Returns a new OpenOptions object.
    ///
    /// This function returns a new OpenOptions object that you can use to
    /// open or create a file with specific options if `open()` or `create()`
    /// are not appropriate.
    ///
    /// It is equivalent to `OpenOptions::new()`, but allows you to write more
    /// readable code. Instead of
    /// `OpenOptions::new().append(true).open("example.log")`,
    /// you can write `File::options().append(true).open("example.log")`. This
    /// also avoids the need to import `OpenOptions`.
    ///
    /// See the [`OpenOptions::new`] function for more details.
    #[must_use]
    pub const fn options() -> OpenOptions {
        OpenOptions::new()
    }

    /// Queries metadata about the underlying file.
    ///
    /// # Errors
    ///
    /// * [`InvalidData`]: Internal filesystem error occurred.
    ///
    /// [`InvalidData`]: io::ErrorKind::InvalidData
    pub fn metadata(&self) -> io::Result<Metadata> {
        Metadata::from_fd(self.fd)
    }

    /// Attempts to sync all OS-internal file content and metadata to disk.
    ///
    /// This function will attempt to ensure that all in-memory data reaches the
    /// filesystem before returning.
    ///
    /// This can be used to handle errors that would otherwise only be caught
    /// when the `File` is closed, as dropping a `File` will ignore all errors.
    /// Note, however, that `sync_all` is generally more expensive than closing
    /// a file by dropping it, because the latter is not required to block until
    /// the data has been written to the filesystem.
    ///
    /// If synchronizing the metadata is not required, use [`sync_data`] instead.
    ///
    /// [`sync_data`]: File::sync_data
    ///
    /// # Errors
    ///
    /// This function is infallible.
    pub fn sync_all(&self) -> io::Result<()> {
        self.flush();
        Ok(())
    }

    /// This function is similar to [`sync_all`], except that it might not
    /// synchronize file metadata to the filesystem.
    ///
    /// This is intended for use cases that must synchronize content, but don't
    /// need the metadata on disk. The goal of this method is to reduce disk
    /// operations.
    ///
    /// Note that some platforms may simply implement this in terms of
    /// [`sync_all`].
    ///
    /// [`sync_all`]: File::sync_all
    ///
    /// # Errors
    ///
    /// This function is infallible.
    pub fn sync_data(&self) -> io::Result<()> {
        self.flush();
        Ok(())
    }
}
impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> no_std_io::io::Result<usize> {
        if !self.write {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Files opened in read mode cannot be written to.",
            ));
        }

        let len = buf.len();
        let buf_ptr = buf.as_ptr();
        let written =
            unsafe { vex_sdk::vexFileWrite(buf_ptr.cast_mut().cast(), 1, len as _, self.fd) };
        if written < 0 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not write to file",
            ))
        } else {
            Ok(written as usize)
        }
    }

    fn flush(&mut self) -> no_std_io::io::Result<()> {
        File::flush(self);
        Ok(())
    }
}
impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> no_std_io::io::Result<usize> {
        if self.write {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Files opened in write mode cannot be read from.",
            ));
        }

        let len = buf.len() as _;
        let buf_ptr = buf.as_mut_ptr();
        let read = unsafe { vex_sdk::vexFileRead(buf_ptr.cast(), 1, len, self.fd) };
        if read < 0 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not read from file",
            ))
        } else {
            Ok(read as usize)
        }
    }
}

impl Seek for File {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        const SEEK_SET: i32 = 0;
        const SEEK_CUR: i32 = 1;
        const SEEK_END: i32 = 2;

        fn try_convert_offset<T: TryInto<u32>>(offset: T) -> io::Result<u32> {
            offset.try_into().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Cannot seek to an offset too large to fit in a 32 bit integer",
                )
            })
        }

        match pos {
            io::SeekFrom::Start(offset) => unsafe {
                map_fresult(vex_sdk::vexFileSeek(
                    self.fd,
                    try_convert_offset(offset)?,
                    SEEK_SET,
                ))?;
            },
            io::SeekFrom::End(offset) => unsafe {
                if offset >= 0 {
                    map_fresult(vex_sdk::vexFileSeek(
                        self.fd,
                        try_convert_offset(offset)?,
                        SEEK_END,
                    ))?;
                } else {
                    // `vexFileSeek` does not support seeking with negative offset, meaning
                    // we have to calculate the offset from the end of the file ourselves.
                    map_fresult(vex_sdk::vexFileSeek(
                        self.fd,
                        try_convert_offset((self.metadata()?.size as i64) + offset)?,
                        SEEK_SET,
                    ))?;
                }
            },
            io::SeekFrom::Current(offset) => unsafe {
                if offset >= 0 {
                    map_fresult(vex_sdk::vexFileSeek(
                        self.fd,
                        try_convert_offset(offset)?,
                        SEEK_CUR,
                    ))?;
                } else {
                    // `vexFileSeek` does not support seeking with negative offset, meaning
                    // we have to calculate the offset from the stream position ourselves.
                    map_fresult(vex_sdk::vexFileSeek(
                        self.fd,
                        try_convert_offset((self.tell()? as i64) + offset)?,
                        SEEK_SET,
                    ))?;
                }
            },
        }

        self.tell()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        // We do not need to sync because vexFileClose will do that for us
        unsafe {
            vex_sdk::vexFileClose(self.fd);
        }
    }
}

/// An entry returned by the [`ReadDir`] iterator.
///
/// A `DirEntry` represents an item within a directory on the Brain's Micro SD card.
/// The Brain provides very little metadata on files, so only the base std `DirEntry` methods are supported.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DirEntry {
    base: FsString,
    name: FsString,
}
impl DirEntry {
    /// Returns the full path to the directory item.
    ///
    /// This path is creeated by joining the path of the call to [`read_dir`] to the name of the file.
    ///
    /// # Examples
    ///
    /// ```
    /// for entry in fs::read_dir(".").unwrap() {
    ///    println!("{:?}", entry.path());
    /// }
    /// ```
    ///
    /// This example will lead to output like:
    /// ```text
    /// "somefile.txt"
    /// "breakingbadseason1.mp4"
    /// "badapple.mp3"
    /// ```
    #[must_use]
    pub fn path(&self) -> PathBuf {
        PathBuf::from(format!("{}/{}", self.base.display(), self.name.display()))
    }

    /// Returns the metadata for the full path to the item.
    ///
    /// This is equivalent to calling [`metadata`] with the output from [`DirEntry::path`].
    ///
    /// # Errors
    ///
    /// This function will error if the path does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// for entry in read_dir("somepath") {
    ///     println!(
    ///         "{:?} is a {}.",
    ///         entry.path(),
    ///         match entry.metadata().is_file() {
    ///             true => "file",
    ///             false => "folder"
    ///         }
    ///     );
    /// }
    /// ```
    pub fn metadata(&self) -> io::Result<Metadata> {
        let path = self.path();
        Metadata::from_path(&path)
    }

    /// Returns the file type of the file that this [`DirEntry`] points to.
    ///
    /// This function is equivalent to getting the [`FileType`] from the metadata returned by [`DirEntry::metadata`].
    ///
    /// # Errors
    ///
    /// This function will error if the path does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// for entry in read_dir("somepath") {
    ///     println!(
    ///         "{:?} is a {}.",
    ///         entry.path(),
    ///         match entry.file_type().is_file() {
    ///             true => "file",
    ///             false => "folder"
    ///         }
    ///     );
    /// }
    /// ```
    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(self.metadata()?.file_type)
    }

    /// Returns the name of the file not including any leading components.
    ///
    /// The following paths will all lead to a file name of `foo`:
    /// - `./foo`
    /// - `../foo`
    /// - `/some/global/foo`
    #[must_use]
    pub fn file_name(&self) -> FsString {
        self.name.clone()
    }
}

/// An iterator over the entries of a directory.
///
/// This iterator is returned from [`read_dir`] and will yield items of type [`DirEntry`].
/// Information about the path is exposed through the [`DirEntry`]
///
/// Unlike the equivalent iterator in the standard library,
/// this iterator does not return results as it is infallible.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadDir {
    idx: usize,
    filenames: Box<[Option<FsString>]>,
    base: FsString,
}
impl Iterator for ReadDir {
    type Item = DirEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.filenames.len() {
            return None;
        }
        let entry = DirEntry {
            base: self.base.clone(),
            name: self.filenames[self.idx].take().unwrap(),
        };
        self.idx += 1;
        Some(entry)
    }
}

/// Returns an iterator over the items in a directory.
///
/// The returned [`ReadDir`] iterator will yield items of type [`DirEntry`].
/// This is slightly different from the standard library API which yields items of type `io::Result<DirEntry>`.
/// This is due to the fact that all directory items are gathered at iterator creation and the iterator itself is infallible.
///
/// # Errors
///
/// This function will error if:
/// - The given path does not exist
/// - The given path does not point to a directory.
///
/// # Examples
///
/// ```
/// for entry in vexide::fs::read_dir("somefolder") {
///     println!("{:?}", entry.path);
/// }
/// ```
pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<ReadDir> {
    let path = path.as_ref();
    let meta = metadata(path)?;
    if meta.is_file() {
        return Err(io::Error::new(
            no_std_io::io::ErrorKind::InvalidInput,
            "Cannot read the entries of a path that is not a directory.",
        ));
    }

    let c_path = CString::new(path.as_fs_str().as_encoded_bytes())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Path contained a null byte"))?;

    let mut size_guess = 1024;
    let mut last_buffer_size = None;

    let mut filename_buffer;
    loop {
        filename_buffer = vec![0; size_guess];

        unsafe {
            map_fresult(vex_sdk::vexFileDirectoryGet(
                c_path.as_ptr(),
                filename_buffer.as_mut_ptr().cast(),
                size_guess as _,
            ))?;
        }

        let mut len = 0;
        for (i, byte) in filename_buffer.iter().enumerate().rev() {
            if *byte != 0 {
                len = i;
                break;
            }
        }

        if last_buffer_size == Some(len) {
            break;
        }

        last_buffer_size.replace(len);
        size_guess *= 2;
    }

    let mut file_names = vec![];

    let fs_str = unsafe { FsStr::from_inner(&filename_buffer) };

    let mut filename_start_idx = 0;
    for (i, byte) in fs_str.as_encoded_bytes().iter().enumerate() {
        if *byte == b'\n' {
            let filename = &fs_str.as_encoded_bytes()[filename_start_idx..i];
            let filename = unsafe { FsString::from_encoded_bytes_unchecked(filename.to_vec()) };
            file_names.push(Some(filename));
            filename_start_idx = i + 1;
        }
    }

    let base = path.inner.to_fs_string();

    Ok(ReadDir {
        idx: 0,
        filenames: file_names.into_boxed_slice(),
        base,
    })
}

fn map_fresult(fresult: vex_sdk::FRESULT) -> io::Result<()> {
    // VEX presumably uses a derivative of FatFs (most likely the xilffs library)
    // for sdcard filesystem functions.
    //
    // Documentation for each FRESULT originates from here:
    // <http://elm-chan.org/fsw/ff/doc/rc.html>
    match fresult {
        vex_sdk::FRESULT::FR_OK => Ok(()),
        vex_sdk::FRESULT::FR_DISK_ERR => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "internal function reported an unrecoverable hard error",
        )),
        vex_sdk::FRESULT::FR_INT_ERR => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "assertion failed and an insanity is detected in the internal process",
        )),
        vex_sdk::FRESULT::FR_NOT_READY => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "the storage device could not be prepared to work",
        )),
        vex_sdk::FRESULT::FR_NO_FILE => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "could not find the file in the directory",
        )),
        vex_sdk::FRESULT::FR_NO_PATH => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "a directory in the path name could not be found",
        )),
        vex_sdk::FRESULT::FR_INVALID_NAME => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the given string is invalid as a path name",
        )),
        vex_sdk::FRESULT::FR_DENIED => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the required access for this operation was denied",
        )),
        vex_sdk::FRESULT::FR_EXIST => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "an object with the same name already exists in the directory",
        )),
        vex_sdk::FRESULT::FR_INVALID_OBJECT => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "invalid or null file/directory object",
        )),
        vex_sdk::FRESULT::FR_WRITE_PROTECTED => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "a write operation was performed on write-protected media",
        )),
        vex_sdk::FRESULT::FR_INVALID_DRIVE => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "an invalid drive number was specified in the path name",
        )),
        vex_sdk::FRESULT::FR_NOT_ENABLED => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "work area for the logical drive has not been registered",
        )),
        vex_sdk::FRESULT::FR_NO_FILESYSTEM => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "valid FAT volume could not be found on the drive",
        )),
        vex_sdk::FRESULT::FR_MKFS_ABORTED => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "failed to create filesystem volume",
        )),
        vex_sdk::FRESULT::FR_TIMEOUT => Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "the function was canceled due to a timeout of thread-safe control",
        )),
        vex_sdk::FRESULT::FR_LOCKED => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "the operation to the object was rejected by file sharing control",
        )),
        vex_sdk::FRESULT::FR_NOT_ENOUGH_CORE => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "not enough memory for the operation",
        )),
        vex_sdk::FRESULT::FR_TOO_MANY_OPEN_FILES => Err(io::Error::new(
            io::ErrorKind::Uncategorized,
            "maximum number of open files has been reached",
        )),
        vex_sdk::FRESULT::FR_INVALID_PARAMETER => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "a given parameter was invalid",
        )),
        _ => unreachable!(), // C-style enum
    }
}

/// Copies the contents of one file to another.
///
/// If the destination file does not exist, it will be created.
/// If it does exist, it will be overwritten.
///
/// # Errors
///
/// This function will error if the source file does not exist
/// or any other error according to [`OpenOptions::open`].
pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
    let from = read(from)?;
    let mut to = File::create(to)?;
    // Not completely accurate to std, but this is the best we've got
    let len = from.len() as u64;

    to.write_all(&from)?;

    Ok(len)
}

/// Returns true if the path points to a file that exists on the filesystem.
///
/// Unlike in the standard library, this function cannot fail because there are not permissions.
///
/// # Examples
///
/// ```
/// use vexide::fs::*;
///
/// assert!(exists("existent.txt"));
/// assert!(!exists("nonexistent.txt"));
/// ```
pub fn exists<P: AsRef<Path>>(path: P) -> bool {
    let file_exists = unsafe { vex_sdk::vexFileStatus(path.as_ref().as_fs_str().as_ptr().cast()) };
    // Woop woop we've got a nullptr!
    file_exists != 0
}

/// Gets the metadata for a file or path.
///
/// # Errors
///
/// This function will error if the path doesn't exist.
pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    Metadata::from_path(path.as_ref())
}

/// Reads the entire contents of a file into a vector.
///
/// This is a convenience function for using [`File::open`] and [`Read::read_to_end`].
///
/// # Errors
///
/// This function will error if the path doesn't exist
/// or any other error according to [`OpenOptions::open`].
pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Reads the entire contents of a file into a string.
///
/// This is a convenience function for using [`File::open`], [`Read::read_to_end`], and [`String::from_utf8`] (no_std_io does not support read_to_string).
///
/// # Errors
///
/// This function will error if the path doesn't exist, if the file is not valid UTF-8,
/// or any other error according to [`OpenOptions::open`].
pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let string = String::from_utf8(buf)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "File was not valid UTF-8"))?;
    Ok(string)
}

/// Writes an entire buffer to a file, replacing its contents.
///
/// This function will create a new file if it does not exist.
///
/// This is a convenience function for using [`File::create`] and [`Write::write_all`].
///
/// # Errors
///
/// This function will error if the path is invalid or for any other error according to [`OpenOptions::open`].
pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(contents.as_ref())
}
