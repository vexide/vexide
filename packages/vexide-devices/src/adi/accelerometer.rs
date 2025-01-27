//! ADI Accelerometer
//!
//! This module provides an interface for the LIS344ALH-based three-axis analog accelerometer.
//!
//! # Hardware Overview
//!
//! The LIS344ALH capacitive accelerometer features signal conditioning, a 1-pole low pass filter,
//! temperature compensation and a jumper switch which allows for the selection of 2 sensitivities.
//! Zero-g offset full scale span and filter cut-off are factory set and require no external devices.
//!
//! The sensor will measure acceleration in both directions along each of the 3 axis. Acceleration
//! along the X or Y axis in the direction of the silkscreened arrows will produce a larger reading,
//! while acceleration in the opposite direction will produce a smaller reading. For the Z axis,
//! upward acceleration (in the direction of the board's face) produces larger values, and downward
//! acceleration (toward the board's back) produces lower values.
//!
//! # Gravity
//!
//! Gravity is indistinguishable from upward acceleration, so the sensor will detect a constant 1.0g
//! on the vertical axis while at rest. For example, if the board is mounted horizontally, gravity will
//! effect only the Z axis. If the sensor is tilted away from the horizontal, the gravity reading on the
//! Z axis will diminish, and the readings on the other axis will change depending on the sensor's
//! mounting orientation.
//!
//! # Wiring
//!
//! Each axis on the accelerometer requires its own ADI port. This means that the accelerometer will take
//! three ADI ports if you wish to measure acceleration on all axes. You don't have to hook up all the
//! channels; you only need to connect the ones required for your application.
//!
//! The white (signal) wire of each cable goes near the 'X', 'Y', or 'Z' labels on the board. The black
//! (ground) wires go at the other end, adjacent to the 'B' label on the board. The center wire is for +5
//! volts. The sensor's mounting holes are electrically isolated from the circuit, meaning it is safe to
//! mount the device using screws on a robot.

use vex_sdk::vexDeviceAdiValueGet;

use super::{analog, AdiDevice, AdiDeviceType, AdiPort};
use crate::PortError;

/// A single axis connection to the 3-axis analog accelerometer.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiAccelerometer {
    sensitivity: Sensitivity,
    port: AdiPort,
}

impl AdiAccelerometer {
    /// Create a new accelerometer from an [`AdiPort`].
    #[must_use]
    pub fn new(port: AdiPort, sensitivity: Sensitivity) -> Self {
        port.configure(AdiDeviceType::Accelerometer);

        Self { sensitivity, port }
    }

    /// Returns the configured sensitivity of the ADI accelerometer device.
    #[must_use]
    pub const fn sensitivity(&self) -> Sensitivity {
        self.sensitivity
    }

    /// Returns the maximum acceleration measurement supported by the current [`Sensitivity`] jumper.
    #[must_use]
    pub const fn max_acceleration(&self) -> f64 {
        self.sensitivity().max_acceleration()
    }

    /// Returns the current acceleration measurement for this axis in g (~9.8 m/s/s).
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn acceleration(&self) -> Result<f64, PortError> {
        Ok(
            // Convert 0-4095 to 0-1, then scale to max accel.
            f64::from(self.raw_acceleration()?) / f64::from(analog::ADC_MAX_VALUE)
                * self.sensitivity.max_acceleration(),
        )
    }

    /// Returns the raw acceleration reading from [0, 4096]. This represents an ADC-converted
    /// analog input from 0-5V.
    ///
    /// For example, when on high sensitivity a value of `4096` would represent a reading of 6g
    /// ([`Sensitivity::HIGH_MAX_ACCELERATION`]). When on low acceleration, this same value
    /// would instead represent a 2g reading ([`Sensitivity::LOW_MAX_ACCELERATION`]).
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn raw_acceleration(&self) -> Result<u16, PortError> {
        self.port.validate_expander()?;

        Ok(unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } as u16)
    }
}

/// The jumper state of the accelerometer.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Sensitivity {
    /// 0-2g sensitivity
    Low,

    /// 0-6g sensitivity
    High,
}

impl Sensitivity {
    /// Maximum acceleration measurement when in low sensitivity mode.
    pub const LOW_MAX_ACCELERATION: f64 = 2.0;

    /// Maximum acceleration measurement when in high sensitivity mode.
    pub const HIGH_MAX_ACCELERATION: f64 = 6.0;

    /// Returns the maximum acceleration measurement (in g) for this sensitivity.
    #[must_use]
    pub const fn max_acceleration(&self) -> f64 {
        match self {
            Self::Low => Self::LOW_MAX_ACCELERATION,
            Self::High => Self::HIGH_MAX_ACCELERATION,
        }
    }
}

impl AdiDevice<1> for AdiAccelerometer {
    fn port_numbers(&self) -> [u8; 1] {
        [self.port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Accelerometer
    }
}
