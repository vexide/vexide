pub(crate) fn take_errno() -> i32 {
    let err = unsafe { *pros_sys::__errno() };
    if err != 0 {
        unsafe { *pros_sys::__errno() = 0 };
    }
    err
}

/// Generate an implementation of FromErrno for the given type.
///
/// Example:
/// ```
/// map_errno!(GpsError inherits PortError as |x| Self::Port(x) {
///    ENXIO => PortOutOfRange,
///   ENODEV => PortCannotBeConfigured,
/// });
/// ```
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
pub(crate) use map_errno;

/// If errno has an error, return early.
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
pub(crate) use bail_errno;

/// Checks if the value is equal to the error state, and if it is,
/// uses the value of errno to create an error and return early.
macro_rules! bail_on {
    ($err_state:expr, $val:expr) => {{
        let val = $val;
        if val == $err_state {
            let errno = $crate::error::take_errno();
            let err = $crate::error::FromErrno::from_errno(errno)
                .unwrap_or_else(|| panic!("Unknown errno code {errno}"));
            return Err(err); // where are we using this in a function that doesn't return result?
        }
        val
    }};
}
pub(crate) use bail_on;
use snafu::Snafu;

pub trait FromErrno {
    /// Consume the current `errno` and, if it contains a known error, returns Self.
    fn from_errno(num: i32) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug, Snafu)]
pub enum PortError {
    #[snafu(display("The port you specified is outside of the allowed range!"))]
    PortOutOfRange,
    #[snafu(display(
        "The port you specified couldn't be configured as what you specified.\n Is something else plugged in?"
    ))]
    PortCannotBeConfigured,
}

map_errno!(PortError {
    ENXIO => Self::PortOutOfRange,
    ENODEV => Self::PortCannotBeConfigured,
});
