use vex_sdk::vexDeviceAdiValueSet;

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::{position::Position, PortError};

#[derive(Debug, Eq, PartialEq)]
pub struct AdiServo {
    port: AdiPort,
}

impl AdiServo {
    pub const MAX_POSITION: Position = Position::from_ticks(
        ((50.0 / 360.0) * (Position::INTERNAL_TPR as f64)) as i64,
        Position::INTERNAL_TPR,
    );

    pub const MIN_POSITION: Position = Position::from_ticks(
        ((-50.0 / 360.0) * (Position::INTERNAL_TPR as f64)) as i64,
        Position::INTERNAL_TPR,
    );

    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::Servo);

        Self { port }
    }

    pub fn set_target(&mut self, position: Position) -> Result<(), PortError> {
        self.port.validate_expander()?;

        let degrees = position.as_degrees();
        if (-50.0..50.0).contains(&degrees) {
            unsafe {
                vexDeviceAdiValueSet(
                    self.port.device_handle(),
                    self.port.index(),
                    ((degrees / Self::MAX_POSITION.as_degrees()) * 127.0) as i32,
                );
            }
        }

        Ok(())
    }
}

impl AdiDevice for AdiServo {
    type PortNumberOutput = u8;

    fn port_number(&self) -> Self::PortNumberOutput {
        self.port.number()
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Servo
    }
}
