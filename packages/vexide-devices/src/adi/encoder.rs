//! ADI Shaft Encoder
//!
//! This module provides an interface to interact with three-wire encoders, which is used to measure both
//! relative position of and rotational distance traveled by a shaft.
//!
//! In addition to the [VEX Optical Shaft Encoder](https://www.vexrobotics.com/276-2156.html), this API also
//! supports custom three wire encoders with custom resolutions (TPR).
//!
//! # Hardware Overview (for the Optical Shaft Encoder)
//!
//! The Optical Shaft Encoder can be used to track distance traveled, direction of motion, or position of any rotary
//! component, such as a gripper arm or tracking wheel.
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
//! Encoders are two-wire devices that must be connected to two adjacent ports on the same brain/ADI expander.
//! One of the wires must be plugged into an odd-numbered port (A, C, E, G), while the other wire must be
//! plugged into the port directly above that wire (that is, B, D, F, or H, respectively). If the top wire is
//! plugged into the lower odd-numbered port (A, C, E, G), then *clockwise* rotation will represent a positive
//! change in position. If the bottom wire is plugged into the lower port, then *counterclockwise* rotation will
//! be positive instead.
//!
//! # Comparison to [`RotationSensor`]
//!
//! Rotation sensors and Shaft Encoders both measure the same thing (angular position), but with some important
//! differences. The largest distinction is how position is measured. Rotation sensors use hall-effect magnets
//! and know their absolute angle at any given time, including after a power cycle or loss of voltage. In contrast,
//! encoders only track their *change* in position, meaning that any changes made to the encoder while unplugged
//! will not be detected as a change in position. Rotation sensors have much higher resolution than the old
//! encoders sold by VEX at 0.088° accuracy (compared to 1° of accuracy) and can measure accurately at higher
//! speeds. Rotation sensors are also capable of slotting VEX's new high-strength shafts, while these older
//! encoders can only fit low-strength shafts.
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

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};
use crate::{adi::adi_port_name, math::Angle};

/// VEX Optical Shaft Encoder
///
/// This is a type alias to [`AdiEncoder<360>`](AdiEncoder) for simplifying the creation of
/// the [legacy VEX Optical Shaft Encoders (276-2156)]. This represents an instance of
/// [`AdiEncoder`] with a resolution of 360 TPR (ticks per revolution).
///
/// [legacy VEX Optical Shaft Encoders (276-2156)]: https://www.vexrobotics.com/276-2156.html.
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
///
/// use vexide::prelude::*;
///
/// #[vexide::main]
/// async fn main(peripherals: Peripherals) {
///     let encoder = AdiOpticalEncoder::new(peripherals.adi_a, peripherals.adi_b);
///
///     loop {
///         println!("encoder position: {:?}", encoder.position());
///         sleep(AdiOpticalEncoder::UPDATE_INTERVAL).await;
///     }
/// }
/// ```
pub type AdiOpticalEncoder = AdiEncoder<360>;

/// ADI Shaft Encoder
#[derive(Debug, Eq, PartialEq)]
pub struct AdiEncoder<const TICKS_PER_REVOLUTION: u32> {
    top_port: AdiPort,
    bottom_port: AdiPort,
}

impl<const TICKS_PER_REVOLUTION: u32> AdiEncoder<TICKS_PER_REVOLUTION> {
    /// Create a new encoder with a given TPR from a top and bottom [`AdiPort`].
    ///
    /// # Panics
    ///
    /// - If the top and bottom ports originate from different [`AdiExpander`](crate::smart::expander::AdiExpander)s.
    /// - If the ports are not directly next to each other or in an invalid position (one port is not in A, C, E, G and
    ///   the other is not in in B, D, F).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// const ENCODER_TPR: u32 = 8192; // Change to 360 if you're using the encoders sold by VEX.
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let encoder = AdiEncoder::<ENCODER_TPR>::new(peripherals.adi_a, peripherals.adi_b);
    ///
    ///     loop {
    ///         println!("encoder position: {:?}", encoder.position());
    ///         sleep(vexide::adi::ADI_UPDATE_INTERVAL).await;
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
        assert!(
            if top_number.is_multiple_of(2) {
                bottom_number == top_number - 1
            } else {
                bottom_number == top_number + 1
            },
            "Encoder ports must be placed directly next to each other and in some combination of AB, CD, EF, GH, or BA, CD, EF, HG. (Got `{}{}`)",
            adi_port_name(top_number),
            adi_port_name(bottom_number),
        );

        if top_number < bottom_number {
            top_port.configure(AdiDeviceType::Encoder);
        } else {
            bottom_port.configure(AdiDeviceType::Encoder);
        }

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
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::{math::Angle, prelude::*};
    ///
    /// const ENCODER_TPR: u32 = 8192; // Change to 360 if you're using the encoders sold by VEX.
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let encoder = AdiEncoder::<ENCODER_TPR>::new(peripherals.adi_a, peripherals.adi_b);
    ///
    ///     loop {
    ///         println!("encoder position: {:?}", encoder.position());
    ///         sleep(vexide::adi::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn position(&self) -> Result<Angle, PortError> {
        self.top_port.validate_expander()?;

        Ok(Angle::from_ticks(
            f64::from(unsafe {
                vexDeviceAdiValueGet(self.top_port.device_handle(), self.top_port.index())
            }),
            TICKS_PER_REVOLUTION,
        ))
    }

    /// Sets the current encoder position to the given position without any actual movement.
    ///
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given position.
    /// This can be useful if you want to reset the encoder position to a known value at a certain point.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::{math::Angle, prelude::*};
    ///
    /// const ENCODER_TPR: u32 = 8192; // Change to 360 if you're using the encoders sold by VEX.
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let encoder = AdiEncoder::<ENCODER_TPR>::new(peripherals.adi_a, peripherals.adi_b);
    ///
    ///     // Treat the encoder as if it were at 180 degrees.
    ///     _ = encoder.set_position(Angle::from_degrees(180.0));
    /// }
    /// ```
    pub fn set_position(&self, position: Angle) -> Result<(), PortError> {
        self.top_port.validate_expander()?;

        unsafe {
            vexDeviceAdiValueSet(
                self.top_port.device_handle(),
                self.top_port.index(),
                position.as_ticks(TICKS_PER_REVOLUTION) as i32,
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
    /// These errors are only returned if the device is plugged into an [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// const ENCODER_TPR: u32 = 8192; // Change to 360 if you're using the encoders sold by VEX.
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut encoder = AdiEncoder::<ENCODER_TPR>::new(peripherals.adi_a, peripherals.adi_b);
    ///
    ///     // Reset the encoder position to zero.
    ///     // This doesn't really do anything in this case, but it's a good example.
    ///     _ = encoder.reset_position();
    /// }
    /// ```
    pub fn reset_position(&mut self) -> Result<(), PortError> {
        self.set_position(Angle::ZERO)
    }
}

impl<const TICKS_PER_REVOLUTION: u32> AdiDevice<2> for AdiEncoder<TICKS_PER_REVOLUTION> {
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
