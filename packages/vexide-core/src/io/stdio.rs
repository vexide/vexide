extern crate alloc;

use alloc::vec::Vec;
use no_std_io::io::{self, BufRead, Read, Write};
use vex_sdk::{vexBackgroundProcessing, vexSerialPeekChar, vexSerialReadChar, vexSerialWriteBuffer};

use crate::sync::{Mutex, MutexGuard};

pub const SERIAL_BUFFER_SIZE: usize = 2048;
const STDIO_CHANNEL: u32 = 1;

static STDOUT: Mutex<StdoutRaw> = Mutex::new(StdoutRaw);
static STDIN: Mutex<StdinRaw> = Mutex::new(StdinRaw);

/// A handle to a raw instance of the serial output stream of this program.
///
/// This handle is not synchronized or buffered in any fashion. Constructed via
/// the `stdout_raw` function.
struct StdoutRaw;

impl io::Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {

		let written = unsafe { vexSerialWriteBuffer(STDIO_CHANNEL, buf.as_ptr(), buf.len() as u32) };

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
		// Background processing flushes all serial FIFO buffers when it runs.
		unsafe {
			vexBackgroundProcessing();
		}

		Ok(())
	}
}

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

pub struct Stdout;

pub fn stdout() -> Stdout {
	Stdout
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.lock().write(buf)
	}

	fn flush(&mut self) -> io::Result<()> {
		self.lock().flush()
	}
}

impl Stdout {
	pub fn lock(&self) -> StdoutLock<'static> {
		StdoutLock {
			inner: STDOUT.lock_blocking(),
		}
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
			}  else {
				return Ok(written);
			}
		}

		Ok(written)
	}
}

pub struct StdinLock<'a> {
	inner: MutexGuard<'a, StdinRaw>,
}

impl io::Read for StdinLock<'_> {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.inner.read(buf)
	}
}

pub struct Stdin;

impl io::Read for Stdin {
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.lock().read(buf)
	}
}

impl Stdin {
	pub fn lock(&self) -> StdinLock<'static> {
		StdinLock {
			inner: STDIN.lock_blocking(),
		}
	}
}

pub fn stdin() -> Stdin {
	Stdin
}

#[macro_export]
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
macro_rules! print {
    ($($arg:tt)*) => {{
		{
			use $crate::io::Write;
			if let Err(e) = $crate::io::stdout().write_fmt(format_args!($($arg)*)) {
				panic!("failed printing to stdout: {e}");
			}
		}
    }};
}
pub use print;

#[macro_export]
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
