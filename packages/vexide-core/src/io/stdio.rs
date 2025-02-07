use no_std_io::io::{self, Write};
use vex_sdk::{vexSerialReadChar, vexSerialWriteBuffer, vexSerialWriteFree};

use crate::sync::{Mutex, MutexGuard};

pub(crate) const STDIO_CHANNEL: u32 = 1;

static STDOUT: Mutex<StdoutRaw> = Mutex::new(StdoutRaw);
static STDIN: Mutex<StdinRaw> = Mutex::new(StdinRaw);

/// A handle to a raw instance of the serial output stream of this program.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `stdout_raw` function.
struct StdoutRaw;

impl io::Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written =
            unsafe { vexSerialWriteBuffer(STDIO_CHANNEL, buf.as_ptr(), buf.len() as u32) };

        if written == -1 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Internal write error occurred.",
            ));
        }

        self.flush()?;

        Ok(written as usize)
    }

    fn flush(&mut self) -> io::Result<()> {
        unsafe {
            while vexSerialWriteFree(STDIO_CHANNEL) < Stdout::INTERNAL_BUFFER_SIZE as _ {
                // Allow VEXos to flush the buffer by yielding.
                vex_sdk::vexTasksRun();
            }
        }

        Ok(())
    }
}

/// A locked serial output stream.
/// Only one of these can exist at a time and writes occur without waiting.
pub struct StdoutLock<'a> {
    inner: MutexGuard<'a, StdoutRaw>,
}

impl Write for StdoutLock<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// A handle to the serial output stream of this program.
///
/// An instance of this can be obtained using the [`stdout`] function.
pub struct Stdout(());

/// Constructs a handle to the serial output stream
#[must_use]
pub const fn stdout() -> Stdout {
    Stdout(())
}

impl Stdout {
    /// The size of the internal VEXOs FIFO serial out buffer.
    pub const INTERNAL_BUFFER_SIZE: usize = 2048;

    /// Locks the stdout for writing.
    /// This function will wait until the lock is acquired.
    pub async fn lock(&self) -> StdoutLock<'static> {
        StdoutLock {
            inner: STDOUT.lock().await,
        }
    }
    /// Attempts to lock the stdout for writing.
    ///
    /// This function will return `None` if the lock could not be acquired.
    pub fn try_lock(&self) -> Option<StdoutLock<'static>> {
        Some(StdoutLock {
            inner: STDOUT.try_lock()?,
        })
    }
}

struct StdinRaw;

impl io::Read for StdinRaw {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut iterator = buf.iter_mut();

        let mut byte: i32;
        let mut written: usize = 0;

        // Little but cursed, but hey it gets the job done...
        while {
            byte = unsafe { vexSerialReadChar(STDIO_CHANNEL) };
            byte != -1
        } {
            if let Some(next) = iterator.next() {
                *next = byte as u8;
                written += 1;
            } else {
                return Ok(written);
            }
        }

        Ok(written)
    }
}

/// A locked serial input stream.
/// Only one of these can exist at a time and reads occur without waiting.
pub struct StdinLock<'a> {
    inner: MutexGuard<'a, StdinRaw>,
}

impl io::Read for StdinLock<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

/// A handle to the serial input stream of this program.
///
/// An instance of this can be obtained using the [`stdin`] function.
pub struct Stdin(());

impl Stdin {
    /// The size of the internal VEXos serial in buffer.
    pub const STDIN_BUFFER_SIZE: usize = 4096;

    /// Locks the stdin for reading.
    /// This function is blocking and will wait until the lock is acquired.
    pub async fn lock(&self) -> StdinLock<'static> {
        StdinLock {
            inner: STDIN.lock().await,
        }
    }
    /// Attempts to lock the stdin for writing.
    ///
    /// This function will return `None` if the lock could not be acquired.
    pub fn try_lock(&self) -> Option<StdinLock<'static>> {
        Some(StdinLock {
            inner: STDIN.try_lock()?,
        })
    }
}

/// Constructs a handle to the serial input stream.
#[must_use]
pub const fn stdin() -> Stdin {
    Stdin(())
}

//////////////////////////////
// Printing implementations //
//////////////////////////////

#[doc(hidden)]
#[inline]
pub fn __print(args: core::fmt::Arguments<'_>) {
    use alloc::format;

    use crate::io::Write;

    // Panic on print if stdout is not available.
    // While this is less than ideal,
    // the alternative is either ingoring the print, a complete deadlock, or writing unsafely without locking.
    let mut stdout = stdout()
        .try_lock()
        .expect("Attempted to print while stdout was already locked.");

    // Format the arguments into a byte buffer before printing them.
    // This lets us calculate if the bytes will overflow the buffer before printing them.
    let formatted_bytes = format!("{args}").into_bytes();
    let remaining_bytes_in_buffer = unsafe { vexSerialWriteFree(STDIO_CHANNEL) as usize };

    // Write all of our data in chunks the size of the outgoing serial buffer.
    // This ensures that writes of length greater than [`Stdout::INTERNAL_BUFFER_SIZE`] can still be written
    // by flushing several times.
    for chunk in formatted_bytes.chunks(Stdout::INTERNAL_BUFFER_SIZE) {
        // If this chunk would overflow the buffer and cause a panic during `write_all`, wait for the buffer to clear.
        // Not only does this prevent a panic (if the panic handler prints it could cause a recursive panic and immediately exit. **Very bad**),
        // but it also allows prints and device comms inside of tight loops that have a print.
        //
        //TODO: In the future we may want to actually handle prints in tight loops
        //TODO: in a way that makes it more clear that that loop is hogging executor time.
        //TODO: In the past a lack of serial output was a tell that the executor was being hogged.
        //TODO: Flushing the serial buffer now removes this tell, though.
        if remaining_bytes_in_buffer < chunk.len() {
            // Flushing is infallible, so we can unwrap here.
            stdout.flush().unwrap();
        }

        // Re-use the buffer to write the formatted bytes to the serial output.
        // This technically should never error because we have already flushed the buffer if it would overflow.
        if let Err(e) = stdout.write_all(chunk) {
            panic!("failed printing to stdout: {e}");
        }
    }
}

#[macro_export]
/// Prints a message to the standard output and appends a newline.
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
		$crate::print!("{}\n", format_args!($($arg)*))
	};
}
pub use println;

#[macro_export]
/// Prints a message to the standard output.
macro_rules! print {
    ($($arg:tt)*) => {{
		$crate::io::__print(format_args!($($arg)*))
    }};
}
pub use print;

#[macro_export]
#[expect(
    edition_2024_expr_fragment_specifier,
    reason = "OK for this macro to accept `const {}` expressions"
)]
/// Prints and returns the value of a given expression for quick and dirty debugging.
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", file!(), line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::println!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
pub use dbg;
