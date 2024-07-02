//! File system API for the Brain SD card.

use vex_sdk::{vexFileOpen, vexFileOpenCreate, vexFileOpenWrite, vexFileRead, vexFileWrite};

use self::path::Path;
use crate::io;

pub mod path;

/// Represents a file in the file system.
pub struct File {
    fd: *mut vex_sdk::FIL,
}
impl File {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let file = unsafe { vexFileOpen(path.inner.as_ptr(), c"".as_ptr()) };
        if file.is_null() {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not open file",
            ))
        } else {
            Ok(Self { fd: file })
        }
    }

    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let file = unsafe {
            vexFileOpenWrite(path.inner.as_ptr())
        };
        if file.is_null() {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Could not create file",
            ))
        } else {
            Ok(Self { fd: file })
        }
    }
}
impl io::Write for File {
    fn write(&mut self, buf: &[u8]) -> no_std_io::io::Result<usize> {
        let buf_ptr = buf.as_ptr();
        let len = buf.len() as _;
        let written = unsafe {
            vexFileWrite(buf_ptr.cast_mut().cast(), 1, len, self.fd)
        };
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
        Ok(())
    }
}
impl io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> no_std_io::io::Result<usize> {
        let buf_ptr = buf.as_mut_ptr();
        let len = buf.len() as _;
        let read = unsafe { vexFileRead(buf_ptr.cast(), 1, len, self.fd) };
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
