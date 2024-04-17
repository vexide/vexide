use embedded_io_async::{self, ErrorKind, ErrorType, Read, Write};
use futures_core::Future;
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
impl ErrorType for StdoutRaw {
    type Error = ErrorKind;
}


struct BufferFullFuture {
    channel: u32,
}
impl Future for BufferFullFuture {
    type Output = ();

    fn poll(self: core::pin::Pin<&mut Self>, _: &mut core::task::Context<'_>) -> core::task::Poll<()> {
        if unsafe { vexSerialWriteFree(self.channel) } == 0 {
            core::task::Poll::Pending
        } else {
            core::task::Poll::Ready(())
        }
    }
}

impl Write for StdoutRaw {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        BufferFullFuture { channel: STDIO_CHANNEL }.await;
        let written =
            unsafe { vexSerialWriteBuffer(STDIO_CHANNEL, buf.as_ptr(), buf.len() as u32) };

        if written == -1 {
            return Err(ErrorKind::Other);
        }

        self.flush().await?;

        Ok(written as usize)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
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
impl ErrorType for StdoutLock<'_> {
    type Error = ErrorKind;
}
impl Write for StdoutLock<'_> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.inner.write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.inner.flush().await
    }
}

/// A handle to the serial output stream of this program.
pub struct Stdout;
impl ErrorType for Stdout {
    type Error = ErrorKind;
}
/// Contstructs a handle to the serial output stream
pub const fn stdout() -> Stdout {
    Stdout
}

impl Write for Stdout {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.lock().await.write(buf).await
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.lock().await.flush().await
    }
}

impl Stdout {
    /// The size of the internal VEXOs FIFO serial out buffer.
    pub const INTERNAL_BUFFER_SIZE: usize = 2048;

    /// Locks the stdout for writing.
    /// This function is blocking and will wait until the lock is acquired.
    pub async fn lock(&self) -> StdoutLock<'static> {
        StdoutLock {
            inner: STDOUT.lock().await,
        }
    }
}

struct StdinRaw;
impl ErrorType for StdinRaw {
    type Error = ErrorKind;
}
impl Read for StdinRaw {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
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
impl ErrorType for StdinLock<'_> {
    type Error = ErrorKind;
}
impl Read for StdinLock<'_> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.inner.read(buf).await
    }
}

/// A handle to the serial input stream of this program.
pub struct Stdin;
impl ErrorType for Stdin {
    type Error = ErrorKind;
}
impl Read for Stdin {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.lock().await.read(buf).await
    }
}

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
}

/// Constructs a handle to the serial input stream.
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
			use $crate::io::WriteExt;
			if let Err(e) = $crate::io::stdout().write_fmt(format_args!($($arg)*)) {
				panic!("failed printing to stdout: {e}");
			}
		}
    }};
}
pub use print;

#[macro_export]
/// Prints and returns the value of a given expression for quick and dirty debugging.
macro_rules! dbg {
    () => {
        $crate::println!("[{}:{}]", $file!(), $line!())
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
