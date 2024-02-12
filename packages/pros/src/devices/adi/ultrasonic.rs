//! ADI ultrasonic sensor.

use pros_sys::{ext_adi_ultrasonic_t, PROS_ERR};

use super::{AdiDevice, AdiDeviceType, AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
/// Adi ultrasonic sensor.
/// Requires two ports one for pinging, and one for listening for the response.
pub struct AdiUltrasonic {
    raw: ext_adi_ultrasonic_t,
    port_ping: AdiPort,
    port_echo: AdiPort,
}

impl AdiUltrasonic {
    /// Create a new ultrasonic sensor from a ping and echo [`AdiPort`].
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

    /// Get the distance reading of the ultrasonic sensor in centimeters.
    ///
    /// Round and/or fluffy objects can cause inaccurate values to be returned.
    pub fn distance(&self) -> Result<u16, AdiError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi_ultrasonic_get(self.raw)
        }) as u16)
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
