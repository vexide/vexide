use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::devices::adi::AdiPort;

/// Represents an ADI expander module plugged into a smart port.
///
/// ADI Expanders allow a smart port to be used as an "adapter" for eight additional ADI slots
/// if all onboard [`AdiPort`]s are used.
///
/// This struct gives access to [`AdiPort`]s similarly to how [`Peripherals`] works. Ports may
/// be partially moved out of this struct to create devices.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiExpander {
    pub adi_a: AdiPort,
    pub adi_b: AdiPort,
    pub adi_c: AdiPort,
    pub adi_d: AdiPort,
    pub adi_e: AdiPort,
    pub adi_f: AdiPort,
    pub adi_g: AdiPort,
    pub adi_h: AdiPort,
    port: SmartPort,
}

impl AdiExpander {
    /// Create a new expander from a smart port index.
    pub fn new(port: SmartPort) -> Self {
        unsafe {
            Self {
                adi_a: AdiPort::new(1, Some(port.index())),
                adi_b: AdiPort::new(2, Some(port.index())),
                adi_c: AdiPort::new(3, Some(port.index())),
                adi_d: AdiPort::new(4, Some(port.index())),
                adi_e: AdiPort::new(5, Some(port.index())),
                adi_f: AdiPort::new(6, Some(port.index())),
                adi_g: AdiPort::new(7, Some(port.index())),
                adi_h: AdiPort::new(8, Some(port.index())),
                port,
            }
        }
    }
}

impl SmartDevice for AdiExpander {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Adi
    }
}
