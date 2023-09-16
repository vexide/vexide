use crate::{
    error::{FromErrno, PortError},
    position::Position,
};

pub struct RotationSensor {
    port: u8,
    pub reversed: bool,
}

impl RotationSensor {
    pub fn new(port: u8, reversed: bool) -> Result<Self, PortError> {
        unsafe {
            pros_sys::rotation_reset_position(port);

            pros_sys::rotation_set_reversed(port, reversed);
        }
        PortError::from_last_errno()?;

        Ok(Self { port, reversed })
    }

    /// Sets the position to zero.
    pub fn zero(&mut self) {
        unsafe {
            pros_sys::rotation_reset_position(self.port);
        }
    }

    /// Sets the position.
    pub fn set_position(&mut self, position: Position) {
        unsafe {
            pros_sys::rotation_set_position(self.port, (position.into_counts() * 100) as _);
        }
    }

    /// Sets whether or not the rotation sensor should be reversed.
    pub fn set_reversed(&mut self, reversed: bool) {
        self.reversed = reversed;

        unsafe {
            pros_sys::rotation_set_reversed(self.port, reversed);
        }
    }

    //TODO: See if this is accurate enough or consider switching to get_position function.
    /// Gets the current position of the sensor.
    pub fn position(&self) -> Position {
        unsafe { Position::from_degrees(pros_sys::rotation_get_angle(self.port) as _) }
    }
}
