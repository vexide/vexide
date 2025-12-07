//! Integration with C/C++ language runtime.
//!
//! This module provides support for:
//!
//! - Standard I/O: stdout, stdin, stderr
//! - Global constructors and destructors
//! - Memory allocations (although limited to 4MiB)
//! - Process control: exit()
//! - Thread local variables
//!
//! Libc does provide a mechanism for [locking] various resources like malloc, env, arc4random, etc.
//! We don't implement it because vexide is single-threaded.
//!
//! [locking]: <https://github.com/picolibc/picolibc/blob/main/doc/locking.md>

use core::ffi::{c_char, c_int, c_void};
use std::{
    cell::Cell,
    io::{Read, Write, stdin, stdout},
    ptr,
};

unsafe extern "C" {
    /// Run global constructors.
    ///
    /// See: <https://github.com/picolibc/picolibc/blob/main/doc/init.md>
    pub unsafe fn __libc_init_array();

    /// Set the current TLS block.
    unsafe fn _set_tls(tls: *mut c_void);

    /// Set errno to ENOMEM.
    safe fn vexide_set_enomem();

    static mut __tls_base: u32;
    static mut __tbss_start: u32;
    static mut __tbss_end: u32;
}

/// Initialize C language runtime.
///
/// # Safety
///
/// This must be called before thread-local storage
/// has been accessed.
///
/// This may not be called twice.
pub unsafe fn init() {
    unsafe {
        init_tls();
        __libc_init_array();
    }
}

/// Initialize thread-local storage.
///
/// See: <https://github.com/picolibc/picolibc/blob/main/doc/tls.md>
///
/// # Safety
///
/// This may only be called before thread-local storage
/// has been accessed.
unsafe fn init_tls() {
    // Clear the .tbss (uninitialized statics) section by filling it with zeroes.
    // This is required since the compiler assumes it will be zeroed on first access.
    unsafe {
        ptr::write_bytes(
            &raw mut __tbss_start,
            0,
            (&raw mut __tbss_end).offset_from_unsigned(&raw mut __tbss_start),
        );
    }

    // Set the current TLS block to just use the template (.tdata). It's fine if the template gets
    // overwritten as this will be our only thread.
    unsafe {
        _set_tls((&raw mut __tls_base).cast());
    }
}

// These functions are used in vexide_sysrt.c to initialize C stdio.

const ERR: c_int = -1;

#[unsafe(no_mangle)]
extern "C" fn vexide_stdio_putc(ch: c_char, _file: *mut c_void) -> c_int {
    let mut stdout = stdout().lock();
    let result = stdout.write_all(&[ch as _]);

    if result.is_err() { ERR } else { ch.into() }
}

#[unsafe(no_mangle)]
extern "C" fn vexide_stdio_getc(_file: *mut c_void) -> c_int {
    let mut stdin = stdin().lock();

    let mut buf = [0];
    let result = stdin.read_exact(&mut buf);

    if result.is_err() { ERR } else { buf[0].into() }
}

#[unsafe(no_mangle)]
extern "C" fn vexide_stdio_flush(_file: *mut c_void) -> c_int {
    let mut stdout = stdout().lock();
    let result = stdout.flush();

    if result.is_err() { ERR } else { 0 }
}

// Other general purpose syscall implementations.

#[unsafe(no_mangle)]
extern "C" fn _exit(code: c_int) {
    // Note: destructors are not called for consistency with libstd exit.
    std::process::exit(code);
}

const BRK_SIZE: usize = 0x40_0000; // 4 MiB
thread_local! {
    static BRK_BASE: Cell<*mut u8> = const { Cell::new(ptr::null_mut()) };
    static BRK: Cell<*mut u8> = const { Cell::new(ptr::null_mut()) };
}

/// Move the BRK pointer by a certain number of bytes and return its
/// old value. Positive values claim memory, negative values return it.
#[unsafe(no_mangle)]
extern "C" fn sbrk(incr: isize) -> *mut c_void {
    // On first call, allocate a buffer that C programs can use.
    if BRK_BASE.get().is_null() {
        let buffer = vec![0u8; BRK_SIZE].into_boxed_slice();
        let ptr = Box::into_raw(buffer).cast();
        BRK_BASE.set(ptr);
        BRK.set(ptr);
    }

    let base = BRK_BASE.get();
    let range = base..base.wrapping_add(BRK_SIZE);

    let start = BRK.get();
    let end = start.wrapping_offset(incr);

    if !range.contains(&end) {
        vexide_set_enomem();
        return usize::MAX as *mut c_void;
    }

    BRK.set(end);
    start.cast()
}
