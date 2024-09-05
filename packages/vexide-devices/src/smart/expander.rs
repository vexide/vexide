//! ADI expander module support.
//!
//! The ADI expander API is similar to that of [`Peripherals`](crate::peripherals::Peripherals).
//! A main difference between the two is that ADI expanders can be created safely without returning an option.
//! This is because they require a [`SmartPort`] to be created which can only be created without either peripherals struct unsafely.

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::adi::AdiPort;

/// Represents an ADI expander module plugged into a smart port.
///
/// ADI Expanders allow a smart port to be used as an "adapter" for eight additional ADI slots
/// if all onboard [`AdiPort`]s are used.
///
/// This struct gives access to [`AdiPort`]s similarly to how [`Peripherals`](crate::peripherals::Peripherals) works. Ports may
/// be partially moved out of this struct to create devices.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiExpander {
    /// ADI port A on the expander.
    pub adi_a: AdiPort,
    /// ADI port B on the expander.
    pub adi_b: AdiPort,
    /// ADI Port C on the expander.
    pub adi_c: AdiPort,
    /// ADI Port D on the expander.
    pub adi_d: AdiPort,
    /// ADI Port E on the expander.
    pub adi_e: AdiPort,
    /// ADI Port F on the expander.
    pub adi_f: AdiPort,
    /// ADI Port G on the expander.
    pub adi_g: AdiPort,
    /// ADI Port H on the expander.
    pub adi_h: AdiPort,

    port: SmartPort,
}

impl AdiExpander {
    /// Create a new expander from a smart port index.
    pub const fn new(port: SmartPort) -> Self {
        unsafe {
            Self {
                adi_a: AdiPort::new(1, Some(port.number())),
                adi_b: AdiPort::new(2, Some(port.number())),
                adi_c: AdiPort::new(3, Some(port.number())),
                adi_d: AdiPort::new(4, Some(port.number())),
                adi_e: AdiPort::new(5, Some(port.number())),
                adi_f: AdiPort::new(6, Some(port.number())),
                adi_g: AdiPort::new(7, Some(port.number())),
                adi_h: AdiPort::new(8, Some(port.number())),
                port,
            }
        }
    }
}

impl SmartDevice for AdiExpander {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Adi
    }
}
