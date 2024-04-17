//! Serial input and output functionality.
//!
//! This module aims to provide a very similar API to the Rust standard library's `std::io` module.

mod stdio;

#[derive(Debug)]
pub enum WriteFmtError<E> {
    /// An error was encountered while formatting.
    FmtError,
    /// Error returned by the inner Write.
    Other(E),
}

impl<E> From<E> for WriteFmtError<E> {
    fn from(err: E) -> Self {
        Self::Other(err)
    }
}

impl<E: core::fmt::Debug> core::fmt::Display for WriteFmtError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

pub trait WriteExt: Write {
    fn write_fmt(&mut self, fmt: core::fmt::Arguments<'_>) -> Result<(), WriteFmtError<Self::Error>> {
        // Create a shim which translates a Write to a fmt::Write and saves
        // off I/O errors. instead of discarding them
        struct Adapter<'a, T: Write + ?Sized + 'a> {
            inner: &'a mut T,
            error: Result<(), T::Error>,
        }

        impl<T: Write + ?Sized> core::fmt::Write for Adapter<'_, T> {
            fn write_str(&mut self, s: &str) -> core::fmt::Result {
                let mut fut = Box::pin(self.inner.write_all(s.as_bytes()));
                let res = loop {
                    if let core::task::Poll::Ready(res) =  fut.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
                        break res;
                    }
                };
                match res {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(core::fmt::Error)
                    }
                }
            }
        }

        let mut output = Adapter {
            inner: self,
            error: Ok(()),
        };
        match core::fmt::write(&mut output, fmt) {
            Ok(()) => Ok(()),
            Err(..) => match output.error {
                // check if the error came from the underlying `Write` or not
                Err(e) => Err(WriteFmtError::Other(e)),
                Ok(()) => Err(WriteFmtError::FmtError),
            },
        }
    }
}

impl <T: Write> WriteExt for T {}

use core::task::{Context, Waker};

use alloc::boxed::Box;
pub use embedded_io_async::*;
use futures_core::Future;
pub(crate) use stdio::STDIO_CHANNEL;
pub use stdio::{dbg, print, println, stdin, stdout, Stdin, StdinLock, Stdout, StdoutLock};
