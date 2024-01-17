use pros_sys::{ext_adi_ultrasonic_t, PROS_ERR};

use super::{AdiError, AdiPort};
use crate::error::bail_on;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiUltrasonic {
    raw: ext_adi_ultrasonic_t,
}

impl AdiUltrasonic {
    /// Create an AdiUltrasonic, returning err `AdiError::InvalidPort` if the port is invalid.
    pub unsafe fn new(ports: (AdiPort, AdiPort)) -> Result<Self, AdiError> {
        let port_ping = ports.0;
        let port_echo = ports.1;

        if port_ping.internal_expander_index() != port_echo.internal_expander_index() {
            return Err(AdiError::ExpanderPortMismatch);
        }

        Ok(Self {
            raw: unsafe {
                bail_on!(
                    PROS_ERR,
                    pros_sys::ext_adi_ultrasonic_init(
                        port_ping.internal_expander_index(),
                        port_ping.index(),
                        port_echo.index()
                    )
                )
            },
        })
    }

    /// Gets the current ultrasonic sensor value in centimeters.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::ext_adi_ultrasonic_get(self.raw)) })
    }
}
