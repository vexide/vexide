//! ADI Ports & Devices
//!
//! This module provides abstractions for devices connected through VEX's Analog/Digital Interface
//! (ADI) ports, also known as the "three-wire ports" or "triports".
//!
//! # Hardware Overview
//!
//! The V5 Brain features 8 three-wire connector ports on its left side that allow connecting
//! simple analog and digital devices to the brain. These commonly include VEX's legacy sensors
//! and motors that plugged into the old [Cortex microcontroller].
//!
//! ADI ports can also be found on the [`AdiExpander`] device, which grants you eight additional
//! ports at the cost of a Smart port.
//!
//! ADI ports are capable of digital input (3.3V logic), 12-bit analog input, digital output,
//! and 8-bit PWM output. Each port has a dedicated 12-bit Analog-to-Digital Converter (ADC)
//! to allow for analog sensors to send a range of values to the port. There is no DAC, making
//! equivalent analog output impossible. ADI has a max voltage of 5V.
//!
//! # Update Times
//!
//! All ADI devices are updated at a fixed interval of 10ms (100Hz), defined by [`ADI_UPDATE_INTERVAL`].
//!
//! [`AdiExpander`]: crate::smart::expander::AdiExpander
//! [Cortex microcontroller]: <https://www.vexrobotics.com/276-2194.html>

use core::time::Duration;

use crate::PortError;

pub mod accelerometer;
pub mod addrled;
pub mod analog;
pub mod digital;
pub mod encoder;
pub mod gyroscope;
pub mod light_sensor;
pub mod line_tracker;
pub mod motor;
pub mod potentiometer;
pub mod pwm;
pub mod range_finder;
pub mod servo;

pub use accelerometer::{AdiAccelerometer, Sensitivity};
pub use analog::AdiAnalogIn;
pub use digital::{AdiDigitalIn, AdiDigitalOut};
pub use encoder::AdiEncoder;
pub use gyroscope::AdiGyroscope;
pub use light_sensor::AdiLightSensor;
pub use line_tracker::AdiLineTracker;
pub use motor::AdiMotor;
pub use potentiometer::{AdiPotentiometer, PotentiometerType};
pub use pwm::AdiPwmOut;
pub use range_finder::AdiRangeFinder;
pub use servo::AdiServo;
use vex_sdk::{
    vexDeviceAdiPortConfigGet, vexDeviceAdiPortConfigSet, vexDeviceGetByIndex,
    V5_AdiPortConfiguration, V5_DeviceT,
};

use crate::smart::{validate_port, SmartDeviceType};

/// Update rate for all ADI devices and ports.
pub const ADI_UPDATE_INTERVAL: Duration = Duration::from_millis(10);

/// Represents an ADI (three wire) port on a V5 Brain or V5 Three Wire Expander.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiPort {
    /// The number of the port.
    ///
    /// Ports are numbered starting from 1.
    number: u8,

    /// The index of this port's associated [`AdiExpander`](super::smart::AdiExpander).
    ///
    /// If this port is not associated with an [`AdiExpander`](super::smart::AdiExpander) it should be set to `None`.
    expander_number: Option<u8>,
}

impl AdiPort {
    pub(crate) const INTERNAL_ADI_PORT_NUMBER: u8 = 22;

    /// Create a new port.
    ///
    /// # Safety
    ///
    /// Creating new [`AdiPort`]s is inherently unsafe due to the possibility of constructing
    /// more than one device on the same port index allowing multiple mutable references to
    /// the same hardware device. Prefer using [`Peripherals`](crate::peripherals::Peripherals) to register devices if possible.
    #[must_use]
    pub const unsafe fn new(number: u8, expander_number: Option<u8>) -> Self {
        Self {
            number,
            expander_number,
        }
    }

    /// Returns the number of the port.
    ///
    /// Ports are numbered starting from 1.
    #[must_use]
    pub const fn number(&self) -> u8 {
        self.number
    }

    /// Returns the index of this port's associated [`AdiExpander`](super::smart::AdiExpander) Smart Port, or `None` if this port is not
    /// associated with an expander.
    #[must_use]
    pub const fn expander_number(&self) -> Option<u8> {
        self.expander_number
    }

    pub(crate) const fn index(&self) -> u32 {
        (self.number - 1) as u32
    }

    pub(crate) fn expander_index(&self) -> u32 {
        u32::from(
            (self
                .expander_number
                .unwrap_or(Self::INTERNAL_ADI_PORT_NUMBER))
                - 1,
        )
    }

    pub(crate) fn device_handle(&self) -> V5_DeviceT {
        unsafe { vexDeviceGetByIndex(self.expander_index()) }
    }

    pub(crate) fn validate_expander(&self) -> Result<(), PortError> {
        validate_port(
            self.expander_number
                .unwrap_or(Self::INTERNAL_ADI_PORT_NUMBER),
            SmartDeviceType::Adi,
        )
    }

    /// Configures the ADI port to a specific type if it wasn't already configured.
    pub(crate) fn configure(&self, config: AdiDeviceType) {
        unsafe {
            vexDeviceAdiPortConfigSet(self.device_handle(), self.index(), config.into());
        }
    }

    /// Returns the type of device this port is currently configured as.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`PortError::IncorrectDevice`] error is returned if an ADI expander device was required but
    ///   something else was connected.
    pub fn configured_type(&self) -> Result<AdiDeviceType, PortError> {
        self.validate_expander()?;

        Ok(unsafe { vexDeviceAdiPortConfigGet(self.device_handle(), self.index()) }.into())
    }
}

impl<T: AdiDevice<1>> From<T> for AdiPort {
    fn from(device: T) -> Self {
        // SAFETY: We can do this, since we ensure that the old Smart Port was disposed of.
        // This can effectively be thought as a move out of the device's private `port` field.
        unsafe { Self::new(device.port_numbers()[0], device.expander_port_number()) }
    }
}

impl From<AdiRangeFinder> for (AdiPort, AdiPort) {
    fn from(device: AdiRangeFinder) -> Self {
        let numbers = device.port_numbers();
        let expander_number = device.expander_port_number();

        unsafe {
            (
                AdiPort::new(numbers[0], expander_number),
                AdiPort::new(numbers[1], expander_number),
            )
        }
    }
}

impl From<AdiEncoder> for (AdiPort, AdiPort) {
    fn from(device: AdiEncoder) -> Self {
        let numbers = device.port_numbers();
        let expander_number = device.expander_port_number();

        unsafe {
            (
                AdiPort::new(numbers[0], expander_number),
                AdiPort::new(numbers[1], expander_number),
            )
        }
    }
}

/// Common functionality for a ADI (three-wire) devices.
pub trait AdiDevice<const N: usize> {
    /// Update rate of ADI devices.
    const UPDATE_INTERVAL: Duration = ADI_UPDATE_INTERVAL;

    /// Returns the port numbers of the [`AdiPort`]s that this device is registered to.
    ///
    /// Ports are numbered starting from 1.
    fn port_numbers(&self) -> [u8; N];

    /// Returns the port number of the [`SmartPort`] this device's expander is connected to,
    /// or [`None`] if the device is plugged into an onboard ADI port.
    ///
    /// Ports are numbered starting from 1.
    fn expander_port_number(&self) -> Option<u8>;

    /// Returns the variant of [`AdiDeviceType`] that this device is associated with.
    fn device_type(&self) -> AdiDeviceType;
}

/// Represents a possible type of device that can be registered on a [`AdiPort`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AdiDeviceType {
    /// Undefined Device Type
    ///
    /// Interestingly, this port type appears to NOT be used
    /// for devices that are unconfigured (they are configured
    /// as [`Self::AnalogIn`] by default, since that's enum variant 0
    /// in the SDK's API).
    Undefined,

    /// Generic digital input
    DigitalIn,

    /// Generic digital output
    DigitalOut,

    /// 12-bit Generic analog input
    AnalogIn,

    /// 8-git generic PWM output
    PwmOut,

    /// Limit Switch / Bumper Switch
    Switch,

    /// V2 Bumper Switch
    SwitchV2,

    /// Cortex-era potentiometer
    Potentiometer,

    /// V2 Potentiometer
    PotentiometerV2,

    /// Cortex-era yaw-rate gyroscope
    Gyro,

    /// Cortex-era servo motor
    Servo,

    /// Quadrature Encoder
    Encoder,

    /// Ultrasonic Sensor/Sonar
    RangeFinder,

    /// Cortex-era Line Tracker
    LineTracker,

    /// Cortex-era Light Sensor
    LightSensor,

    /// Cortex-era 3-Axis Accelerometer
    Accelerometer,

    /// MC29 Controller Output
    ///
    /// This differs from [`Self::PwmOut`] in that it is specifically designed for controlling
    /// legacy ADI motors. Rather than taking a u8 for output, it takes a i8 allowing negative
    /// values to be sent for controlling motors in reverse with a nicer API.
    Motor,

    /// Slew-rate limited motor PWM output
    MotorSlew,

    /// Other device type code returned by the SDK that is currently unsupported, undocumented,
    /// or unknown.
    Unknown(V5_AdiPortConfiguration),
}

impl From<V5_AdiPortConfiguration> for AdiDeviceType {
    fn from(value: V5_AdiPortConfiguration) -> Self {
        match value {
            V5_AdiPortConfiguration::kAdiPortTypeUndefined => Self::Undefined,
            V5_AdiPortConfiguration::kAdiPortTypeDigitalIn => Self::DigitalIn,
            V5_AdiPortConfiguration::kAdiPortTypeDigitalOut => Self::DigitalOut,
            V5_AdiPortConfiguration::kAdiPortTypeAnalogIn => Self::AnalogIn,
            V5_AdiPortConfiguration::kAdiPortTypeAnalogOut => Self::PwmOut,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyButton => Self::Switch,
            V5_AdiPortConfiguration::kAdiPortTypeSmartButton => Self::SwitchV2,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyPotentiometer => Self::Potentiometer,
            V5_AdiPortConfiguration::kAdiPortTypeSmartPot => Self::PotentiometerV2,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyGyro => Self::Gyro,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyServo => Self::Servo,
            V5_AdiPortConfiguration::kAdiPortTypeQuadEncoder => Self::Encoder,
            V5_AdiPortConfiguration::kAdiPortTypeSonar => Self::RangeFinder,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyLineSensor => Self::LineTracker,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyLightSensor => Self::LightSensor,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyAccelerometer => Self::Accelerometer,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyPwm => Self::Motor,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyPwmSlew => Self::MotorSlew,
            other => Self::Unknown(other),
        }
    }
}

impl From<AdiDeviceType> for V5_AdiPortConfiguration {
    fn from(value: AdiDeviceType) -> Self {
        match value {
            AdiDeviceType::Undefined => V5_AdiPortConfiguration::kAdiPortTypeUndefined,
            AdiDeviceType::DigitalIn => Self::kAdiPortTypeDigitalIn,
            AdiDeviceType::DigitalOut => Self::kAdiPortTypeDigitalOut,
            AdiDeviceType::AnalogIn => Self::kAdiPortTypeAnalogIn,
            AdiDeviceType::PwmOut => Self::kAdiPortTypeAnalogOut,
            AdiDeviceType::Switch => Self::kAdiPortTypeLegacyButton,
            AdiDeviceType::SwitchV2 => Self::kAdiPortTypeSmartButton,
            AdiDeviceType::Potentiometer => Self::kAdiPortTypeLegacyPotentiometer,
            AdiDeviceType::PotentiometerV2 => Self::kAdiPortTypeSmartPot,
            AdiDeviceType::Gyro => Self::kAdiPortTypeLegacyGyro,
            AdiDeviceType::Servo => Self::kAdiPortTypeLegacyServo,
            AdiDeviceType::Encoder => Self::kAdiPortTypeQuadEncoder,
            AdiDeviceType::RangeFinder => Self::kAdiPortTypeSonar,
            AdiDeviceType::LineTracker => Self::kAdiPortTypeLegacyLineSensor,
            AdiDeviceType::LightSensor => Self::kAdiPortTypeLegacyLightSensor,
            AdiDeviceType::Accelerometer => Self::kAdiPortTypeLegacyAccelerometer,
            AdiDeviceType::Motor => Self::kAdiPortTypeLegacyPwm,
            AdiDeviceType::MotorSlew => Self::kAdiPortTypeLegacyPwmSlew,
            AdiDeviceType::Unknown(raw) => raw,
        }
    }
}

/// Returns the name of the specified ADI port as a character.
///
/// This function is intended to help format errors.
const fn adi_port_name(port: u8) -> char {
    match port {
        1 => 'A',
        2 => 'B',
        3 => 'C',
        4 => 'D',
        5 => 'E',
        6 => 'F',
        7 => 'G',
        8 => 'H',
        _ => '?',
    }
}
