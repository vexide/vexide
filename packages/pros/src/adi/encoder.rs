use crate::adi::{
    AdiError,
    AdiSlot
};

use pros_sys::{
    PROS_ERR,
    adi_encoder_t
};

use crate::error::bail_on;

pub struct AdiEncoder {
    port_top: u8,
    port_bottom: u8,
    reverse: bool,
    reference: adi_encoder_t
}

impl AdiEncoder {
    /// Create an AdiEncoder without checking if it is valid.
    ///
    /// # Safety
    ///
    /// The port must be above 0 and below [`pros_sys::NUM_ADI_PORTS`].
    pub unsafe fn new_unchecked(port_top: AdiSlot, port_bottom: AdiSlot, reverse: bool) -> Self {
        Self {
            port_top: port_top as u8,
            port_bottom: port_bottom as u8,
            reverse,
            reference: pros_sys::adi_encoder_init(port_top as u8, port_bottom as u8, reverse)
        }
    }
    
    /// Create an AdiEncoder, panicking if the port is invalid.
    pub unsafe fn new_raw(port_top: AdiSlot, port_bottom: AdiSlot, reverse: bool) -> Self {
        Self::new(port_top, port_bottom, reverse).unwrap()
    }

    /// Create an AdiEncoder, returning err `AdiError::InvalidPort` if the port is invalid.
    pub unsafe fn new(port_top: AdiSlot, port_bottom: AdiSlot, reverse: bool) -> Result<Self, AdiError> {
        if {port_bottom as u8} < 1 || {port_top as u8} > {pros_sys::NUM_ADI_PORTS as u8} {
            return Err(AdiError::InvalidPort);
        }
        Ok(Self {
            port_top: port_top as u8,
            port_bottom: port_bottom as u8,
            reverse,
            reference: bail_on! {PROS_ERR, pros_sys::adi_encoder_init(port_top as u8, port_bottom as u8, reverse)}
        })
    }

    /// Resets the encoder to zero.
    pub fn reset(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_encoder_reset(self.reference)) })
    }

    /// Gets the number of ticks recorded by the encoder.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_encoder_get(self.reference)) })
    }
}