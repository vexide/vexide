#[derive(Debug)]
pub enum PortError {
    PortOutOfRange,
    PortCannotBeConfigured,
}

impl PortError {
    pub fn from_last_errno() -> Result<(), Self> {
        match unsafe { crate::errno::ERRNO.get() as u32 } {
            pros_sys::ENXIO => Err(Self::PortOutOfRange),
            pros_sys::ENODEV => Err(Self::PortCannotBeConfigured),
            _ => Ok(()),
        } 
    }
}