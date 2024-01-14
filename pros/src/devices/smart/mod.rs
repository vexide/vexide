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
pub mod optical;
pub mod rotation;
pub mod vision;
pub mod motor;
pub mod link;

pub use distance::DistanceSensor;
pub use gps::GpsSensor;
pub use imu::InertialSensor;
pub use optical::OpticalSensor;
pub use rotation::RotationSensor;
pub use vision::VisionSensor;
pub use motor::Motor;

use crate::prelude::PortError;
use crate::error::bail_on;

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
    fn device_connected(&self) -> bool {
		let plugged_type_result: Result<SmartDeviceType, _> = unsafe { pros_sys::apix::registry_get_plugged_type(self.port_index() - 1).try_into() };
		
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
    pub index: u8,
}

impl SmartPort {
    /// Create a new port.
    ///
    /// # Safety
    ///
    /// Creating new `ADIPort`s is inherently unsafe due to the possibility of constructing
    /// more than one device on the same port index allowing multiple mutable references to
    /// the same hardware device. Prefer using [`Peripherals`] to register devices if possible.
    pub unsafe fn new(index: u8) -> Self {
        Self { index }
    }

    /// Get the index of the port (port number).
    ///
    /// Ports are indexed starting from 1.
    pub fn index(&self) -> u8 {
        self.index
    }

	/// Get the type of device currently connected to this port.
	pub fn connected_type(&self) -> Result<SmartDeviceType, PortError> {
		unsafe { pros_sys::apix::registry_get_plugged_type(self.index() - 1).try_into() }
	}

    /// Get the type of device currently registered to this port.
	pub fn bound_type(&self) -> Result<SmartDeviceType, PortError> {
		unsafe { pros_sys::apix::registry_get_bound_type(self.index() - 1).try_into() }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SmartDeviceType {
    None = pros_sys::apix::E_DEVICE_NONE,
    Motor = pros_sys::apix::E_DEVICE_MOTOR,
    RotationSensor = pros_sys::apix::E_DEVICE_ROTATION,
    InertialSensor = pros_sys::apix::E_DEVICE_IMU,
    DistanceSensor = pros_sys::apix::E_DEVICE_DISTANCE,
    VisionSensor = pros_sys::apix::E_DEVICE_VISION,
    OpticalSensor = pros_sys::apix::E_DEVICE_OPTICAL,
    GpsSensor = pros_sys::apix::E_DEVICE_GPS,
    Radio = pros_sys::apix::E_DEVICE_RADIO,
    AdiExpander = pros_sys::apix::E_DEVICE_ADI,
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
            pros_sys::apix::E_DEVICE_ROTATION => Self::RotationSensor,
            pros_sys::apix::E_DEVICE_IMU => Self::InertialSensor,
            pros_sys::apix::E_DEVICE_DISTANCE => Self::DistanceSensor,
            pros_sys::apix::E_DEVICE_VISION => Self::VisionSensor,
            pros_sys::apix::E_DEVICE_OPTICAL => Self::OpticalSensor,
            pros_sys::apix::E_DEVICE_RADIO => Self::Radio,
            pros_sys::apix::E_DEVICE_ADI => Self::AdiExpander,
            pros_sys::apix::E_DEVICE_SERIAL => Self::Serial,
            _ => unreachable!(),
        })
	}
}