//! Smart ports and devices.
//!
//! This module provides abstractions for devices connected through VEX Smart Ports. This includes
//! motors, many common sensors, and raw serial access.
//!
//! # Overview
//!
//! The V5 Brain features 21 RJ9 ports known as "Smart Ports" for connecting newer V5 devices to the
//! Brain. These ports use serial communication over a 4-wire RS-485 protocol to allow the device
//! and the Brain to send commands to each other.
//!
//! # Ports
//!
//! There are 21 instances of the [`SmartPort`] struct available to you through the [`Peripherals`]
//! instance passed to your `main` function. Each [`SmartPort`] represents a physical port on the
//! Brain.
//!
//! Smart Ports are labeled 1 through 21 on the Brain and in the [`Peripherals`] API. If we wanted
//! to do something with port 1, we can simply use `peripherals.port_1`:
//!
//! ```
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main(peripherals: Peripherals) {
//!     let port_1 = peripherals.port_1;
//! }
//! ```
//!
//! # Devices
//!
//! Most devices are created using a `new` function that takes ownership of a [`SmartPort`], along
//! with any other device-specific parameters. All devices are thread-safe due to being singletons,
//! and can only be safely constructed using the [`Peripherals`] API. The general device creation
//! process looks like this:
//!
//! [`Peripherals`]: crate::peripherals::Peripherals
//!
//! ```no_run
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main(peripherals: Peripherals) {
//!     // Create two new motors on Smart Ports 1 and 10.
//!     let mut left_motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
//!     let mut right_motor = Motor::new(peripherals.port_10, Gearset::Green, Direction::Forward);
//!
//!     // Create a new inertial sensor (IMU) on Smart Port 6.
//!     let mut imu = InertialSensor::new(peripherals.port_6);
//! }
//! ```
//!
//! Specific info for using each device is available in their respective modules below.

use vex_sdk::{
    V5_DeviceT, V5_DeviceType, V5_MAX_DEVICE_PORTS, vexDeviceGetByIndex, vexDeviceGetStatus,
    vexDeviceGetTimestamp,
};
use vexide_core::time::LowResolutionTime;

pub mod ai_vision;
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

use core::time::Duration;

use snafu::{Snafu, ensure};

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
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = InertialSensor::new(peripherals.port_1);
    ///     assert_eq!(sensor.port_number(), 1);
    /// }
    /// ```
    fn port_number(&self) -> u8;

    /// Returns the variant of [`SmartDeviceType`] that this device is associated with.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::{prelude::*, smart::SmartDeviceType};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = InertialSensor::new(peripherals.port_1);
    ///     assert_eq!(sensor.device_type(), SmartDeviceType::Imu);
    /// }
    /// ```
    fn device_type(&self) -> SmartDeviceType;

    /// Determines if this device type is currently plugged into its [`SmartPort`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     if sensor.is_connected() {
    ///         println!("IMU is connected!");
    ///     } else {
    ///         println!("No IMU connection found.");
    ///     }
    /// }
    /// ```
    fn is_connected(&self) -> bool {
        let mut device_types: [V5_DeviceType; V5_MAX_DEVICE_PORTS] = unsafe { core::mem::zeroed() };
        unsafe {
            vexDeviceGetStatus(device_types.as_mut_ptr());
        }

        SmartDeviceType::from(device_types[(self.port_number() - 1) as usize]) == self.device_type()
    }

    /// Returns a timestamp recorded when the last packet sent by this device was processed by
    /// VEXos.
    ///
    /// # Precision
    ///
    /// This is a timestamp from the brain's low-resolution timer, meaning it has a precision of 1
    /// millisecond. See the [`LowResolutionTime`] API for more information.
    ///
    /// # Errors
    ///
    /// Currently, this function never returns an error. This behavior should be considered
    /// unstable.
    fn timestamp(&self) -> Result<LowResolutionTime, PortError> {
        Ok(LowResolutionTime::from_millis_since_epoch(unsafe {
            vexDeviceGetTimestamp(vexDeviceGetByIndex(u32::from(self.port_number() - 1)))
        }))
    }

    /// Verify that the device type is currently plugged into this port, returning an appropriate
    /// [`PortError`] if not available.
    ///
    /// # Errors
    ///
    /// Returns a [`PortError`] if there is not a physical device of type
    /// [`SmartDevice::device_type`] in this [`SmartDevice`]'s port.
    fn validate_port(&self) -> Result<(), PortError> {
        validate_port(self.port_number(), self.device_type())
    }
}

/// Verify that the device type is currently plugged into this port.
///
/// This function provides the internal implementations of [`SmartDevice::validate_port`],
/// [`SmartPort::validate_type`], and [`AdiPort::validate_expander`].
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

/// A Smart Port on a Brain.
#[derive(Debug, Eq, PartialEq)]
pub struct SmartPort {
    /// The number of the port (port number).
    ///
    /// Ports are numbered starting from 1.
    number: u8,
}

impl SmartPort {
    /// Creates a new Smart Port on a specified port number.
    ///
    /// # Safety
    ///
    /// Creating new `SmartPort`s is inherently unsafe due to the possibility of constructing more
    /// than one device on the same port index allowing multiple mutable references to the same
    /// hardware device. This violates Rust's borrow checker guarantees. Prefer using
    /// [`Peripherals`](crate::peripherals::Peripherals) to register devices if possible.
    ///
    /// For more information on safely creating peripherals, see [this page](https://vexide.dev/docs/peripherals/).
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::smart::SmartPort;
    ///
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
    /// use vexide::smart::SmartPort;
    ///
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

    /// Returns the type of device currently connected to this port, or `None` if no device is
    /// connected.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::smart::SmartPort;
    ///
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

    /// Returns the timestamp of the last device packet processed by this port, or `None` if no
    /// device is connected.
    ///
    /// # Precision
    ///
    /// This is a timestamp from the brain's low-resolution timer, meaning it has a precision of 1
    /// millisecond. See the [`LowResolutionTime`] API for more information.
    #[must_use]
    pub fn timestamp(&self) -> Option<LowResolutionTime> {
        if self.device_type().is_some() {
            Some(LowResolutionTime::from_millis_since_epoch(unsafe {
                vexDeviceGetTimestamp(vexDeviceGetByIndex(self.index()))
            }))
        } else {
            None
        }
    }

    /// Returns the raw handle of the underlying Smart device connected to this port.
    pub(crate) unsafe fn device_handle(&self) -> V5_DeviceT {
        unsafe { vexDeviceGetByIndex(self.index()) }
    }
}

/// A possible type of device that can be plugged into a [`SmartPort`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[non_exhaustive]
pub enum SmartDeviceType {
    /// Smart Motor
    ///
    /// This corresponds to the [`Motor`](motor::Motor) device.
    Motor,

    /// Rotation Sensor
    ///
    /// This corresponds to the [`RotationSensor`](rotation::RotationSensor) device.
    Rotation,

    /// Inertial Sensor
    ///
    /// This corresponds to the [`InertialSensor`](imu::InertialSensor) device.
    Imu,

    /// Distance Sensor
    ///
    /// This corresponds to the [`DistanceSensor`](distance::DistanceSensor) device.
    Distance,

    /// Vision Sensor
    ///
    /// This corresponds to the [`VisionSensor`](vision::VisionSensor) device.
    Vision,

    /// AI Vision Sensor
    ///
    /// This corresponds to the [`AiVisionSensor`](ai_vision::AiVisionSensor) device.
    AiVision,

    /// Workcell Electromagnet
    ///
    /// This corresponds to the [`Electromagnet`](electromagnet::Electromagnet) device.
    Electromagnet,

    /// CTE Workcell Light Tower
    LightTower,

    /// CTE Workcell Arm
    Arm,

    /// Optical Sensor
    ///
    /// This corresponds to the [`OpticalSensor`](optical::OpticalSensor) device.
    Optical,

    /// GPS Sensor
    ///
    /// This corresponds to the [`GpsSensor`](gps::GpsSensor) device.
    Gps,

    /// Smart Radio
    Radio,

    /// ADI Expander
    ///
    /// This corresponds to the [`AdiExpander`](expander::AdiExpander) device.
    Adi,

    /// Generic Serial Port
    ///
    /// This corresponds to the [`SerialPort`](serial::SerialPort) device.
    GenericSerial,

    /// Other device type code returned by the SDK that is currently unsupported, undocumented, or
    /// unknown.
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

/// Errors that can occur when performing operations on [`SmartPort`]-connected devices.
///
/// Most smart devices will return this type or something wrapping this type when an error occurs.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Snafu)]
pub enum PortError {
    /// No device was plugged into the port, when one was expected.
    #[snafu(display("Expected a device to be connected to port {port}"))]
    Disconnected {
        /// The port that was expected to have a device
        port: u8,
    },

    /// The wrong type of device is plugged into the port.
    #[snafu(display(
        "Expected a {expected:?} device on port {port}, but found a {actual:?} device"
    ))]
    IncorrectDevice {
        /// The device type that was expected
        expected: SmartDeviceType,
        /// The device type that was found
        actual: SmartDeviceType,
        /// The port that was expected to have a device
        port: u8,
    },
}

#[cfg(feature = "std")]
impl From<PortError> for std::io::Error {
    fn from(value: PortError) -> Self {
        match value {
            PortError::Disconnected { .. } => std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                "A device is not connected to the specified port.",
            ),
            PortError::IncorrectDevice { .. } => std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                "Port is in use as another device.",
            ),
        }
    }
}

#[cfg(feature = "embedded-io")]
impl embedded_io::Error for PortError {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            PortError::Disconnected { .. } => embedded_io::ErrorKind::AddrNotAvailable,
            PortError::IncorrectDevice { .. } => embedded_io::ErrorKind::AddrInUse,
        }
    }
}
