//! ADI Ultrasonic Range Finder
//!
//! The Ultrasonic Range Finder is a rangefinding device which uses ultrasonic sound to measure the
//! distance between the sensor and the object the sound is being reflected back from.
//!
//! The Ultrasonic Range Finder is also known as a sonar sensor in VEXCode terminology.
//!
//! # Hardware Overview
//!
//! The Ultrasonic Rangefinder uses sound pulses to measure distance, in a similar way to
//! how bats or submarines find their way around. By emitting an 40KHz ultrasonic pulse for 250mS
//! and timing how long it takes to hear an echo, the Ultrasonic Rangefinder can accurately
//! estimate how far away an object in front of it is.
//!
//! The equation used by the Ultrasonic Range Finder's to calculate its distance reading is
//! `d = t * 171.5` where "d" represents the distance between the sensor and the object found, "t"
//! represents the time it took for the sound wave to return to the sensor, and 171.5 is half the
//! speed of sound in `m/s`.
//!
//! # Effective Range
//!
//! The usable range of the Range Finder is between 1.5" (3.0cm) and 115" (300cm). When the sensor
//! attempts to measure an object at less than 1.5", the sound echos back too quickly for the
//! sensor to detect and much beyond 115" the intensity of the sound is too weak to detect.
//!
//! Since the Ultrasonic Rangefinder relies on sound waves, surfaces that absorb or deflect sound
//! (such as cushioned surfaces or sharp angles) will limit the operating range of the sensor.
//!
//! # Wiring
//!
//! The sensor has two 3-Wire Cables. There is a black, red, and orange "Output" cable which
//! pulses power to a 40KHz speaker; and a black, red, and yellow "Input" cable which sends a
//! signal back from its high frequency microphone receiver.
//!
//! When wiring the Ultrasonic Rangefinder to the, both wires must be plugged into adjacent ADI
//! ports. For the sensor to work properly, the "OUTPUT" wire must be in an odd-numbered slot
//! (A, C, E, G), and the "INPUT" wire must be in the higher slot next to the input wire.

use vex_sdk::vexDeviceAdiValueGet;

use super::{adi_port_name, AdiDevice, AdiDeviceType, AdiPort, PortError};

/// Range Finder
///
/// Requires two ports - one for pinging (output), and one for listening for the response (input).
/// This output port ("ping") must be indexed directly below the input ("echo") port.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiRangeFinder {
    output_port: AdiPort,
    input_port: AdiPort,
}

impl AdiRangeFinder {
    /// Create a new rangefinder sensor from an output and input [`AdiPort`].
    ///
    /// # Panics
    ///
    /// - If the top and bottom ports originate from different [`AdiExpander`](crate::smart::expander::AdiExpander)s.
    /// - If the output port is not odd (A, C, E, G).
    /// - If the input port is not the next after the output port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let range_finder = AdiRangeFinder::new(peripherals.adi_a, peripherals.adi_b);
    ///     loop {
    ///         let distance = range_finder.distance().expect("Failed to get distance");
    ///         println!("Distance: {} cm", distance);
    ///         sleep(vexide::adi::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn new(output_port: AdiPort, input_port: AdiPort) -> Self {
        let output_number = output_port.number();
        let input_number = input_port.number();

        // Input and output must be plugged into the same ADI expander.
        assert!(
            input_port.expander_index() == output_port.expander_index(),
            "The specified output and input ports belong to different ADI expanders. Both expanders {:?} and {:?} were provided.",
            output_port.expander_number(),
            input_port.expander_number(),
        );

        // Output must be on an odd indexed port (A, C, E, G).
        assert!(
            !output_number.is_multiple_of(2),
            "The output ADI port provided (`{}`) was not odd numbered (A, C, E, G).",
            adi_port_name(output_number),
        );

        // Input must be directly next to top on the higher port index.
        assert!(
            input_number == output_number + 1,
            "The input ADI port provided (`{}`) was not directly above the output port (`{}`). Instead, it should be port `{}`.",
            adi_port_name(input_number),
            adi_port_name(output_number),
            adi_port_name(output_number + 1),
        );

        output_port.configure(AdiDeviceType::RangeFinder);

        Self {
            output_port,
            input_port,
        }
    }

    /// Returns the distance reading of the rangefinder sensor in centimeters, or `None` if the sensor was unable
    /// to find an object in range.
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
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let range_finder = AdiRangeFinder::new(peripherals.adi_a, peripherals.adi_b);
    ///     loop {
    ///         match range_finder.distance().expect("Failed to get distance") {
    ///             Some(distance) => println!("Distance: {} cm", distance),
    ///             None => println!("Can't find anything in range :("),
    ///         }
    ///
    ///         sleep(vexide::adi::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn distance(&self) -> Result<Option<u16>, PortError> {
        self.output_port.validate_expander()?;

        match unsafe {
            vexDeviceAdiValueGet(self.output_port.device_handle(), self.output_port.index())
        } {
            -1 => Ok(None),
            val => Ok(Some(val as u16)),
        }
    }
}

impl AdiDevice<2> for AdiRangeFinder {
    fn port_numbers(&self) -> [u8; 2] {
        [self.output_port.number(), self.input_port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.output_port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::RangeFinder
    }
}
