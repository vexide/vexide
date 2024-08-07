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
//! More specific info for each device is availible in their respective modules.

pub mod distance;
pub mod expander;
pub mod gps;
pub mod imu;
pub mod link;
pub mod motor;
pub mod optical;
pub mod rotation;
pub mod serial;
pub mod vision;
pub mod ai_vision;

use core::fmt;

pub use distance::DistanceSensor;
pub use expander::AdiExpander;
pub use gps::GpsSensor;
pub use imu::InertialSensor;
pub use link::RadioLink;
pub use motor::Motor;
pub use optical::OpticalSensor;
pub use rotation::RotationSensor;
pub use serial::SerialPort;
use vex_sdk::{vexDeviceGetByIndex, vexDeviceGetTimestamp, V5_DeviceT, V5_DeviceType};
pub use vision::VisionSensor;

use crate::PortError;

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
    fn is_connected(&self) -> bool {
        let connected_type: SmartDeviceType =
            unsafe { *vexDeviceGetByIndex((self.port_index() - 1) as u32) }
                .device_type
                .into();

        connected_type == self.device_type()
    }

    /// Get the timestamp recorded by this device's internal clock.
    fn timestamp(&self) -> Result<SmartDeviceTimestamp, PortError> {
        Ok(SmartDeviceTimestamp(unsafe {
            vexDeviceGetTimestamp(vexDeviceGetByIndex((self.port_index() - 1) as u32))
        }))
    }

    /// Verify that the device type is currently plugged into this port, returning an appropriate
    /// [`PortError`] if not available.
    fn validate_port(&self) -> Result<(), PortError> {
        validate_port(self.port_index(), self.device_type())
    }
}

impl<T: SmartDevice> From<T> for SmartPort {
    fn from(device: T) -> Self {
        // SAFETY: We can do this, since we ensure that the old smartport was disposed of.
        // This can effectively be thought as a move out of the device's private `port` field.
        unsafe { Self::new(device.port_index()) }
    }
}

/// Verify that the device type is currently plugged into this port.
///
/// This function provides the internal implementations of [`SmartDevice::validate_port`], [`SmartPort::validate_type`],
/// and [`AdiPort::validate_expander`].
pub(crate) fn validate_port(index: u8, device_type: SmartDeviceType) -> Result<(), PortError> {
    let device = unsafe { *vexDeviceGetByIndex((index - 1) as u32) };
    let plugged_type: SmartDeviceType = device.device_type.into();

    if !device.installed {
        // No device is plugged into the port.
        return Err(PortError::Disconnected);
    } else if plugged_type != device_type {
        // The connected device doesn't match the requested type.
        return Err(PortError::IncorrectDevice);
    }

    Ok(())
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
    pub fn device_type(&self) -> Result<SmartDeviceType, PortError> {
        Ok(unsafe { *vexDeviceGetByIndex((self.index() - 1) as u32) }
            .device_type
            .into())
    }

    /// Verify that a device type is currently plugged into this port, returning an appropriate
    /// [`PortError`] if not available.
    pub fn validate_type(&self, device_type: SmartDeviceType) -> Result<(), PortError> {
        validate_port(self.index(), device_type)
    }

    /// Get the raw handle of the underlying smart device connected to this port.
    pub(crate) unsafe fn device_handle(&self) -> V5_DeviceT {
        unsafe { vexDeviceGetByIndex((self.index() - 1) as u32) }
    }
}

/// Represents a possible type of device that can be registered on a [`SmartPort`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SmartDeviceType {
    /// No device
    None,

    /// Smart Motor
    Motor,

    /// Rotation Sensor
    Rotation,

    /// Inertial Sensor
    Imu,

    /// Distance Sensor
    Distance,

    /// Vision Sensor
    Vision,

    /// AI Vision Sensor
    AiVision,

    /// Workcell Electromagnet
    Magnet,

    /// CTE Workcell Light Tower
    LightTower,

    /// CTE Workcell Arm
    Arm,

    /// Optical Sensor
    Optical,

    /// GPS Sensor
    Gps,

    /// Smart Radio
    Radio,

    /// ADI Expander
    ///
    /// This variant is also internally to represent the brain's onboard ADI slots.
    Adi,

    /// Generic Serial Port
    GenericSerial,

    /// Other device type code returned by the SDK that is currently unsupported, undocumented,
    /// or unknown.
    Unknown(V5_DeviceType),
}

impl From<V5_DeviceType> for SmartDeviceType {
    fn from(value: V5_DeviceType) -> Self {
        match value {
            V5_DeviceType::kDeviceTypeNoSensor => Self::None,
            V5_DeviceType::kDeviceTypeMotorSensor => Self::Motor,
            V5_DeviceType::kDeviceTypeAbsEncSensor => Self::Rotation,
            V5_DeviceType::kDeviceTypeImuSensor => Self::Imu,
            V5_DeviceType::kDeviceTypeDistanceSensor => Self::Distance,
            V5_DeviceType::kDeviceTypeRadioSensor => Self::Radio,
            V5_DeviceType::kDeviceTypeVisionSensor => Self::Vision,
            V5_DeviceType::kDeviceTypeAdiSensor => Self::Adi,
            V5_DeviceType::kDeviceTypeOpticalSensor => Self::Optical,
            V5_DeviceType::kDeviceTypeMagnetSensor => Self::Magnet,
            V5_DeviceType::kDeviceTypeGpsSensor => Self::Gps,
            V5_DeviceType::kDeviceTypeLightTowerSensor => Self::LightTower,
            V5_DeviceType::kDeviceTypeArmDevice => Self::Arm,
            V5_DeviceType::kDeviceTypeAiVisionSensor => Self::AiVision,
            V5_DeviceType::kDeviceTypeGenericSerial => Self::GenericSerial,
            other => Self::Unknown(other),
        }
    }
}

impl From<SmartDeviceType> for V5_DeviceType {
    fn from(value: SmartDeviceType) -> Self {
        match value {
            SmartDeviceType::None => V5_DeviceType::kDeviceTypeNoSensor,
            SmartDeviceType::Motor => V5_DeviceType::kDeviceTypeMotorSensor,
            SmartDeviceType::Rotation => V5_DeviceType::kDeviceTypeAbsEncSensor,
            SmartDeviceType::Imu => V5_DeviceType::kDeviceTypeImuSensor,
            SmartDeviceType::Distance => V5_DeviceType::kDeviceTypeDistanceSensor,
            SmartDeviceType::Vision => V5_DeviceType::kDeviceTypeVisionSensor,
            SmartDeviceType::AiVision => V5_DeviceType::kDeviceTypeAiVisionSensor,
            SmartDeviceType::Magnet => V5_DeviceType::kDeviceTypeMagnetSensor,
            SmartDeviceType::LightTower => V5_DeviceType::kDeviceTypeLightTowerSensor,
            SmartDeviceType::Arm => V5_DeviceType::kDeviceTypeArmDevice,
            SmartDeviceType::Optical => V5_DeviceType::kDeviceTypeOpticalSensor,
            SmartDeviceType::Gps => V5_DeviceType::kDeviceTypeGpsSensor,
            SmartDeviceType::Radio => V5_DeviceType::kDeviceTypeRadioSensor,
            SmartDeviceType::Adi => V5_DeviceType::kDeviceTypeAdiSensor,
            SmartDeviceType::GenericSerial => V5_DeviceType::kDeviceTypeGenericSerial,
            SmartDeviceType::Unknown(raw_type) => raw_type,
        }
    }
}

/// Represents a timestamp on a smart device's internal clock. This type offers
/// no guarantees that the device's clock is in sync with the internal clock of
/// the brain, and thus cannot be safely compared with [`vexide_core::time::Instant`]s.
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
