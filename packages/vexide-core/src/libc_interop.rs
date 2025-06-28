#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    clippy::upper_case_acronyms
)]

use core::ffi::{c_char, c_int};

use no_std_io::io::{Read, Write};

use crate::io::{stdin, stdout};

mod picolibc {
    // Force linking to the C standard library.
    #[link(name = "c")]
    unsafe extern "C" {}

    use core::ffi::{c_char, c_int};

    use bitflags::bitflags;

    pub const EOF: c_int = -1;

    pub type ungetc_t = u16;

    bitflags! {
        #[repr(transparent)]
        pub struct FileFlags: u8 {
            /// OK to read
            const SRD = 0x01;
            /// OK to write
            const SWR = 0x02;
            /// found error
            const SERR = 0x04;
            /// found EOF
            const SEOF = 0x08;
            /// struct is __file_close
            const SCLOSE = 0x10;
            /// struct is __file_ext
            const SEXT = 0x20;
            /// struct is __file_bufio
            const SBUF = 0x40;
            /// wchar output mode
            const SWIDE = 0x80;
        }
    }

    #[repr(C)]
    pub struct FILE {
        /// ungetc buffer
        pub unget: ungetc_t,
        /// flags
        pub flags: FileFlags,
        /// function to write one char to device
        pub put: Option<unsafe extern "C" fn(c_char, *mut FILE) -> i32>,
        /// function to read one char from device
        pub get: Option<unsafe extern "C" fn(*mut FILE) -> i32>,
        /// function to flush output to device
        pub flush: Option<unsafe extern "C" fn(*mut FILE) -> i32>,
    }
}

// Separate module prevents name collisions
mod stdio_globals {
    use super::{picolibc, STDIO};

    #[repr(transparent)]
    struct FilePtr(pub *mut picolibc::FILE);
    unsafe impl Sync for FilePtr {} // presumably handled by libc

    #[unsafe(no_mangle)]
    static stdin: FilePtr = FilePtr(&raw mut STDIO);
    #[unsafe(no_mangle)]
    static stdout: FilePtr = FilePtr(&raw mut STDIO);
    #[unsafe(no_mangle)]
    static stderr: FilePtr = FilePtr(&raw mut STDIO);
}

static mut STDIO: picolibc::FILE = picolibc::FILE {
    unget: 0,
    // make this stream writable and readable because we use it for both
    // stdout and stdin
    flags: picolibc::FileFlags::SRD.union(picolibc::FileFlags::SWR),
    put: Some(serial_putc),
    get: Some(serial_getc),
    flush: Some(serial_flush),
};

unsafe extern "C" fn serial_putc(c: c_char, _file: *mut picolibc::FILE) -> c_int {
    let Some(mut stdout) = stdout().try_lock() else {
        return picolibc::EOF;
    };

    if stdout.write_all(&[c]).is_err() {
        return picolibc::EOF;
    }

    c_int::from(c)
}

unsafe extern "C" fn serial_getc(_file: *mut picolibc::FILE) -> c_int {
    let Some(mut stdin) = stdin().try_lock() else {
        return picolibc::EOF;
    };

    let mut buf = [0; 1];
    if stdin.read_exact(&mut buf).is_err() {
        return picolibc::EOF;
    }

    c_int::from(buf[0])
}

unsafe extern "C" fn serial_flush(_file: *mut picolibc::FILE) -> c_int {
    let Some(mut stdout) = stdout().try_lock() else {
        return picolibc::EOF;
    };

    if stdout.flush().is_err() {
        return picolibc::EOF;
    }

    0 // success
}
