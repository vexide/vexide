#[derive(Debug)]
pub enum PortError {
    PortOutOfRange,
    PortCannotBeConfigured,
}

impl PortError {
    pub fn from_last_errno() -> Option<Self> {
        match unsafe { crate::errno::ERRNO.get() as u32 } {
            pros_sys::ENXIO => Some(Self::PortOutOfRange),
            pros_sys::ENODEV => Some(Self::PortCannotBeConfigured),
            _ => None,
        } 
    }
}