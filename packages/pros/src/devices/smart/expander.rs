use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::devices::adi::AdiPort;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiExpander {
    pub adi_1: AdiPort,
    pub adi_2: AdiPort,
    pub adi_3: AdiPort,
    pub adi_4: AdiPort,
    pub adi_5: AdiPort,
    pub adi_6: AdiPort,
    pub adi_7: AdiPort,
    pub adi_8: AdiPort,
    port: SmartPort,
}

impl AdiExpander {
    /// Create a new expander from a smart port index.
    pub fn new(port: SmartPort) -> Self {
        unsafe {
            Self {
                adi_1: AdiPort::new(1, Some(port.index())),
                adi_2: AdiPort::new(2, Some(port.index())),
                adi_3: AdiPort::new(3, Some(port.index())),
                adi_4: AdiPort::new(4, Some(port.index())),
                adi_5: AdiPort::new(5, Some(port.index())),
                adi_6: AdiPort::new(6, Some(port.index())),
                adi_7: AdiPort::new(7, Some(port.index())),
                adi_8: AdiPort::new(8, Some(port.index())),
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
