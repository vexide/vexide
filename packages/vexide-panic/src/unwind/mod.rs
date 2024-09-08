//! Safe wrapper around the low-level HP part of libunwind

pub mod sys;

use core::{cell::RefCell, mem::MaybeUninit};

use snafu::Snafu;
use sys::*;

/// An error that can occur during unwinding.
#[derive(Debug, Snafu)]
pub enum UnwindError {
    /// Unspecified/general error.
    Unspecified,
    /// Out of memory
    NoMemory,
    /// Invalid register number
    BadRegisterNumber,
    /// Attempt to write to a read-only register
    WriteToReadOnlyRegister,
    /// Stop unwinding
    StopUnwinding,
    /// Invalid instruction pointer
    InvalidIP,
    /// Bad frame
    BadFrame,
    /// Unsupported operation or bad value
    BadValue,
    /// Unwind info has unsupported version
    BadVersion,
    /// No unwind info found
    NoInfo,
    /// An error with an unknown error code occured
    #[snafu(display("libunwind error {code}"))]
    Unknown {
        /// The error's code
        code: uw_error_t,
    },
}

impl UnwindError {
    /// Creates a `Result` that is `Ok` if the error code represents a success
    /// and `Err` if it represents an error.
    pub const fn from_code(code: uw_error_t) -> Result<(), UnwindError> {
        if code == error::UNW_ESUCCESS {
            Ok(())
        } else {
            Err(match code {
                error::UNW_EUNSPEC => UnwindError::Unspecified,
                error::UNW_ENOMEM => UnwindError::NoMemory,
                error::UNW_EBADREG => UnwindError::BadRegisterNumber,
                error::UNW_EREADONLYREG => UnwindError::WriteToReadOnlyRegister,
                error::UNW_ESTOPUNWIND => UnwindError::StopUnwinding,
                error::UNW_EINVALIDIP => UnwindError::InvalidIP,
                error::UNW_EBADFRAME => UnwindError::BadFrame,
                error::UNW_EINVAL => UnwindError::BadValue,
                error::UNW_EBADVERSION => UnwindError::BadVersion,
                error::UNW_ENOINFO => UnwindError::NoInfo,
                code => UnwindError::Unknown { code },
            })
        }
    }
}

/// Holds context about an unwind operation.
pub struct UnwindContext {
    inner: unw_context_t,
}

impl UnwindContext {
    /// Creates an new unwind context for the current point of execution.
    #[inline(always)] // Inlining keeps this function from appearing in backtraces
    pub fn new() -> Result<Self, UnwindError> {
        let mut inner = MaybeUninit::<unw_context_t>::uninit();
        // SAFETY: `unw_getcontext` initializes the context struct.
        let inner = unsafe {
            let code = unw_getcontext(inner.as_mut_ptr());
            UnwindError::from_code(code)?;
            inner.assume_init()
        };
        Ok(Self { inner })
    }

    /// Returns the underlying libunwind object.
    pub fn as_mut_ptr(&mut self) -> *mut unw_context_t {
        &mut self.inner
    }
}

/// Allows access to information about stack frames.
pub struct UnwindCursor<'a> {
    inner: RefCell<unw_cursor_t>,
    lifetime: core::marker::PhantomData<&'a mut UnwindContext>,
}

impl<'a> UnwindCursor<'a> {
    /// Creates an unwind cursor for the given context.
    pub fn new(context: &'a mut UnwindContext) -> Result<Self, UnwindError> {
        let mut cursor = MaybeUninit::<unw_cursor_t>::uninit();
        // SAFETY: `unw_init_local` initializes the cursor struct.
        let cursor = unsafe {
            let code = unw_init_local(cursor.as_mut_ptr(), context.as_mut_ptr());
            UnwindError::from_code(code)?;
            cursor.assume_init()
        };
        Ok(Self {
            inner: RefCell::new(cursor),
            lifetime: core::marker::PhantomData,
        })
    }

    /// Steps to the next frame of the unwind cursor.
    ///
    /// Returns true if was another frame to step to or false
    /// if the cursor has reached the end.
    pub fn step(&mut self) -> bool {
        let code = unsafe { unw_step(&mut *self.inner.borrow_mut()) };
        code == UNW_STEP_SUCCESS
    }

    /// Returns the value of the given register for the current frame.
    pub fn get_register(&self, register: unw_regnum_t) -> Result<usize, UnwindError> {
        let mut reg_value = 0;
        let code = unsafe { unw_get_reg(&mut *self.inner.borrow_mut(), register, &mut reg_value) };
        UnwindError::from_code(code)?;
        Ok(reg_value)
    }

    /// Returns whether the current frame is a signal frame.
    pub fn is_signal_frame(&self) -> bool {
        unsafe { unw_is_signal_frame(&mut *self.inner.borrow_mut()) > 0 }
    }
}
