use core::ops::{Deref, DerefMut};

use pros_sys::{
    adi_port_config_e_t, E_ADI_ANALOG_IN, E_ADI_ANALOG_OUT, E_ADI_DIGITAL_IN, E_ADI_DIGITAL_OUT,
    E_ADI_LEGACY_ACCELEROMETER, E_ADI_LEGACY_ENCODER, E_ADI_LEGACY_GYRO, E_ADI_LEGACY_PWM,
    E_ADI_LEGACY_SERVO, E_ADI_LEGACY_ULTRASONIC, PROS_ERR,
};

use crate::{
    adi::{AdiError, AdiSlot},
    error::bail_on,
};

#[repr(i32)]
pub enum AdiPortType {
    AnalogIn = E_ADI_ANALOG_IN,
    AnalogOut = E_ADI_ANALOG_OUT,
    DigitalIn = E_ADI_DIGITAL_IN,
    DigitalOut = E_ADI_DIGITAL_OUT,

    LegacyGyro = E_ADI_LEGACY_GYRO,
    LegacyAccelerometer = E_ADI_LEGACY_ACCELEROMETER,

    LegacyServo = E_ADI_LEGACY_SERVO,
    LegacyPwm = E_ADI_LEGACY_PWM,

    LegacyEncoder = E_ADI_LEGACY_ENCODER,
    LegacyUltrasonic = E_ADI_LEGACY_ULTRASONIC,
}

impl TryFrom<adi_port_config_e_t> for AdiPortType {
    type Error = AdiError;

    fn try_from(value: adi_port_config_e_t) -> Result<Self, Self::Error> {
        match value {
            E_ADI_ANALOG_IN => Ok(AdiPortType::AnalogIn),
            E_ADI_ANALOG_OUT => Ok(AdiPortType::AnalogOut),
            E_ADI_DIGITAL_IN => Ok(AdiPortType::DigitalIn),
            E_ADI_DIGITAL_OUT => Ok(AdiPortType::DigitalOut),

            E_ADI_LEGACY_GYRO => Ok(AdiPortType::LegacyGyro),
            E_ADI_LEGACY_ACCELEROMETER => Ok(AdiPortType::LegacyAccelerometer),

            E_ADI_LEGACY_SERVO => Ok(AdiPortType::LegacyServo),
            E_ADI_LEGACY_PWM => Ok(AdiPortType::LegacyPwm),

            E_ADI_LEGACY_ENCODER => Ok(AdiPortType::LegacyEncoder),
            E_ADI_LEGACY_ULTRASONIC => Ok(AdiPortType::LegacyUltrasonic),

            _ => Err(AdiError::InvalidConfigType),
        }
    }
}

impl From<AdiPortType> for adi_port_config_e_t {
    fn from(value: AdiPortType) -> Self {
        value as _
    }
}

pub struct AdiPort(u8);

impl AdiPort {
    // Create an ADI port.
    pub fn new(slot: AdiSlot) -> Self {
        let port = slot as u8;
        Self(port)
    }

    /// Sets the value for the given ADI port
    ///
    /// This only works on ports configured as outputs, and the behavior will change depending on the configuration of the port.
    pub fn set_value(&mut self, value: i32) -> Result<i32, AdiError> {
        Ok(bail_on! {
            PROS_ERR,
            unsafe { pros_sys::adi_port_set_value(self.0, value) }
        })
    }

    /// Gets the current ultrasonic sensor value in centimeters.
    ///
    /// If no object was found, zero is returned. If the ultrasonic sensor was never started, the return value is PROS_ERR. Round and fluffy objects can cause inaccurate values to be returned.
    pub fn value(&self) -> Result<i32, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_port_get_value(self.0)) })
    }

    /// Configures an ADI port to act as a given sensor type.
    pub fn configure(&mut self, config: AdiPortType) -> Result<i32, AdiError> {
        Ok(bail_on! {
            PROS_ERR,
            unsafe { pros_sys::adi_port_set_config(self.0, config as _) }
        })
    }

    /// Returns the configuration for the given ADI port.
    pub fn config(&self) -> Result<AdiPortType, AdiError> {
        Ok(unsafe { bail_on!(PROS_ERR, pros_sys::adi_port_get_config(self.0)) }.try_into()?)
    }
}

impl Deref for AdiPort {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AdiPort {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
