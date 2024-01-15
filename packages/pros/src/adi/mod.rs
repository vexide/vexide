use core::{
    ffi::c_int,
    ops::{Deref, DerefMut},
};

pub struct AdiPort(u8);

impl AdiPort {
    /// Create an AdiPort without checking if it is valid.
    ///
    /// # Safety
    ///
    /// The port must be above 0 and below [`pros_sys::NUM_ADI_PORTS`].
    pub unsafe fn new_unchecked(port: u8) -> Self {
        Self(port)
    }
    /// Create an AdiPort, returning `None` if the port is invalid.
    pub fn try_new(port: u8) -> Option<Self> {
        if c_int::from(port) < pros_sys::NUM_ADI_PORTS {
            Some(Self(port))
        } else {
            None
        }
    }
    /// Create an AdiPort.
    ///
    /// # Panics
    ///
    /// Panics if the port is greater than or equal to [`pros_sys::NUM_ADI_PORTS`].
    pub fn new(port: u8) -> Self {
        Self::try_new(port).expect("Invalid ADI port")
    }
}

impl Deref for AdiPort {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AdiPort {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct AdiAnalogIn {
    port: AdiPort,
}

impl AdiAnalogIn {
    pub fn new(port: AdiPort) -> Self {
        Self { port }
    }
}
