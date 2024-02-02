//! Smart Ports & Devices
//!
//! This module provides abstractions over device access connected through VEX V5 Smart Ports. This
//! includes motors, many common sensors, vexlink, and raw serial access.
//!
//! # Overview
//!
//! Most devices can be created with a `new` function that generally takes a port number along with other
//! device-specific parameters. All sensors are thread safe, however sensors can only be safely constructed
//! using the [`Peripherals`] API.
//!
//! In cases where PROS gives the option of a blocking or non-blocking API,
//! the blocking API is used for a synchronous method and the non-blocking API is used to create a future.
//!
//! More specific info for each device is availible in their respective modules.

pub mod distance;
pub mod gps;
pub mod imu;
pub mod link;
pub mod motor;
pub mod optical;
pub mod rotation;
pub mod vision;

pub use distance::DistanceSensor;
pub use gps::GpsSensor;
pub use imu::InertialSensor;
pub use link::{Link, RxLink, TxLink};
pub use motor::Motor;
pub use optical::OpticalSensor;
pub use rotation::RotationSensor;
pub use vision::VisionSensor;

use crate::{error::bail_on, prelude::PortError};

/// Common functionality for a smart port device.
pub trait SmartDevice {
    /// Get the index of the [`SmartPort`] this device is registered on.
    ///
    /// Ports are indexed starting from 1.
    fn port_index(&self) -> u8;

    /// Get the variant of [`SmartDeviceType`] that this device is associated with.
    fn device_type(&self) -> SmartDeviceType;

    /// Determine if this device type is currently connected to the [`SmartPort`]
    /// that it's registered to.
    fn port_connected(&self) -> bool {
        let plugged_type_result: Result<SmartDeviceType, _> =
            unsafe { pros_sys::apix::registry_get_plugged_type(self.port_index() - 1).try_into() };

        if let Ok(plugged_type) = plugged_type_result {
            plugged_type == self.device_type()
        } else {
            false
        }
    }
}

/// Represents a smart port on a V5 Brain
#[derive(Debug, Eq, PartialEq)]
pub struct SmartPort {
    /// The index of the port (port number).
    ///
    /// Ports are indexed starting from 1.
    index: u8,
}

impl SmartPort {
    /// Create a new port.
    ///
    /// # Safety
    ///
    /// Creating new `SmartPort`s is inherently unsafe due to the possibility of constructing
    /// more than one device on the same port index allowing multiple mutable references to
    /// the same hardware device. Prefer using [`Peripherals`] to register devices if possible.
    pub const unsafe fn new(index: u8) -> Self {
        Self { index }
    }

    /// Get the index of the port (port number).
    ///
    /// Ports are indexed starting from 1.
    pub const fn index(&self) -> u8 {
        self.index
    }

    /// Get the type of device currently connected to this port.
    pub fn connected_type(&self) -> Result<SmartDeviceType, PortError> {
        unsafe { pros_sys::apix::registry_get_plugged_type(self.index() - 1).try_into() }
    }

    /// Get the type of device this port is configured as.
    pub fn configured_type(&self) -> Result<SmartDeviceType, PortError> {
        unsafe { pros_sys::apix::registry_get_bound_type(self.index() - 1).try_into() }
    }
}

/// Represents a possible type of device that can be registered on a [`SmartPort`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SmartDeviceType {
    /// No device is connected to the port.
    None = pros_sys::apix::E_DEVICE_NONE,
    /// A motor is connected to the port.
    Motor = pros_sys::apix::E_DEVICE_MOTOR,
    /// A rotation sensor is connected to the port.
    Rotation = pros_sys::apix::E_DEVICE_ROTATION,
    /// An inertial sensor is connected to the port.
    Imu = pros_sys::apix::E_DEVICE_IMU,
    /// A distance sensor is connected to the port.
    Distance = pros_sys::apix::E_DEVICE_DISTANCE,
    /// A vision sensor is connected to the port.
    Vision = pros_sys::apix::E_DEVICE_VISION,
    /// An optical sensor is connected to the port.
    Optical = pros_sys::apix::E_DEVICE_OPTICAL,
    /// A GPS sensor is connected to the port.
    Gps = pros_sys::apix::E_DEVICE_GPS,
    /// A VEXLink radio is connected to the port.
    Radio = pros_sys::apix::E_DEVICE_RADIO,
    /// An ADI expander is connected to the port.
    Adi = pros_sys::apix::E_DEVICE_ADI,
    /// A generic serial device is connected to the port.
    Serial = pros_sys::apix::E_DEVICE_SERIAL,
}

impl TryFrom<pros_sys::apix::v5_device_e_t> for SmartDeviceType {
    type Error = PortError;

    fn try_from(value: pros_sys::apix::v5_device_e_t) -> Result<Self, Self::Error> {
        // PROS returns either -1 (WTF?!?!) or 255 which both cast to E_DEVICE_UNDEFINED
        // when setting ERRNO, which can only be ENXIO.
        // https://github.com/purduesigbots/pros/issues/623
        bail_on!(pros_sys::apix::E_DEVICE_UNDEFINED, value);

        Ok(match value {
            pros_sys::apix::E_DEVICE_NONE => Self::None,
            pros_sys::apix::E_DEVICE_MOTOR => Self::Motor,
            pros_sys::apix::E_DEVICE_ROTATION => Self::Rotation,
            pros_sys::apix::E_DEVICE_IMU => Self::Imu,
            pros_sys::apix::E_DEVICE_DISTANCE => Self::Distance,
            pros_sys::apix::E_DEVICE_VISION => Self::Vision,
            pros_sys::apix::E_DEVICE_OPTICAL => Self::Optical,
            pros_sys::apix::E_DEVICE_RADIO => Self::Radio,
            pros_sys::apix::E_DEVICE_ADI => Self::Adi,
            pros_sys::apix::E_DEVICE_SERIAL => Self::Serial,
            _ => unreachable!(),
        })
    }
}

impl From<SmartDeviceType> for pros_sys::apix::v5_device_e_t {
    fn from(value: SmartDeviceType) -> Self {
        value as _
    }
}
