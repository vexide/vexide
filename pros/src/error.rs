pub trait FromErrno {
    fn from_last_errno() -> Result<(), Self> where Self: Sized;
}

#[derive(Debug)]
pub enum PortError {
    PortOutOfRange,
    PortCannotBeConfigured,
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