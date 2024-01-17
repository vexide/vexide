//! ADI (Triport) devices on the Vex V5.

use pros_sys::{adi_port_config_e_t, PROS_ERR};
use snafu::Snafu;

use crate::error::{bail_on, map_errno, PortError};

pub mod analog;
pub mod digital;
pub mod encoder;
pub mod gyro;
pub mod motor;
pub mod potentiometer;
pub mod ultrasonic;

pub use analog::{AdiAnalogIn, AdiAnalogOut};
pub use digital::{AdiDigitalIn, AdiDigitalOut};
pub use encoder::AdiEncoder;
pub use gyro::AdiGyro;
pub use motor::AdiMotor;
pub use potentiometer::AdiPotentiometer;
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
    /// the same hardware device. Prefer using [`Peripherals`] to register devices if possible.
    pub unsafe fn new(index: u8, expander_index: Option<u8>) -> Self {
        Self {
            index,
            expander_index,
        }
    }

    /// Get the index of the port (port number).
    ///
    /// Ports are indexed starting from 1.
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Get the index of this port's associated [`AdiExpander`] smart port, or `None` if this port is not
    /// associated with an expander.
    pub fn expander_index(&self) -> Option<u8> {
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
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::ext_adi::ext_adi_port_get_config(self.internal_expander_index(), self.index())
        })
        .try_into()?)
    }
}

/// Common functionality for a ADI (three-wire) devices.
pub trait AdiDevice {
    type PortIndexOutput;

    /// Get the index of the [`AdiPort`] this device is registered on.
    ///
    /// Ports are indexed starting from 1.
    fn port_index(&self) -> Self::PortIndexOutput;

    /// Get the index of the [`AdiPort`] this device is registered on.
    ///
    /// Ports are indexed starting from 1.
    fn expander_port_index(&self) -> Option<u8>;

    /// Get the variant of [`SmartDeviceType`] that this device is associated with.
    fn device_type(&self) -> AdiDeviceType;
}

/// Represents a possible type of device that can be registered on a [`AdiPort`].
#[repr(i32)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AdiDeviceType {
    AnalogIn = pros_sys::adi::E_ADI_ANALOG_IN,
    AnalogOut = pros_sys::adi::E_ADI_ANALOG_OUT,
    DigitalIn = pros_sys::adi::E_ADI_DIGITAL_IN,
    DigitalOut = pros_sys::adi::E_ADI_DIGITAL_OUT,

    LegacyGyro = pros_sys::adi::E_ADI_LEGACY_GYRO,

    LegacyServo = pros_sys::adi::E_ADI_LEGACY_SERVO,
    LegacyPwm = pros_sys::adi::E_ADI_LEGACY_PWM,

    LegacyEncoder = pros_sys::E_ADI_LEGACY_ENCODER,
    LegacyUltrasonic = pros_sys::E_ADI_LEGACY_ULTRASONIC,
}

impl TryFrom<adi_port_config_e_t> for AdiDeviceType {
    type Error = AdiError;

    fn try_from(value: adi_port_config_e_t) -> Result<Self, Self::Error> {
        match value {
            pros_sys::E_ADI_ANALOG_IN => Ok(AdiDeviceType::AnalogIn),
            pros_sys::E_ADI_ANALOG_OUT => Ok(AdiDeviceType::AnalogOut),
            pros_sys::E_ADI_DIGITAL_IN => Ok(AdiDeviceType::DigitalIn),
            pros_sys::E_ADI_DIGITAL_OUT => Ok(AdiDeviceType::DigitalOut),

            pros_sys::E_ADI_LEGACY_GYRO => Ok(AdiDeviceType::LegacyGyro),

            pros_sys::E_ADI_LEGACY_SERVO => Ok(AdiDeviceType::LegacyServo),
            pros_sys::E_ADI_LEGACY_PWM => Ok(AdiDeviceType::LegacyPwm),

            pros_sys::E_ADI_LEGACY_ENCODER => Ok(AdiDeviceType::LegacyEncoder),
            pros_sys::E_ADI_LEGACY_ULTRASONIC => Ok(AdiDeviceType::LegacyUltrasonic),

            _ => Err(AdiError::InvalidConfigType),
        }
    }
}

impl From<AdiDeviceType> for adi_port_config_e_t {
    fn from(value: AdiDeviceType) -> Self {
        value as _
    }
}

#[derive(Debug, Snafu)]
pub enum AdiError {
    #[snafu(display("Another resource is currently trying to access the ADI."))]
    AlreadyInUse,

    #[snafu(display(
        "The port specified has been reconfigured or is not configured for digital input."
    ))]
    DigitalInputNotConfigured,

    #[snafu(display(
        "The port type specified is invalid, and cannot be used to configure a port."
    ))]
    InvalidConfigType,

    #[snafu(display("The port has already been configured."))]
    AlreadyConfigured,

    #[snafu(display("The port specified is invalid."))]
    InvalidPort,

    #[snafu(display("ADI devices may only be initialized from one expander port."))]
    ExpanderPortMismatch,

    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    AdiError {
        EACCES => Self::AlreadyInUse,
        EADDRINUSE => Self::DigitalInputNotConfigured,
    }
    inherit PortError;
}
