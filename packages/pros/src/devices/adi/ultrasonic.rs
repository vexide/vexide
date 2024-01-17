use pros_sys::{ext_adi_ultrasonic_t, PROS_ERR};

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiUltrasonic {
    raw: ext_adi_ultrasonic_t,
    port_ping: AdiPort,
    port_echo: AdiPort,
}

impl AdiUltrasonic {
    /// Create an AdiUltrasonic, returning err `AdiError::InvalidPort` if the port is invalid.
    pub fn new(ports: (AdiPort, AdiPort)) -> Result<Self, AdiError> {
        let port_ping = ports.0;
        let port_echo = ports.1;

        if port_ping.internal_expander_index() != port_echo.internal_expander_index() {
            return Err(AdiError::ExpanderPortMismatch);
        }

        let raw = bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_ultrasonic_init(
                port_ping.internal_expander_index(),
                port_ping.index(),
                port_echo.index(),
            )
        });

        Ok(Self {
            raw,
            port_ping,
            port_echo,
        })
    }

    /// Gets the current ultrasonic sensor value in centimeters.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_ultrasonic_get(self.raw)
        }))
    }
}

impl AdiDevice for AdiUltrasonic {
    type PortIndexOutput = (u8, u8);

    fn port_index(&self) -> Self::PortIndexOutput {
        (self.port_ping.index(), self.port_echo.index())
    }

    fn expander_port_index(&self) -> Option<u8> {
        self.port_ping.expander_index()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::LegacyUltrasonic
    }
}
