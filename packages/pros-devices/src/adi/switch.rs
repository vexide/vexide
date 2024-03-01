//! ADI Digital Switch

use pros_core::bail_on;
use pros_sys::PROS_ERR;

use super::{digital::LogicLevel, AdiDevice, AdiDeviceType, AdiDigitalIn, AdiError, AdiPort};

/// Generic digital input ADI device.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiSwitch {
    port: AdiPort,
}

impl AdiSwitch {
    /// Create a digital input from an ADI port.
    pub fn new(port: AdiPort) -> Result<Self, AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_port_set_config(
                port.internal_expander_index(),
                port.index(),
                pros_sys::E_ADI_DIGITAL_IN,
            )
        });

        Ok(Self { port })
    }

    /// Gets the current logic level of a digital switch.
    pub fn level(&self) -> Result<LogicLevel, AdiError> {
        let value = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_digital_read(self.port.internal_expander_index(), self.port.index())
        }) != 0;

        Ok(match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        })
    }

    /// Returrns `true` if the switch is currently being pressed.
    ///
    /// This is equivalent shorthand to calling `Self::level().is_high()`.
    pub fn is_pressed(&self) -> Result<bool, AdiError> {
        Ok(self.level()?.is_high())
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
                self.port.internal_expander_index(),
                self.port.index(),
            )
        }) != 0)
    }
}

impl From<AdiDigitalIn> for AdiSwitch {
    fn from(device: AdiDigitalIn) -> Self {
        Self {
            port: unsafe { AdiPort::new(device.port_index(), device.expander_port_index()) },
        }
    }
}

impl AdiDevice for AdiSwitch {
    type PortIndexOutput = u8;

    fn port_index(&self) -> Self::PortIndexOutput {
        self.port.index()
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::DigitalIn
    }
}
