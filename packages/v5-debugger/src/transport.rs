use std::{fmt::Debug, io};

use gdbstub::conn::{Connection, ConnectionExt};
use vex_sdk::{vexSerialWriteChar, vexSerialWriteFree, vexTasksRun};
use vexide_devices::display::Display;

/// A means of communicating with a debug console.
pub trait Transport: Connection<Error = io::Error> + ConnectionExt + Send + Clone {}

/// Debug logging via stdio.
#[derive(Debug)]
pub struct StdioTransport {
    display: Display,
}

impl StdioTransport {
    /// Create a new stdio-based transport.
    #[must_use]
    pub fn new() -> Self {
        Self {
            display: unsafe { Display::new() },
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

impl Transport for StdioTransport {}

impl Connection for StdioTransport {
    type Error = std::io::Error;

    fn write(&mut self, byte: u8) -> Result<(), Self::Error> {
        {
            use std::fmt::Write;
            _ = write!(self.display, "{}", byte as char);
        }

        if unsafe { vexSerialWriteFree(1) } == 0 {
            drop(self.flush());
        }
        _ = unsafe { vexSerialWriteChar(1, byte as _) };

        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        {
            use std::fmt::Write;
            _ = write!(self.display, "{}", str::from_utf8(buf).unwrap());
        }

        for chunk in buf.chunks(2048) {
            if unsafe { vex_sdk::vexSerialWriteFree(1) as usize } < chunk.len() {
                self.flush().unwrap();
            }

            let count =
                unsafe { vex_sdk::vexSerialWriteBuffer(1, chunk.as_ptr(), chunk.len() as u32) }
                    as usize;

            // This is a sanity check to ensure that we don't end up with non-contiguous
            // buffer writes. e.g. a chunk gets only partially written, but we continue
            // attempting to write the remaining chunks.
            //
            // In practice, this should never really occur since the previous flush ensures
            // enough space in FIFO to write the entire chunk to vexSerialWriteBuffer.
            if count != chunk.len() {
                break;
            }
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        unsafe {
            while (vex_sdk::vexSerialWriteFree(1) as usize) != 2048 {
                vex_sdk::vexTasksRun();
            }
        }

        Ok(())
    }
}

impl ConnectionExt for StdioTransport {
    fn peek(&mut self) -> Result<Option<u8>, Self::Error> {
        let char = unsafe { vex_sdk::vexSerialPeekChar(1) };

        if char == -1 {
            return Ok(None);
        }

        Ok(Some(char as u8))
    }

    fn read(&mut self) -> Result<u8, Self::Error> {
        loop {
            let c = unsafe { vex_sdk::vexSerialReadChar(1) };

            if c != -1 {
                return Ok(c as u8);
            }

            unsafe {
                vexTasksRun();
            }
        }
    }
}
