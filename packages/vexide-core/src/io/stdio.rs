use no_std_io::io::{self, Write};
use vex_sdk::{vexSerialReadChar, vexSerialWriteBuffer};

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
        // Serial buffers are automatically flushed every 2mS by vexTasksRun
        // in our background processing task.
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
pub struct Stdout;

/// Constructs a handle to the serial output stream
#[must_use]
pub const fn stdout() -> Stdout {
    Stdout
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
pub struct Stdin;

impl Stdin {
    /// The size of the internal VEXOs serial in buffer.
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
    Stdin
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
		{
			use $crate::io::Write;
            // Panic on print if stdout is not available.
            // While this is less than ideal,
            // the alternative is either ingoring the print, a complete deadlock, or writing unsafely without locking.
            let mut stdout =  $crate::io::stdout().try_lock().expect("Attempted to print while stdout was already locked.");
            if let Err(e) = stdout.write_fmt(format_args!($($arg)*)) {
                panic!("failed printing to stdout: {e}");
            }
		}
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
