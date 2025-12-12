//! Debugging utilities for vexide.

#![cfg(target_os = "vexos")]

use std::{fmt::Debug, io::{self, Read, Stdin, Stdout, Write, stdin, stdout}};

mod debugger;
mod target;
mod arch;

pub use debugger::VexideDebugger;
use gdbstub::conn::{Connection, ConnectionExt};

/// A means of communicating with a debug console.
pub trait DebugIO: Connection<Error = io::Error> + ConnectionExt + Send + Clone {
}

/// Debug logging via stdio.
#[derive(Debug)]
pub struct StdioTransport {
    stdout: Stdout,
    stdin: Stdin,
}

impl StdioTransport {
    /// Create a new stdio-based transport.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stdout: stdout(),
            stdin: stdin(),
        }
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StdioTransport {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl DebugIO for StdioTransport {
}

impl Connection for StdioTransport {
    type Error = std::io::Error;

    fn write(&mut self, byte: u8) -> Result<(), Self::Error> {
        self.stdout.write_all(&[byte])?;
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.stdout.write_all(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.stdout.flush()
    }
}

impl ConnectionExt for StdioTransport {
    fn peek(&mut self) -> Result<Option<u8>, Self::Error> {
        let _lock = self.stdin.lock();

        let char = unsafe {
            vex_sdk::vexSerialPeekChar(1)
        };

        if char == -1 {
            return Ok(None);
        }

        Ok(Some(char as u8))
    }

    fn read(&mut self) -> Result<u8, Self::Error> {
        let mut buf = [0];
        self.stdin.read_exact(&mut buf)?;

        Ok(buf[0])
    }
}

impl Write for StdioTransport {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}
