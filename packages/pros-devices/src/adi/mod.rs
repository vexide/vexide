//! ADI (Triport) devices on the Vex V5.

use pros_core::{bail_on, error::PortError, map_errno};
use pros_sys::{adi_port_config_e_t, E_ADI_ERR, PROS_ERR};
use snafu::Snafu;

//TODO: much more in depth module documentation for device modules as well as this module.
pub mod analog;
pub mod digital;
pub mod pwm;

pub mod encoder;
pub mod gyro;
pub mod linetracker;
pub mod motor;
pub mod potentiometer;
pub mod solenoid;
pub mod switch;
pub mod ultrasonic;

pub use analog::AdiAnalogIn;
pub use digital::{AdiDigitalIn, AdiDigitalOut};
pub use encoder::AdiEncoder;
pub use gyro::AdiGyro;
pub use linetracker::AdiLineTracker;
pub use motor::AdiMotor;
pub use potentiometer::AdiPotentiometer;
pub use solenoid::AdiSolenoid;
pub use ultrasonic::AdiUltrasonic;

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

    /// Get the index of this port's associated [`AdiExpander`] smart port, or `pros_sys::adi::INTERNAL_ADI_PORT`
    /// if this port is not associated with an expander.
    pub(crate) fn internal_expander_index(&self) -> u8 {
        self.expander_index
            .unwrap_or(pros_sys::adi::INTERNAL_ADI_PORT as u8)
    }

    /// Get the type of device this port is currently configured as.
    pub fn configured_type(&self) -> Result<AdiDeviceType, AdiError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi::ext_adi_port_get_config(self.internal_expander_index(), self.index())
        })
        .try_into()
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
#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AdiDeviceType {
    /// Generic analog input.
    AnalogIn = pros_sys::adi::E_ADI_ANALOG_IN,

    /// Generic PWM output.
    ///
    /// This is actually equivalent `pros_sys::adi::E_ADI_ANALOG_OUT`, which is a misnomer.
    /// "Analog Out" in reality outputs an 8-bit PWM value.
    PwmOut = pros_sys::adi::E_ADI_ANALOG_OUT,

    /// Generic digital input.
    DigitalIn = pros_sys::adi::E_ADI_DIGITAL_IN,

    /// Generic digital output.
    DigitalOut = pros_sys::adi::E_ADI_DIGITAL_OUT,

    /// Cortex-era yaw-rate gyroscope.
    LegacyGyro = pros_sys::adi::E_ADI_LEGACY_GYRO,

    /// Cortex-era servo motor.
    LegacyServo = pros_sys::adi::E_ADI_LEGACY_SERVO,

    /// MC29 Controller Output
    ///
    /// This differs from [`Self::PwmOut`] in that it is specifically designed for controlling
    /// legacy ADI motors. Rather than taking a u8 for output, it takes a i8 allowing negative
    /// values to be sent for controlling motors in reverse with a nicer API.
    LegacyPwm = pros_sys::adi::E_ADI_LEGACY_PWM,

    /// Cortex-era encoder.
    LegacyEncoder = pros_sys::E_ADI_LEGACY_ENCODER,

    /// Cortex-era ultrasonic sensor.
    LegacyUltrasonic = pros_sys::E_ADI_LEGACY_ULTRASONIC,
}

impl TryFrom<adi_port_config_e_t> for AdiDeviceType {
    type Error = AdiError;

    fn try_from(value: adi_port_config_e_t) -> Result<Self, Self::Error> {
        bail_on!(E_ADI_ERR, value);

        match value {
            pros_sys::E_ADI_ANALOG_IN => Ok(AdiDeviceType::AnalogIn),
            pros_sys::E_ADI_ANALOG_OUT => Ok(AdiDeviceType::PwmOut),
            pros_sys::E_ADI_DIGITAL_IN => Ok(AdiDeviceType::DigitalIn),
            pros_sys::E_ADI_DIGITAL_OUT => Ok(AdiDeviceType::DigitalOut),

            pros_sys::E_ADI_LEGACY_GYRO => Ok(AdiDeviceType::LegacyGyro),

            pros_sys::E_ADI_LEGACY_SERVO => Ok(AdiDeviceType::LegacyServo),
            pros_sys::E_ADI_LEGACY_PWM => Ok(AdiDeviceType::LegacyPwm),

            pros_sys::E_ADI_LEGACY_ENCODER => Ok(AdiDeviceType::LegacyEncoder),
            pros_sys::E_ADI_LEGACY_ULTRASONIC => Ok(AdiDeviceType::LegacyUltrasonic),

            _ => Err(AdiError::UnknownDeviceType),
        }
    }
}

impl From<AdiDeviceType> for adi_port_config_e_t {
    fn from(value: AdiDeviceType) -> Self {
        value as _
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when working with ADI devices.
pub enum AdiError {
    /// Another resource is currently trying to access the ADI.
    AlreadyInUse,

    /// PROS returned an unrecognized device type.
    UnknownDeviceType,

    /// The port specified has not been configured for the device type specified.
    PortNotConfigured,

    /// ADI devices may only be initialized from one expander port.
    ExpanderPortMismatch,

    /// A given value is not correct, or the buffer is null.
    InvalidValue,

    #[snafu(display("{source}"), context(false))]
    /// An error occurred while interacting with a port.
    Port {
        /// The source of the error
        source: PortError,
    },
}

map_errno! {
    AdiError {
        EACCES => Self::AlreadyInUse,
        EADDRINUSE => Self::PortNotConfigured,
        EINVAL => Self::InvalidValue,
    }
    inherit PortError;
}
