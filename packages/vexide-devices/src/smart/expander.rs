//! ADI Expander
//!
//! The ADI expander (3-Wire Expander) API is very similar to that of
//! [`Peripherals`](crate::peripherals::Peripherals); however, it can only give access to ADI ports.
//! Unlike devices created from the ADI ports on the Brain, devices created through an ADI expander
//! can error if their associated expander is not connected to the Brain.
//!
//! # Hardware Overview
//!
//! The ADI expander plugs into the Brain over a Smart Port and provides an additional eight ADI
//! ports. Just like the builtin ADI ports, these ports update every 10ms.
//!
//! The ADI expander is 3 inches long and 1 inch wide, with a height of 0.8 inches. The 8 additional
//! ports are located long side across from the Smart Port on the opposite side.
//!
//! According to the [BLRS wiki](https://wiki.purduesigbots.com/vex-electronics/vex-sensors/smart-port-sensors/3-wire-expander#behavior),
//! the ADI expander is more prone to damage from electrostatic discharge than other devices.
//!
//! # Examples
//!
//! ```no_run
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main(peripherals: Peripherals) {
//!     let peripherals = Peripherals::take().unwrap();
//!
//!     let expander = AdiExpander::new(peripherals.port_1);
//!     let analog_in = AdiAnalogIn::new(expander.adi_a);
//!
//!     println!("Analog in voltage: {:?}", analog_in.voltage());
//! }
//! ```

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::adi::AdiPort;

/// An ADI expander module plugged into a Smart Port.
///
/// ADI Expanders allow a Smart Port to be used as an "adapter" for eight additional ADI slots if
/// all onboard [`AdiPort`]s are used.
///
/// This struct gives access to [`AdiPort`]s similarly to how
/// [`Peripherals`](crate::peripherals::Peripherals) works. Ports may be partially moved out of this
/// struct to create devices.
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
    /// Creates a new expander from a [`SmartPort`].
    ///
    /// An ADI expander does not return port errors itself if it is unplugged. Any disconnect
    /// handling is done by devices created from the ports on the expander.
    ///
    /// # Examples
    ///
    /// Creating an analog input from a port on the expander:
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let expander = AdiExpander::new(peripherals.port_1);
    ///     let analog_in = AdiAnalogIn::new(expander.adi_a);
    ///
    ///     println!("Analog in voltage: {:?}", analog_in.voltage());
    /// }
    /// ```
    #[must_use]
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
impl From<AdiExpander> for SmartPort {
    fn from(device: AdiExpander) -> Self {
        device.port
    }
}
