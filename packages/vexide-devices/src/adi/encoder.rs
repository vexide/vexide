//! ADI Optical Shaft Encoder
//!
//! This module provides an interface to interact with the VEX Optical Shaft Encoder, which is used to
//! measure both relative position of and rotational distance traveled by a shaft.
//!
//! # Hardware Overview
//!
//! The Optical Shaft Encoder can be used to track distance traveled, direction of motion, or position of
//! any rotary component, such as a gripper arm or tracking wheel.
//!
//! The encoder works by shining light onto the edge of a disk outfitted with evenly
//! spaced slits around the circumference. As the disk spins, light passes through the slits and
//! is blocked by the opaque spaces between the slits. The encoder then detects how many slits have
//! had light shine through, and in which direction the disk is spinning.
//!
//! The encoder can detect up to 1,700 pulses per second, which corresponds to 18.9 revolutions per
//! second and 1,133 rpm (revolutions per minute). Faster revolutions will therefore not be interpreted
//! exactly, potentially resulting in erroneous positional data being returned.
//!
//! ## Connecting to the V5 Brain
//!
//! The Optical Shaft Encoder is a two-wire device that must be connected to two adjacent ports on
//! the same brain/ADI expander. One of the wires must be plugged into an odd-numbered port (A, C, E, G),
//! while the other wire must be plugged into the port directly above that wire (that is, B, D, F, or
//! H, respectively). If the top wire is plugged into the lower odd-numbered port (A, C, E, G), then
//! *clockwise* rotation will represent a positive change in position. If the bottom wire is plugged into
//! the lower port, then *counterclockwise* rotation will be positive instead.
//!
//! # Comparison to [`RotationSensor`]
//!
//! Rotation sensors and Optical Shaft Encoders both measure the same thing (angular position), but
//! with some important differences. The largest distinction is how position is measured. Rotation
//! sensors use hall-effect magnets and know their absolute angle at any given time, including after
//! a power cycle or loss of voltage. In contrast, encoders only track their *change* in position,
//! meaning that any changes made to the encoder while unplugged will not be detected as a change in
//! position. Rotation sensors have much higher resolution at 0.088° accuracy and can measure accurately
//! at higher speeds. Rotation sensors are also capable of slotting VEX's new high-strength shafts, while
//! these older encoders can only fit low-strength shafts.
//!
//! |                     | [`AdiEncoder`]   | [`RotationSensor`]                 |
//! | ------------------- | ---------------- | ---------------------------------- |
//! | Port                | Two [`AdiPort`]s | One [`SmartPort`]                  |
//! | Resolution          | 360 Ticks/Rev    | 4090 Ticks/Rev                     |
//! | Measurements        | Position         | Position, Absolute Angle, Velocity |
//! | Update Rate         | 10mS             | 10mS                               |
//! | Shaft Compatibility | Low Strength     | Low Strength, High Strength        |
//!
//! [`RotationSensor`]: crate::smart::rotation::RotationSensor
//! [`SmartPort`]: crate::smart::SmartPort

use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::{adi::adi_port_name, position::Position, PortError};

/// Optical Shaft Encoder
#[derive(Debug, Eq, PartialEq)]
pub struct AdiEncoder {
    top_port: AdiPort,
    bottom_port: AdiPort,
}

impl AdiEncoder {
    /// Number of encoder ticks (unique sensor readings) per revolution for the encoder.
    pub const TICKS_PER_REVOLUTION: u32 = 360;

    /// Create a new encoder sensor from a top and bottom [`AdiPort`].
    ///
    /// # Panics
    ///
    /// - If the top and bottom ports originate from different [`AdiExpander`](crate::smart::expander::AdiExpander)s.
    /// - If the ports are not directly next to each other or in an invalid position (one port is not in A, C, E, G and
    ///   the other is not in in B, D, F).
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::{
    ///     prelude::*,
    ///     devices::adi::AdiDevice,
    /// };
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let encoder = AdiEncoder::new(peripherals.adi_a, peripherals.adi_b);
    ///
    ///     loop {
    ///         println!("encoder position: {:?}", encoder.position());
    ///         sleep(AdiDevice::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn new(top_port: AdiPort, bottom_port: AdiPort) -> Self {
        let top_number = top_port.number();
        let bottom_number = bottom_port.number();

        // Port error handling - two-wire devices are a little weird with this sort of thing.
        // TODO: This could be refactored to share logic with the range finder.
        // Might be fixed through #120

        // Top and bottom must be plugged into the same ADI expander.
        assert!(
            top_port.expander_index() == bottom_port.expander_index(),
            "The specified top and bottom ports belong to different ADI expanders. Both expanders {:?} and {:?} were provided.",
            top_port.expander_number(),
            bottom_port.expander_number(),
        );

        // Top and bottom must be some combination of (AB, CD, EF, GH) or (BA, CD, FE, HG)
        let top_is_even = top_number % 2 == 0;
        assert!(
            if top_is_even {
                bottom_number == top_number - 1
            } else {
                bottom_number == top_number + 1
            },
            "Encoder ports must be placed directly next to each other and in some combination of AB, CD, EF, GH, or BA, CD, EF, HG. (Got `{}{}`)",
            adi_port_name(top_number),
            adi_port_name(bottom_number),
        );

        top_port.configure(AdiDeviceType::Encoder);

        Self {
            top_port,
            bottom_port,
        }
    }

    /// Returns the distance reading of the encoder sensor in centimeters.
    ///
    /// Round and/or fluffy objects can cause inaccurate values to be returned.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::{
    ///     prelude::*,
    ///     devices::adi::AdiDevice,
    /// };
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let encoder = AdiEncoder::new(peripherals.adi_a, peripherals.adi_b);
    ///
    ///     loop {
    ///         println!("encoder position: {:?}", encoder.position());
    ///         sleep(AdiDevice::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn position(&self) -> Result<Position, PortError> {
        self.top_port.validate_expander()?;
        self.top_port.configure(self.device_type());

        Ok(Position::from_ticks(
            unsafe {
                i64::from(vexDeviceAdiValueGet(
                    self.top_port.device_handle(),
                    self.top_port.index(),
                ))
            },
            360,
        ))
    }

    /// Sets the current encoder position to the given position without any actual movement.
    ///
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given position.
    /// This can be useful if you want to reset the encoder position to a known value at a certain point.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::{
    ///     prelude::*,
    ///     devices::adi::AdiDevice,
    /// };
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let encoder = AdiEncoder::new(peripherals.adi_a, peripherals.adi_b);
    ///
    ///     // Treat the encoder as if it were at 180 degrees.
    ///     _ = encoder.set_position(Position::from_degrees(180));
    /// }
    /// ```
    pub fn set_position(&self, position: Position) -> Result<(), PortError> {
        self.top_port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(
                self.top_port.device_handle(),
                self.top_port.index(),
                position.as_ticks(360) as i32,
            );
        }

        Ok(())
    }

    /// Sets the current encoder position to zero.
    ///
    /// Analogous to taring or resetting the encoder so that the new position is equal
    /// to the given position.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::{
    ///     prelude::*,
    ///     devices::adi::AdiDevice,
    /// };
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let encoder = AdiEncoder::new(peripherals.adi_a, peripherals.adi_b);
    ///
    ///     // Reset the encoder position to zero.
    ///     // This doesn't really do anything in this case, but it's a good example.
    ///     _ = encoder.reset_position();
    /// }
    /// ```
    pub fn reset_position(&mut self) -> Result<(), PortError> {
        self.set_position(Position::default())
    }
}

impl AdiDevice<2> for AdiEncoder {
    type PortNumberOutput = (u8, u8);

    fn port_numbers(&self) -> [u8; 2] {
        [self.top_port.number(), self.bottom_port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.top_port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Encoder
    }
}
