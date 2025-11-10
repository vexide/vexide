//! Analog/Digital Interface (ADI) Ports & Devices
//!
//! This module provides abstractions for devices connected through VEX's Analog/Digital Interface
//! (ADI) ports, also known as "three-wire ports" or "triports".
//!
//! # Overview
//!
//! The V5 Brain features 8 three-wire connector ports on its left side that allow connecting simple
//! analog and digital devices to the brain. These commonly include VEX's legacy sensors and motors
//! that plugged into the old [Cortex microcontroller].
//!
//! ADI ports can also be found on the [`AdiExpander`] device, which grants you eight additional
//! ports at the cost of a Smart port.
//!
//! ADI ports are capable of digital input (3.3V logic), 12-bit analog input, digital output, and
//! 8-bit PWM output. Each port has a dedicated 12-bit Analog-to-Digital Converter (ADC) to allow
//! for analog sensors to send a range of values to the port. There is no DAC, making equivalent
//! analog output impossible. ADI has a max voltage of 5V.
//!
//! # Update Times
//!
//! All ADI devices are updated at a fixed interval of 10ms (100Hz), defined by
//! [`ADI_UPDATE_INTERVAL`].
//!
//! [`AdiExpander`]: crate::smart::expander::AdiExpander
//! [Cortex microcontroller]: <https://www.vexrobotics.com/276-2194.html>

use core::time::Duration;

use crate::{
    adi::{encoder::AdiEncoder, range_finder::AdiRangeFinder},
    smart::PortError,
};

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

use vex_sdk::{
    V5_AdiPortConfiguration, V5_DeviceT, vexDeviceAdiPortConfigGet, vexDeviceAdiPortConfigSet,
    vexDeviceGetByIndex,
};

use crate::smart::{SmartDeviceType, validate_port};

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
    /// If this port is not associated with an [`AdiExpander`](super::smart::AdiExpander) it should
    /// be set to `None`.
    expander_number: Option<u8>,
}

impl AdiPort {
    pub(crate) const INTERNAL_ADI_PORT_NUMBER: u8 = 22;

    /// Creates a new ADI port on the specified port number and [`AdiExpander`] port number.
    ///
    /// [`AdiExpander`]: crate::smart::expander::AdiExpander
    ///
    /// # Safety
    ///
    /// Creating new `AdiPort`s is inherently unsafe due to the possibility of constructing more
    /// than one device on the same port index allowing multiple mutable references to the same
    /// hardware device. This violates Rust's borrow checker guarantees. Prefer using
    /// [`Peripherals`](crate::peripherals::Peripherals) to register devices if possible.
    ///
    /// For more information on safely creating peripherals, see [this page](https://vexide.dev/docs/peripherals/).
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

    /// Returns the index of this port's associated
    /// [`AdiExpander`](super::smart::expander::AdiExpander) Smart Port, or `None` if this port is
    /// not associated with an expander.
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
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
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

impl<const TICKS_PER_REVOLUTION: u32> From<AdiEncoder<TICKS_PER_REVOLUTION>>
    for (AdiPort, AdiPort)
{
    fn from(device: AdiEncoder<TICKS_PER_REVOLUTION>) -> Self {
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

    /// Returns the port number of the [`SmartPort`](crate::smart::SmartPort) this device's expander
    /// is connected to, or [`None`] if the device is plugged into an onboard ADI port.
    ///
    /// Ports are numbered starting from 1.
    fn expander_port_number(&self) -> Option<u8>;

    /// Returns the variant of [`AdiDeviceType`] that this device is associated with.
    fn device_type(&self) -> AdiDeviceType;
}

/// Represents a possible type of device that can be registered on a [`AdiPort`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum AdiDeviceType {
    /// Undefined device
    ///
    /// Interestingly, this port type appears to NOT be used for devices that are unconfigured
    /// (they are configured as [`Self::AnalogIn`] by default. The use of this variant is unknown.
    Undefined,

    /// Generic digital input
    ///
    /// This corresponds to the [`AdiDigitalIn`](digital::AdiDigitalIn) device.
    DigitalIn,

    /// Generic digital output
    ///
    /// This corresponds to the [`AdiDigitalOut`](digital::AdiDigitalOut) device.
    DigitalOut,

    /// 12-bit Generic analog input
    ///
    /// This corresponds to the [`AdiAnalogIn`](analog::AdiAnalogIn) device.
    AnalogIn,

    /// 8-git generic PWM output
    ///
    /// This corresponds to the [`AdiPwmOut`](pwm::AdiPwmOut) device.
    PwmOut,

    /// Limit Switch / Bumper Switch
    Switch,

    /// V2 Bumper Switch
    SwitchV2,

    /// Cortex-era potentiometer
    ///
    /// This corresponds to the [`AdiPotentiometer`](potentiometer::AdiPotentiometer) device when
    /// configured with [`PotentiometerType::Legacy`](potentiometer::PotentiometerType::Legacy).
    Potentiometer,

    /// V2 Potentiometer
    ///
    /// This corresponds to the [`AdiPotentiometer`](potentiometer::AdiPotentiometer) device when
    /// configured with [`PotentiometerType::V2`](potentiometer::PotentiometerType::V2).
    PotentiometerV2,

    /// Cortex-era yaw-rate gyroscope
    ///
    /// This corresponds to the [`AdiGyroscope`](gyroscope::AdiGyroscope) device.
    Gyro,

    /// Cortex-era servo motor
    ///
    /// This corresponds to the [`AdiServo`](servo::AdiServo) device.
    Servo,

    /// Quadrature Encoder
    ///
    /// This corresponds to the [`AdiEncoder`] device.
    Encoder,

    /// Ultrasonic Sensor/Sonar
    ///
    /// This corresponds to the [`AdiRangeFinder`] device.
    RangeFinder,

    /// Cortex-era Line Tracker
    ///
    /// This corresponds to the [`AdiLineTracker`](line_tracker::AdiLineTracker) device.
    LineTracker,

    /// Cortex-era Light Sensor
    ///
    /// This corresponds to the [`AdiLightSensor`](light_sensor::AdiLightSensor) device.
    LightSensor,

    /// Cortex-era 3-Axis Accelerometer
    ///
    /// This corresponds to the [`AdiAccelerometer`](accelerometer::AdiAccelerometer) device.
    Accelerometer,

    /// MC29 Controller Output
    ///
    /// This corresponds to the [`AdiMotor`](motor::AdiMotor) device.
    ///
    /// This differs from [`Self::PwmOut`] in that it is specifically designed for controlling
    /// legacy ADI motors. Rather than taking a u8 for output, it takes a i8 allowing negative
    /// values to be sent for controlling motors in reverse with a nicer API.
    Motor,

    /// Slew-rate limited motor PWM output
    ///
    /// This corresponds to the [`AdiMotor`](motor::AdiMotor) device when configured with
    /// `slew: true`.
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
