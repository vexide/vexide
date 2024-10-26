//! V5 Electromagnet
//!
//! The V5 electromagnet is a device unique to the V5 workcell kit. It is a simple
//! device that produces a magnetic field at a provided power level.

use core::time::Duration;

use vex_sdk::{
    vexDeviceMagnetCurrentGet, vexDeviceMagnetPowerGet, vexDeviceMagnetPowerSet,
    vexDeviceMagnetStatusGet, vexDeviceMagnetTemperatureGet, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

/// An electromagnet plugged into a smart port.
#[derive(Debug, Eq, PartialEq)]
pub struct Electromagnet {
    port: SmartPort,
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for Electromagnet {}
unsafe impl Sync for Electromagnet {}

impl Electromagnet {
    /// Maximum duration that the magnet can be powered for.
    pub const MAX_POWER_DURATION: Duration = Duration::from_secs(2);

    /// Creates a new electromagnet from a [`SmartPort`].
    #[must_use]
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
        }
    }

    /// Sets the power level of the magnet for a given duration.
    ///
    /// Power is expressed as a number from [-1.0, 1.0]. Larger power values will result
    /// in a stronger force of attraction from the magnet.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an electromagnet device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an electromagnet device was required but
    ///   something else was connected.
    pub fn set_power(&mut self, power: f64, duration: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe {
            vexDeviceMagnetPowerSet(self.device, (power * 100.0) as _, duration.as_millis() as _);
        }

        Ok(())
    }

    /// Returns the user-set power level as a number from [-1.0, 1.0].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an electromagnet device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an electromagnet device was required but
    ///   something else was connected.
    pub fn power(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(f64::from(unsafe { vexDeviceMagnetPowerGet(self.device) }) / 100.0)
    }

    /// Returns the magnet's electrical current in amps.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an electromagnet device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an electromagnet device was required but
    ///   something else was connected.
    pub fn current(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceMagnetCurrentGet(self.device) } / 1000.0)
    }

    /// Returns the internal temperature of the magnet in degrees celsius.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an electromagnet device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an electromagnet device was required but
    ///   something else was connected.
    pub fn temperature(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceMagnetTemperatureGet(self.device) })
    }

    /// Returns the status code of the magnet.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an electromagnet device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an electromagnet device was required but
    ///   something else was connected.
    pub fn status(&self) -> Result<u32, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceMagnetStatusGet(self.device) })
    }
}

impl SmartDevice for Electromagnet {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Electromagnet
    }
}
impl From<Electromagnet> for SmartPort {
    fn from(device: Electromagnet) -> Self {
        device.port
    }
}
