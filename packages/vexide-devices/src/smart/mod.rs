//! Smart Ports & Devices
//!
//! This module provides abstractions for devices connected through VEX Smart Ports. This
//! includes motors, many common sensors, vexlink, and raw serial access.
//!
//! # Hardware Overview
//!
//! The V5 Brain features 21 RJ9 4p4c connector ports (known as "Smart Ports") for communicating with
//! newer V5 peripherals. Smart Port devices have a variable sample rate (unlike ADI, which is limited
//! to 10ms), and can support basic data transfer over serial.
//!
//! # Smart Port Devices
//!
//! Most devices can be created with a `new` function that generally takes a [`SmartPort`] instance from [`Peripherals`](crate::peripherals::Peripherals)
//! along with other device-specific parameters. All sensors are thread safe, however sensors can only be safely constructed
//! using the [`peripherals`] API. The general device construction pattern looks like this:
//! ```no_run
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main(peripherals: Peripherals) {
//!     // Create a new device on port 1.
//!     let mut device = Device::new(peripherals.port_1, /* other parameters */);
//!     // Use the device.
//!     // Device errors are usually only returned by methods, and not the constructor.
//!     let _ = device.do_something();
//! }
//! ```
//!
//! More specific info for each device is available in their respective modules.
//!
//! [`peripherals`]: crate::peripherals

use core::fmt;

use vex_sdk::{
    vexDeviceGetByIndex, vexDeviceGetStatus, vexDeviceGetTimestamp, V5_DeviceT, V5_DeviceType,
    V5_MAX_DEVICE_PORTS,
};

use crate::PortError;

pub mod distance;
pub mod electromagnet;
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

use core::time::Duration;

pub use distance::DistanceSensor;
pub use electromagnet::Electromagnet;
pub use expander::AdiExpander;
pub use gps::GpsSensor;
pub use imu::InertialSensor;
pub use link::RadioLink;
pub use motor::Motor;
pub use optical::OpticalSensor;
pub use rotation::RotationSensor;
pub use serial::SerialPort;
use snafu::ensure;
pub use vision::VisionSensor;

use crate::{DisconnectedSnafu, IncorrectDeviceSnafu};

/// Defines common functionality shared by all Smart Port devices.

pub trait SmartDevice {
    /// The interval at which the V5 brain reads packets from Smart devices.
    const UPDATE_INTERVAL: Duration = Duration::from_millis(10);

    /// Returns the port number of the [`SmartPort`] this device is registered on.
    ///
    /// Ports are numbered starting from 1.
    ///
    /// # Examples
    ///
    /// ```
    /// let sensor = InertialSensor::new(peripherals.port_1)?;
    /// assert_eq!(sensor.port_number(), 1);
    /// ```
    fn port_number(&self) -> u8;

    /// Returns the variant of [`SmartDeviceType`] that this device is associated with.
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
        let mut device_types: [V5_DeviceType; V5_MAX_DEVICE_PORTS] = unsafe { core::mem::zeroed() };
        unsafe {
            vexDeviceGetStatus(device_types.as_mut_ptr());
        }

        SmartDeviceType::from(device_types[(self.port_number() - 1) as usize]) == self.device_type()
    }

    /// Returns the timestamp recorded by this device's internal clock.
    ///
    /// # Errors
    ///
    /// Currently, this function never returns an error. This behavior should be considered unstable.
    fn timestamp(&self) -> Result<SmartDeviceTimestamp, PortError> {
        Ok(SmartDeviceTimestamp(unsafe {
            vexDeviceGetTimestamp(vexDeviceGetByIndex(u32::from(self.port_number() - 1)))
        }))
    }

    /// Verify that the device type is currently plugged into this port, returning an appropriate
    /// [`PortError`] if not available.
    ///
    /// # Errors
    ///
    /// Returns a [`PortError`] if there is not a physical device of type [`SmartDevice::device_type`] in this [`SmartDevice`]'s port.
    fn validate_port(&self) -> Result<(), PortError> {
        validate_port(self.port_number(), self.device_type())
    }
}

/// Verify that the device type is currently plugged into this port.
///
/// This function provides the internal implementations of [`SmartDevice::validate_port`], [`SmartPort::validate_type`],
/// and [`AdiPort::validate_expander`].
pub(crate) fn validate_port(number: u8, device_type: SmartDeviceType) -> Result<(), PortError> {
    let mut device_types: [V5_DeviceType; V5_MAX_DEVICE_PORTS] = unsafe { core::mem::zeroed() };
    unsafe {
        vexDeviceGetStatus(device_types.as_mut_ptr());
    }

    let connected_type: Option<SmartDeviceType> = match device_types[(number - 1) as usize] {
        V5_DeviceType::kDeviceTypeNoSensor => None,
        raw_type => Some(raw_type.into()),
    };

    if let Some(connected_type) = connected_type {
        // The connected device must match the requested type.
        ensure!(
            connected_type == device_type,
            IncorrectDeviceSnafu {
                expected: device_type,
                actual: connected_type,
                port: number,
            }
        );
    } else {
        // No device is plugged into the port.
        return DisconnectedSnafu { port: number }.fail();
    }

    Ok(())
}

/// Represents a Smart Port on a V5 Brain
#[derive(Debug, Eq, PartialEq)]
pub struct SmartPort {
    /// The number of the port (port number).
    ///
    /// Ports are numbered starting from 1.
    number: u8,
}

impl SmartPort {
    /// Creates a new Smart Port on a specified index.
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
    /// // Create a new Smart Port at index 1.
    /// // This is unsafe! You are responsible for ensuring that only one device registered on a
    /// // single port index.
    /// let my_port = unsafe { SmartPort::new(1) };
    /// ```
    #[must_use]
    pub const unsafe fn new(number: u8) -> Self {
        Self { number }
    }

    /// Returns the number of the port.
    ///
    /// Ports are numbered starting from 1.
    ///
    /// # Examples
    ///
    /// ```
    /// let my_port = unsafe { SmartPort::new(1) };
    ///
    /// assert_eq!(my_port.number(), 1);
    /// ```
    #[must_use]
    pub const fn number(&self) -> u8 {
        self.number
    }

    pub(crate) const fn index(&self) -> u32 {
        (self.number - 1) as u32
    }

    /// Returns the type of device currently connected to this port, or `None`
    /// if no device is connected.
    ///
    /// # Examples
    ///
    /// ```
    /// let my_port = unsafe { SmartPort::new(1) };
    ///
    /// if let Some(device_type) = my_port.device_type() {
    ///     println!("Type of device connected to port 1: {:?}", device_type);
    /// }
    /// ```
    #[must_use]
    pub fn device_type(&self) -> Option<SmartDeviceType> {
        let mut device_types: [V5_DeviceType; V5_MAX_DEVICE_PORTS] = unsafe { core::mem::zeroed() };
        unsafe {
            vexDeviceGetStatus(device_types.as_mut_ptr());
        }

        match device_types[self.index() as usize] {
            V5_DeviceType::kDeviceTypeNoSensor => None,
            raw_type => Some(raw_type.into()),
        }
    }

    /// Verify that a device type is currently plugged into this port, returning an appropriate
    /// [`PortError`] if not available.
    ///
    /// # Errors
    ///
    /// Returns a [`PortError`] if there is not a device of the specified type in this port.
    pub fn validate_type(&self, device_type: SmartDeviceType) -> Result<(), PortError> {
        if let Some(connected_type) = self.device_type() {
            // The connected device must match the requested type.
            ensure!(
                connected_type == device_type,
                IncorrectDeviceSnafu {
                    expected: device_type,
                    actual: connected_type,
                    port: self.number,
                }
            );
        } else {
            // No device is plugged into the port.
            return DisconnectedSnafu { port: self.number }.fail();
        }

        Ok(())
    }

    /// Returns the raw handle of the underlying Smart device connected to this port.
    pub(crate) unsafe fn device_handle(&self) -> V5_DeviceT {
        unsafe { vexDeviceGetByIndex(self.index()) }
    }
}

/// A possible type of device that can be plugged into a [`SmartPort`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SmartDeviceType {
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
    Electromagnet,

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
    /// This variant is also internally to represent the Brain's onboard ADI slots.
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
            V5_DeviceType::kDeviceTypeMotorSensor => Self::Motor,
            V5_DeviceType::kDeviceTypeAbsEncSensor => Self::Rotation,
            V5_DeviceType::kDeviceTypeImuSensor => Self::Imu,
            V5_DeviceType::kDeviceTypeDistanceSensor => Self::Distance,
            V5_DeviceType::kDeviceTypeRadioSensor => Self::Radio,
            V5_DeviceType::kDeviceTypeVisionSensor => Self::Vision,
            V5_DeviceType::kDeviceTypeAdiSensor => Self::Adi,
            V5_DeviceType::kDeviceTypeOpticalSensor => Self::Optical,
            V5_DeviceType::kDeviceTypeMagnetSensor => Self::Electromagnet,
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
            SmartDeviceType::Motor => V5_DeviceType::kDeviceTypeMotorSensor,
            SmartDeviceType::Rotation => V5_DeviceType::kDeviceTypeAbsEncSensor,
            SmartDeviceType::Imu => V5_DeviceType::kDeviceTypeImuSensor,
            SmartDeviceType::Distance => V5_DeviceType::kDeviceTypeDistanceSensor,
            SmartDeviceType::Vision => V5_DeviceType::kDeviceTypeVisionSensor,
            SmartDeviceType::AiVision => V5_DeviceType::kDeviceTypeAiVisionSensor,
            SmartDeviceType::Electromagnet => V5_DeviceType::kDeviceTypeMagnetSensor,
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

/// Represents a timestamp on a Smart device's internal clock.
///
/// This type offers no guarantees that the device's clock is in sync with the internal
/// clock of the Brain, and thus cannot be safely compared with [`vexide_core::time::Instant`]s.
///
/// There is additionally no guarantee that this is in sync with other Smart devices,
/// or even the same device if a disconnect occurred causing the clock to reset. As such,
/// this is effectively a wrapper of `u32`.
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
