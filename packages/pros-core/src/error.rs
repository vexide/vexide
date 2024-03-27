//! Helpers for dealing with errno.
//!
//! Most errors in pros-rs are created by reading the last value of ERRNO.
//! This includes the very generic [`PortError`], which is used for most hardware that gets plugged into a port on a V5 Brain.
//!
//! Most of the contents of this file are not public.

/// A result type that makes returning errors easier.
pub type Result<T = ()> = core::result::Result<T, alloc::boxed::Box<dyn core::error::Error>>;

/// Gets the value of errno and sets errno to 0.
pub fn take_errno() -> i32 {
    let err = unsafe { *pros_sys::__errno() };
    if err != 0 {
        unsafe { *pros_sys::__errno() = 0 };
    }
    err
}

/// Generate an implementation of FromErrno for the given type.
///
/// Example:
/// ```ignore
/// map_errno! {
///     GpsError {
///         EAGAIN => Self::StillCalibrating,
///     }
///     inherit PortError;
/// }
/// ```
#[macro_export]
macro_rules! map_errno {
    {
        $err_ty:ty { $($errno:pat => $err:expr),*$(,)? }
        $(inherit $base:ty;)?
    } => {
        impl $crate::error::FromErrno for $err_ty {
            fn from_errno(num: i32) -> Option<Self> {
                #[allow(unused_imports)]
                use pros_sys::error::*;
                $(
                    // if the enum we're inheriting from can handle this errno, return it.
                    if let Some(err) = <$base as $crate::error::FromErrno>::from_errno(num) {
                        return Some(err.into());
                    }
                )?
                match num {
                    $($errno => Some($err),)*
                    // this function should only be called if errno is set
                    0 => panic!("Expected error state in errno, found 0."),
                    _ => None,
                }
            }
        }
    }
}

/// If errno has an error, return early.
#[macro_export]
macro_rules! bail_errno {
    () => {{
        let errno = $crate::error::take_errno();
        if errno != 0 {
            let err = $crate::error::FromErrno::from_errno(errno)
                .unwrap_or_else(|| panic!("Unknown errno code {errno}"));
            return Err(err);
        }
    }};
}

/// Checks if the value is equal to the error state, and if it is,
/// uses the value of errno to create an error and return early.
#[macro_export]
macro_rules! bail_on {
    ($err_state:expr, $val:expr) => {{
        let val = $val;
        #[allow(clippy::cmp_null)]
        if val == $err_state {
            let errno = $crate::error::take_errno();
            let err = $crate::error::FromErrno::from_errno(errno)
                .unwrap_or_else(|| panic!("Unknown errno code {errno}"));
            return Err(err); // where are we using this in a function that doesn't return result?
        }
        val
    }};
}
use snafu::Snafu;

#[derive(Debug, Snafu)]
/// Generic erros that can take place when using ports on the V5 Brain.
pub enum PortError {
    /// No device is plugged into the port.
    Disconnected,

    /// The incorrect device type is plugged into the port.
    IncorrectDevice,
}

/// A trait for converting an errno value into an error type.
pub trait FromErrno {
    /// Consume the current `errno` and, if it contains a known error, returns Self.
    fn from_errno(num: i32) -> Option<Self>
    where
        Self: Sized;
}
