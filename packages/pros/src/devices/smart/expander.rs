use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::devices::adi::AdiPort;

#[derive(Debug, Eq, PartialEq)]
pub struct AdiExpander {
    pub port_a: AdiPort,
    pub port_b: AdiPort,
    pub port_c: AdiPort,
    pub port_d: AdiPort,
    pub port_e: AdiPort,
    pub port_f: AdiPort,
    pub port_g: AdiPort,
    pub port_h: AdiPort,
    port: SmartPort,
}

impl AdiExpander {
    /// Create a new expander from a smart port index.
    pub fn new(port: SmartPort) -> Self {
        unsafe {
            Self {
                port_a: AdiPort::new(1, Some(port.index())),
                port_b: AdiPort::new(2, Some(port.index())),
                port_c: AdiPort::new(3, Some(port.index())),
                port_d: AdiPort::new(4, Some(port.index())),
                port_e: AdiPort::new(5, Some(port.index())),
                port_f: AdiPort::new(6, Some(port.index())),
                port_g: AdiPort::new(7, Some(port.index())),
                port_h: AdiPort::new(8, Some(port.index())),
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
