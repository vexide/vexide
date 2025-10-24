//! ADI Line Tracker
//!
//! Line trackers read the difference between a black line and a white surface. They can
//! be used to follow a marked path on the ground.
//!
//! In the V5 ecosystem, line trackers can be used to determine whether a robot is on a
//! white tape line placed on the field. This can be used to determine where a robot is.
//!
//! While line trackers can be used in other applications besides line following, they
//! may not be as reliable when not used in a controlled environment (like pointed down
//! at a surface). For example, they may not be reliable when pointed upward and used
//! to detect objects because there may be infrared light sources in the environment that
//! could interfere with the sensor's readings.
//!
//! # Hardware Overview
//!
//! A line tracker consists of an analog infrared light sensor and an infrared LED.
//! It works by illuminating a surface with infrared light; the sensor then picks up
//! the reflected infrared radiation and, based on its intensity, determines the
//! reflectivity of the surface in question. White surfaces will reflect more light
//! than dark surfaces, resulting in their appearing brighter to the sensor. This
//! allows the sensor to detect a dark line on a white background, or a white line on
//! a dark background.
//!
//! The Line Tracking Sensor is an analog sensor, and it internally measures values in the
//! range of 0 to 4095 from 0-5V. Darker objects reflect less light, and are indicated by
//! higher numbers. Lighter objects reflect more light, and are indicated by lower numbers.
//!
//! Internally, the sensor is comprised of an EE-SB5 photomicrosensor manufactured
//! by Omron mounted in a red housing. The sensor has a standard sensing distance
//! of 5mm.
//!
//! More information about the sensor can be found in the [datasheet](https://omronfs.omron.com/en_US/ecb/products/pdf/en-ee_sb5.pdf).
//!
//! # Effective Range
//!
//! For best results when using the Line Tracking Sensors, it is best to mount the sensors
//! between 1/8 and 1/4 of an inch away from the surface it is measuring. It is also important
//! to keep lighting in the room consistent, so sensors' readings remain accurate.

use vex_sdk::vexDeviceAdiValueGet;

use super::{analog, AdiDevice, AdiDeviceType, AdiPort, PortError};

/// Line Tracker
#[derive(Debug, Eq, PartialEq)]
pub struct AdiLineTracker {
    port: AdiPort,
}

impl AdiLineTracker {
    /// Create a line tracker on the given [`AdiPort`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let line_tracker = AdiLineTracker::new(peripherals.adi_b);
    ///     loop {
    ///         println!(
    ///             "Reflectivity: {}%",
    ///             line_tracker
    ///                 .reflectivity()
    ///                 .expect("couldn't get reflectivity")
    ///                 * 100.0
    ///         );
    ///         sleep(vexide::adi::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::LineTracker);

        Self { port }
    }

    /// Returns the reflectivity factor measured by the sensor. Higher numbers mean
    /// a more reflective object.
    ///
    /// This is returned as a value ranging from [0.0, 1.0].
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
    ///     let line_tracker = AdiLineTracker::new(peripherals.adi_b);
    ///     loop {
    ///         println!(
    ///             "Reflectivity: {}%",
    ///             line_tracker
    ///                 .reflectivity()
    ///                 .expect("couldn't get reflectivity")
    ///                 * 100.0
    ///         );
    ///         sleep(vexide::adi::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn reflectivity(&self) -> Result<f64, PortError> {
        Ok(f64::from(analog::ADC_MAX_VALUE - self.raw_reflectivity()?)
            / f64::from(analog::ADC_MAX_VALUE))
    }

    /// Returns the 12-bit reflectivity reading of the sensor.
    ///
    /// This is a raw 12-bit value from [0, 4095] representing the voltage level from
    /// 0-5V measured by the V5 Brain's ADC.
    ///
    /// A low number (less voltage) represents a **more** reflective object.
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
    ///     let line_tracker = AdiLineTracker::new(peripherals.adi_b);
    ///     loop {
    ///         println!(
    ///             "Raw 12-bit reflectivity: {}%",
    ///             line_tracker
    ///                 .raw_reflectivity()
    ///                 .expect("couldn't get reflectivity")
    ///         );
    ///         sleep(vexide::adi::ADI_UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn raw_reflectivity(&self) -> Result<u16, PortError> {
        self.port.validate_expander()?;

        Ok(unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) } as u16)
    }
}

impl AdiDevice<1> for AdiLineTracker {
    fn port_numbers(&self) -> [u8; 1] {
        [self.port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::LineTracker
    }
}
