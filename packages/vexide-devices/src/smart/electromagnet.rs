//! V5 Electromagnet
//!
//! The V5 electromagnet is a device unique to the V5 workcell kit. It is a simple device that
//! produces a magnetic field at a provided power level. The power level does not have specific
//! units.
//!
//! # Hardware Overview
//!
//! Not much information can be found on the V5 workcell electromagnet; however, the electromagnet
//! is intended to be used to pick up V5 Workcell colored disks. We can assume that the lower bound
//! on the electromagnet's strength is the weight of a V5 Workcell colored disk. Assuming that the
//! plastic part of the disk is made of ABS plastic and the metal part is solid iron, the
//! electromagnet can lift at least ≈0.24oz based off of the CAD model files for the V5 Workcell kit
//! provided by VEX.

use core::time::Duration;

use vex_sdk::{
    V5_DeviceT, vexDeviceMagnetCurrentGet, vexDeviceMagnetPowerGet, vexDeviceMagnetPowerSet,
    vexDeviceMagnetStatusGet, vexDeviceMagnetTemperatureGet,
};

use super::{PortError, SmartDevice, SmartDeviceType, SmartPort};

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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut electromagnet = Electromagnet::new(peripherals.port_1);
    ///     // Use the electromagnet
    ///     _ = electromagnet.set_power(1.0, Electromagnet::MAX_POWER_DURATION);
    ///     _ = electromagnet.set_power(-0.2, Duration::from_secs(1));
    /// }
    /// ```
    #[must_use]
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
        }
    }

    /// Sets the power level of the magnet for a given duration.
    ///
    /// Power is expressed as a number from [-1.0, 1.0]. Larger power values will result in a
    /// stronger force of attraction from the magnet.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut electromagnet = Electromagnet::new(peripherals.port_1);
    ///     _ = electromagnet.set_power(1.0, Electromagnet::MAX_POWER_DURATION);
    /// }
    /// ```
    pub fn set_power(&mut self, power: f64, duration: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        let power = power.clamp(-1.0, 1.0);

        unsafe {
            vexDeviceMagnetPowerSet(self.device, (power * 100.0) as _, duration.as_millis() as _);
        }

        Ok(())
    }

    /// Returns the user-set power level as a number from [-1.0, 1.0].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut electromagnet = Electromagnet::new(peripherals.port_1);
    ///     _ = electromagnet.set_power(0.5, Electromagnet::MAX_POWER_DURATION);
    ///
    ///     if let Ok(power) = electromagnet.power() {
    ///         println!("Power: {}%", power * 100.0);
    ///     }
    /// }
    /// ```
    pub fn power(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(f64::from(unsafe { vexDeviceMagnetPowerGet(self.device) }) / 100.0)
    }

    /// Returns the magnet's electrical current in amps.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut electromagnet = Electromagnet::new(peripherals.port_1);
    ///     _ = electromagnet.set_power(1.0, Electromagnet::MAX_POWER_DURATION);
    ///
    ///     if let Ok(current) = electromagnet.current() {
    ///         println!("Current: {}A", current);
    ///     }
    /// }
    /// ```
    pub fn current(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceMagnetCurrentGet(self.device) } / 1000.0)
    }

    /// Returns the internal temperature of the magnet in degrees celsius.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let electromagnet = Electromagnet::new(peripherals.port_1);
    ///
    ///     if let Ok(temperature) = electromagnet.temperature() {
    ///         println!("Temperature: {}°C", temperature);
    ///     }
    /// }
    /// ```
    pub fn temperature(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceMagnetTemperatureGet(self.device) })
    }

    /// Returns the status code of the magnet.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let electromagnet = Electromagnet::new(peripherals.port_1);
    ///
    ///     if let Ok(status) = electromagnet.status() {
    ///         println!("Status: {:b}", status);
    ///     }
    /// }
    /// ```
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
