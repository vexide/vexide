pub trait FromErrno {
    fn from_last_errno() -> Result<(), Self> where Self: Sized;
}

#[derive(Debug)]
pub enum PortError {
    PortOutOfRange,
    PortCannotBeConfigured,
}
impl core::error::Error for PortError {}

impl core::fmt::Display for PortError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", match self {
            Self::PortOutOfRange => "The port you specified is outside of the allowed range!",
            Self::PortCannotBeConfigured => "The port you specified couldn't be configured as what you specified.\n Is something else plugged in?",
        })
    }
}

impl FromErrno for PortError {
    fn from_last_errno() -> Result<(), Self> {
        match unsafe { crate::errno::ERRNO.get() as u32 } {
            pros_sys::ENXIO => Err(Self::PortOutOfRange),
            pros_sys::ENODEV => Err(Self::PortCannotBeConfigured),
            _ => Ok(()),
        } 
    }
}