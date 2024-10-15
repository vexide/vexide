//! V5 Electromagnet

use core::time::Duration;

use vex_sdk::{
    vexDeviceMagnetCurrentGet, vexDeviceMagnetPowerGet, vexDeviceMagnetPowerSet,
    vexDeviceMagnetStatusGet, vexDeviceMagnetTemperatureGet, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

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
    pub const MAX_POWER_DURATION: Duration = Duration::from_secs(2);
    pub const MAX_POWER: f64 = 1.0;

    /// Create a new electromagnet from a smart port index.
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
        }
    }

    pub fn set_power(&mut self, power: f64, duration: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe {
            vexDeviceMagnetPowerSet(self.device, (power * 100.0) as _, duration.as_millis() as _);
        }

        Ok(())
    }

    pub fn power(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok((unsafe { vexDeviceMagnetPowerGet(self.device) } as f64) / 100.0)
    }

    pub fn current(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceMagnetCurrentGet(self.device) })
    }

    pub fn temperature(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceMagnetTemperatureGet(self.device) })
    }

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
