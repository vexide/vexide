//! ADI Digital Switch

use pros_sys::PROS_ERR;

use super::{digital::LogicLevel, AdiDevice, AdiDeviceType, AdiDigitalIn, AdiError, AdiPort};
use crate::error::bail_on;

/// Generic digital input ADI device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiSwitch {
    digital_in: AdiDigitalIn,
}

impl AdiSwitch {
    /// Create a digital switch from an ADI port.
    pub fn new(port: AdiPort) -> Result<Self, AdiError> {
        Ok(Self {
            digital_in: AdiDigitalIn::new(port)?,
        })
    }

    /// Gets the current logic level of a digital switch.
    pub fn level(&self) -> Result<LogicLevel, AdiError> {
        self.digital_in.level()
    }

    /// Returrns `true` if the switch is currently being pressed.
    ///
    /// This is equivalent shorthand to calling `Self::level().is_high()`.
    pub fn is_pressed(&self) -> Result<bool, AdiError> {
        self.digital_in.is_high()
    }

    /// Returns `true` if the switch has been pressed again since the last time this
    /// function was called.
    ///
    /// # Thread Safety
    ///
    /// This function is not thread-safe.
    ///
    /// Multiple tasks polling a single button may return different results under the
    /// same circumstances, so only one task should call this function for any given
    /// switch. E.g., Task A calls this function for buttons 1 and 2. Task B may call
    /// this function for button 3, but should not for buttons 1 or 2. A typical
    /// use-case for this function is to call inside opcontrol to detect new button
    /// presses, and not in any other tasks.
    pub fn was_pressed(&mut self) -> Result<bool, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_digital_get_new_press(
                self.digital_in.expander_port_index()
                    .unwrap_or(pros_sys::adi::INTERNAL_ADI_PORT as u8),
                self.digital_in.port_index(),
            )
        }) != 0)
    }
}

impl From<AdiDigitalIn> for AdiSwitch {
    fn from(digital_in: AdiDigitalIn) -> Self {
        Self { digital_in }
    }
}

impl AdiDevice for AdiSwitch {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.digital_in.port_index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.digital_in.expander_port_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalIn
    }
}
