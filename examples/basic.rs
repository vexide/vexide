#![no_main]
#![no_std]

use vexide::{devices::smart::SmartDeviceType, prelude::*};

#[derive(Clone, Copy)]
struct PortFactory(u8);

impl SmartDevice for PortFactory {
    fn port_number(&self) -> u8 {
        self.0
    }
    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::GenericSerial
    }
}

#[vexide::main]
async fn main(_peripherals: Peripherals) {
    let cloner = PortFactory(1);
    let port_a: SmartPort = cloner.into();
    let port_b: SmartPort = cloner.into();
    dbg!((port_a, port_b));
}
