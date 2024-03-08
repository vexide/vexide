//! Smart Ports & Devices
//!
//! This module provides abstractions over device access connected through VEX V5 Smart Ports. This
//! includes motors, many common sensors, vexlink, and raw serial access.
//!
//! # Hardware Overview
//!
//! The V5 brain features 21 RJ9 4p4c connector ports (known as "Smart Ports") for communicating with
//! newer V5 peripherals. Smart port devices have a variable sample rate (unlike ADI, which is limited
//! to 10ms), and can support basic data transfer over serial.
//!
//! # Smart Port Devices
//!
//! Most devices can be created with a `new` function that generally takes a port number along with other
//! device-specific parameters. All sensors are thread safe, however sensors can only be safely constructed
//! using the [`peripherals`](crate::peripherals) API.
//!
//! In cases where PROS gives the option of a blocking or non-blocking API,
//! the blocking API is used for a synchronous method and the non-blocking API is used to create a future.
//!
//! More specific info for each device is availible in their respective modules.

pub mod distance;
pub mod expander;
pub mod gps;
pub mod imu;
pub mod link;
pub mod motor;
pub mod optical;
pub mod rotation;
pub mod vision;

use core::fmt;

pub use distance::DistanceSensor;
pub use expander::AdiExpander;
pub use gps::GpsSensor;
pub use imu::InertialSensor;
pub use link::{Link, RxLink, TxLink};
pub use motor::Motor;
pub use optical::OpticalSensor;
use pros_core::{bail_on, error::PortError};
pub use rotation::RotationSensor;
pub use vision::VisionSensor;

/// Defines common functionality shared by all smart port devices.
pub trait SmartDevice {
    /// Get the index of the [`SmartPort`] this device is registered on.
    ///
    /// Ports are indexed starting from 1.
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = InertialSensor::new(peripherals.port_1)?;
    /// assert_eq!(sensor.port_index(), 1);
    /// ```
    fn port_index(&self) -> u8;

    /// Get the variant of [`SmartDeviceType`] that this device is associated with.
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = InertialSensor::new(peripherals.port_1)?;
    /// assert_eq!(sensor.device_type(), SmartDeviceType::Imu);
    /// ```
    fn device_type(&self) -> SmartDeviceType;

    /// Determine if this device type is currently connected to the [`SmartPort`]
    /// that it's registered to.
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = InertialSensor::new(peripherals.port_1)?;
    ///
    /// if sensor.port_connected() {
    ///     println!("IMU is connected!");
    /// } else {
    ///     println!("No IMU connection found.");
    /// }
    /// ```
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
    /// Creates a new smart port on a specified index.
    ///
    /// # Safety
    ///
    /// Creating new `SmartPort`s is inherently unsafe due to the possibility of constructing
    /// more than one device on the same port index allowing multiple mutable references to
    /// the same hardware device. This violates rust's borrow checked guarantees. Prefer using
    /// [`Peripherals`](crate::peripherals::Peripherals) to register devices if possible.
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a new smart port at index 1.
    /// // This is unsafe! You are responsible for ensuring that only one device registered on a
    /// // single port index.
    /// let my_port = unsafe { SmartPort::new(1) };
    /// ```
    pub const unsafe fn new(index: u8) -> Self {
        Self { index }
    }

    /// Get the index of the port (port number).
    ///
    /// Ports are indexed starting from 1.
    ///
    /// # Examples
    ///
    /// ```
    /// let my_port = unsafe { SmartPort::new(1) };
    ///
    /// assert_eq!(my_port.index(), 1);
    /// ```
    pub const fn index(&self) -> u8 {
        self.index
    }

    /// Get the type of device currently connected to this port.
    ///
    /// # Examples
    ///
    /// ```
    /// let my_port = unsafe { SmartPort::new(1) };
    ///
    /// println!("Type of device connected to port 1: {:?}", my_port.connected_type()?);
    /// ```
    pub fn connected_type(&self) -> Result<SmartDeviceType, PortError> {
        unsafe { pros_sys::apix::registry_get_plugged_type(self.index() - 1).try_into() }
    }

    /// Get the type of device this port is configured as.
    ///
    /// # Examples
    ///
    /// ```
    /// let my_port = unsafe { SmartPort::new(1) };
    /// let imu = InertialSensor::new(my_port)?;
    ///
    /// assert_eq!(my_port.configured_type()?, SmartDeviceType::Imu);
    /// ```
    pub fn configured_type(&self) -> Result<SmartDeviceType, PortError> {
        unsafe { pros_sys::apix::registry_get_bound_type(self.index() - 1).try_into() }
    }
}

/// Represents a possible type of device that can be registered on a [`SmartPort`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SmartDeviceType {
    /// No device
    None = pros_sys::apix::E_DEVICE_NONE,

    /// Smart Motor
    Motor = pros_sys::apix::E_DEVICE_MOTOR,

    /// Rotation Sensor
    Rotation = pros_sys::apix::E_DEVICE_ROTATION,

    /// Inertial Sensor
    Imu = pros_sys::apix::E_DEVICE_IMU,

    /// Distance Sensor
    Distance = pros_sys::apix::E_DEVICE_DISTANCE,

    /// Vision Sensor
    Vision = pros_sys::apix::E_DEVICE_VISION,

    /// Optical Sensor
    Optical = pros_sys::apix::E_DEVICE_OPTICAL,

    /// GPS Sensor
    Gps = pros_sys::apix::E_DEVICE_GPS,

    /// Smart Radio
    Radio = pros_sys::apix::E_DEVICE_RADIO,

    /// ADI Expander
    ///
    /// This variant is also internally to represent the brain's onboard ADI slots.
    Adi = pros_sys::apix::E_DEVICE_ADI,

    /// Generic Serial Port
    Serial = pros_sys::apix::E_DEVICE_SERIAL,
}

impl TryFrom<pros_sys::apix::v5_device_e_t> for SmartDeviceType {
    type Error = PortError;

    /// Convert a raw `pros_sys::apix::v5_device_e_t` from `pros_sys` into a [`SmartDeviceType`].
    fn try_from(value: pros_sys::apix::v5_device_e_t) -> Result<Self, Self::Error> {
        // PROS returns either -1 (WTF?!?!) or 255 which both cast to E_DEVICE_UNDEFINED
        // when setting ERRNO, which can only be ENXIO.
        //
        // <https://github.com/purduesigbots/pros/issues/623>
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
    /// Convert a [`SmartDeviceType`] into a raw `pros_sys::apix::v5_device_e_t`.
    fn from(value: SmartDeviceType) -> Self {
        value as _
    }
}

/// Represents a timestamp on a smart device's internal clock. This type offers
/// no guarantees that the device's clock is in sync with the internal clock of
/// the brain, and thus cannot be safely compared with [`pros_core::time::Instant`]s.
///
/// There is additionally no guarantee that this is in sync with other smart devices,
/// or even the same device if a disconnect occurred causing the clock to reset. As such,
/// this is effectively a newtype wrapper of `u32`.
///
/// # Precision
///
/// This type has a precision of 1 millisecond.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SmartDeviceTimestamp(pub u32);

impl fmt::Debug for SmartDeviceTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
