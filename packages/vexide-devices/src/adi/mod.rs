//! ADI (Triport) devices on the Vex V5.

use crate::PortError;

pub mod analog;
pub mod digital;
pub mod linetracker;
pub mod motor;
pub mod pwm;
pub mod solenoid;
pub mod ultrasonic;

pub use analog::AdiAnalogIn;
pub use digital::{AdiDigitalIn, AdiDigitalOut};
pub use linetracker::AdiLineTracker;
pub use motor::AdiMotor;
pub use solenoid::AdiSolenoid;
pub use ultrasonic::AdiUltrasonic;

use vex_sdk::{
    vexDeviceAdiPortConfigGet, vexDeviceAdiPortConfigSet, vexDeviceGetByIndex,
    V5_AdiPortConfiguration, V5_DeviceT,
};

use crate::smart::{validate_port, SmartDeviceType};

/// Represents an ADI (three wire) port on a V5 Brain or V5 Three Wire Expander.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiPort {
    /// The index of the port (port number).
    ///
    /// Ports are indexed starting from 1.
    index: u8,

    /// The index of this port's associated [`AdiExpander`].
    ///
    /// If this port is not associated with an [`AdiExpander`] it should be set to `None`.
    expander_index: Option<u8>,
}

impl AdiPort {
    pub(crate) const INTERNAL_ADI_PORT_INDEX: u8 = 22;

    /// Create a new port.
    ///
    /// # Safety
    ///
    /// Creating new `AdiPort`s is inherently unsafe due to the possibility of constructing
    /// more than one device on the same port index allowing multiple mutable references to
    /// the same hardware device. Prefer using [`Peripherals`](crate::peripherals::Peripherals) to register devices if possible.
    pub const unsafe fn new(index: u8, expander_index: Option<u8>) -> Self {
        Self {
            index,
            expander_index,
        }
    }

    /// Get the index of the port (port number).
    ///
    /// Ports are indexed starting from 1.
    pub const fn index(&self) -> u8 {
        self.index
    }

    /// Get the index of this port's associated [`AdiExpander`] smart port, or `None` if this port is not
    /// associated with an expander.
    pub const fn expander_index(&self) -> Option<u8> {
        self.expander_index
    }

    pub(crate) const fn internal_index(&self) -> u32 {
        (self.index() - 1) as u32
    }

    pub(crate) fn internal_expander_index(&self) -> u32 {
        ((self.expander_index.unwrap_or(Self::INTERNAL_ADI_PORT_INDEX)) - 1) as u32
    }

    pub(crate) fn device_handle(&self) -> V5_DeviceT {
        unsafe { vexDeviceGetByIndex(self.internal_expander_index()) }
    }

    pub(crate) fn validate_expander(&self) -> Result<(), PortError> {
        validate_port(self.internal_expander_index() as u8 + 1, SmartDeviceType::Adi)
    }

    pub(crate) fn configure(&mut self, config: AdiDeviceType) -> Result<(), PortError> {
        self.validate_expander()?;

        unsafe {
            vexDeviceAdiPortConfigSet(self.device_handle(), self.internal_index(), config.into());
        }

        Ok(())
    }

    /// Get the type of device this port is currently configured as.
    pub fn configured_type(&self) -> Result<AdiDeviceType, PortError> {
        self.validate_expander()?;

        Ok(
            unsafe { vexDeviceAdiPortConfigGet(self.device_handle(), self.internal_index()) }
                .into(),
        )
    }
}

/// Common functionality for a ADI (three-wire) devices.
pub trait AdiDevice {
    /// The type that port_index should return. This is usually `u8`, but occasionally `(u8, u8)`.
    type PortIndexOutput;

    /// Get the index of the [`AdiPort`] this device is registered on.
    ///
    /// Ports are indexed starting from 1.
    fn port_index(&self) -> Self::PortIndexOutput;

    /// Get the index of the [`AdiPort`] this device is registered on.
    ///
    /// Ports are indexed starting from 1.
    fn expander_port_index(&self) -> Option<u8>;

    /// Get the variant of [`AdiDeviceType`] that this device is associated with.
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
    PotentimeterV2,

    /// Cortex-era yaw-rate gyroscope
    Gyro,

    /// Cortex-era servo motor
    Servo,

    /// Quadrature Encoder
    Encoder,

    /// Ultrasonic Sensor/Sonar
    Ultrasonic,

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
            V5_AdiPortConfiguration::kAdiPortTypeSmartPot => Self::PotentimeterV2,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyGyro => Self::Gyro,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyServo => Self::Servo,
            V5_AdiPortConfiguration::kAdiPortTypeQuadEncoder => Self::Encoder,
            V5_AdiPortConfiguration::kAdiPortTypeSonar => Self::Ultrasonic,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyLineSensor => Self::LineTracker,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyLightSensor => Self::LightSensor,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyAccelerometer => Self::Accelerometer,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyPwm => Self::Motor,
            V5_AdiPortConfiguration::kAdiPortTypeLegacyPwmSlew => Self::MotorSlew,
            #[allow(unreachable_patterns)]
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
            AdiDeviceType::PotentimeterV2 => Self::kAdiPortTypeSmartPot,
            AdiDeviceType::Gyro => Self::kAdiPortTypeLegacyGyro,
            AdiDeviceType::Servo => Self::kAdiPortTypeLegacyServo,
            AdiDeviceType::Encoder => Self::kAdiPortTypeQuadEncoder,
            AdiDeviceType::Ultrasonic => Self::kAdiPortTypeSonar,
            AdiDeviceType::LineTracker => Self::kAdiPortTypeLegacyLineSensor,
            AdiDeviceType::LightSensor => Self::kAdiPortTypeLegacyLightSensor,
            AdiDeviceType::Accelerometer => Self::kAdiPortTypeLegacyAccelerometer,
            AdiDeviceType::Motor => Self::kAdiPortTypeLegacyPwm,
            AdiDeviceType::MotorSlew => Self::kAdiPortTypeLegacyPwmSlew,
            AdiDeviceType::Unknown(raw) => raw,
        }
    }
}
