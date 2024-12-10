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
//! the same brain/ADI expander. The top wire must be plugged into an odd-numbered port (A, C, E, G),
//! while the bottom wire must be plugged into the port directly above the top wire (that is, B, D, F, or
//! H, respectively).
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

use snafu::{ensure, Snafu};
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
    /// ```no_run
    /// # fn make_encoder(peripherals: Peripherals) -> Result<(), EncoderError> {
    /// let encoder = AdiEncoder::new((peripherals.adi_a, peripherals.adi_b))?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - If the top and bottom ports originate from different [`AdiExpander`](crate::smart::expander::AdiExpander)s,
    ///   returns [`EncoderError::ExpanderPortMismatch`].
    /// - If the top port is not odd (A, C, E, G), returns [`EncoderError::BadTopPort`].
    /// - If the bottom port is not the next after the top port, returns [`EncoderError::BadBottomPort`].
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
    ///     let encoder = AdiEncoder::new((peripherals.adi_a, peripherals.adi_b)).expect("could not create encoder");
    ///
    ///     loop {
    ///         println!("encoder position: {:?}", encoder.position());
    ///         sleep(AdiDevice::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn new(ports: (AdiPort, AdiPort)) -> Result<Self, EncoderError> {
        let top_port = ports.0;
        let bottom_port = ports.1;

        // Port error handling - two-wire devices are a little weird with this sort of thing.
        // TODO: This could be refactored to share logic with the range finder.
        // Might be fixed through #120

        // Top and bottom must be plugged into the same ADI expander.
        ensure!(
            top_port.expander_index() != bottom_port.expander_index(),
            ExpanderPortMismatchSnafu {
                top_port_expander: top_port.expander_number(),
                bottom_port_expander: bottom_port.expander_number()
            }
        );
        // Top must be on an odd indexed port (A, C, E, G).
        ensure!(
            top_port.index() % 2 != 0,
            BadTopPortSnafu {
                port: top_port.number()
            }
        );
        // Bottom must be directly next to top on the higher port index.
        ensure!(
            bottom_port.index() == (top_port.index() + 1),
            BadBottomPortSnafu {
                top_port: top_port.number(),
                bottom_port: bottom_port.number()
            }
        );

        top_port.configure(AdiDeviceType::Encoder);

        Ok(Self {
            top_port,
            bottom_port,
        })
    }

    /// Returns the distance reading of the encoder sensor in centimeters.
    ///
    /// Round and/or fluffy objects can cause inaccurate values to be returned.
    ///
    /// # Errors
    ///
    /// If the ADI device could not be accessed, returns [`EncoderError::Port`].
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
    ///     let encoder = AdiEncoder::new((peripherals.adi_a, peripherals.adi_b)).expect("could not create encoder");
    ///
    ///     loop {
    ///         println!("encoder position: {:?}", encoder.position());
    ///         sleep(AdiDevice::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn position(&self) -> Result<Position, EncoderError> {
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
    /// If the ADI device could not be accessed, returns [`EncoderError::Port`].
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
    ///     let encoder = AdiEncoder::new((peripherals.adi_a, peripherals.adi_b)).expect("could not create encoder");
    ///
    ///     // Treat the encoder as if it were at 180 degrees.
    ///     encoder.set_position(Position::from_degrees(180)).expect("could not set position");
    /// }
    /// ```
    pub fn set_position(&self, position: Position) -> Result<(), EncoderError> {
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
    /// If the ADI device could not be accessed, returns [`EncoderError::Port`].
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
    ///     let encoder = AdiEncoder::new((peripherals.adi_a, peripherals.adi_b)).expect("could not create encoder");
    ///
    ///     // Reset the encoder position to zero.
    ///     // This doesn't really do anything in this case, but it's a good example.
    ///     encoder.reset_position().expect("could not set position");
    /// }
    /// ```
    pub fn reset_position(&mut self) -> Result<(), EncoderError> {
        self.set_position(Position::default())
    }
}

impl AdiDevice for AdiEncoder {
    type PortNumberOutput = (u8, u8);

    fn port_number(&self) -> Self::PortNumberOutput {
        (self.top_port.number(), self.bottom_port.number())
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.top_port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Encoder
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with an encoder range finder.
pub enum EncoderError {
    /// The top wire must be on an odd numbered port (A, C, E, G).
    #[snafu(display(
        "The top ADI port provided (`{}`) was not odd numbered (A, C, E, G).",
        adi_port_name(*port)
    ))]
    BadTopPort {
        /// The port number that caused the error.
        port: u8,
    },

    /// The bottom wire must be plugged in directly above the top wire.
    #[snafu(display(
        "The bottom ADI port provided (`{}`) was not directly above the top port (`{}`). Instead, it should be port `{}`.",
        adi_port_name(*bottom_port),
        adi_port_name(*top_port),
        adi_port_name(*top_port + 1),
    ))]
    BadBottomPort {
        /// The bottom port number that caused the error.
        bottom_port: u8,
        /// The top port number that caused the error.
        top_port: u8,
    },

    /// The specified top and bottom ports may not belong to different ADI expanders.
    #[snafu(display(
        "The specified top and bottom ports may not belong to different ADI expanders. Both expanders {:?} and {:?} were provided.",
        top_port_expander,
        bottom_port_expander
    ))]
    ExpanderPortMismatch {
        /// The top port's expander number.
        top_port_expander: Option<u8>,
        /// The bottom port's expander number.
        bottom_port_expander: Option<u8>,
    },

    /// Generic port related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
