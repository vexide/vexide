//! GPS sensor device.

use core::marker::PhantomData;

use vex_sdk::{
    vexDeviceGpsAttitudeGet, vexDeviceGpsDegreesGet, vexDeviceGpsHeadingGet,
    vexDeviceGpsInitialPositionSet, vexDeviceGpsOriginGet, vexDeviceGpsOriginSet,
    vexDeviceGpsQuaternionGet, vexDeviceGpsRawAccelGet, vexDeviceGpsRawGyroGet,
    V5_DeviceGpsAttitude, V5_DeviceGpsQuaternion, V5_DeviceGpsRaw, V5_DeviceT,
};

use super::{validate_port, SmartDevice, SmartDeviceType, SmartPort};
use crate::{geometry::Point2, PortError};

/// GPS Sensor Device
#[derive(Debug, Eq, PartialEq)]
pub struct GpsSensor {
    port: SmartPort,
    device: V5_DeviceT,

    /// Internal IMU
    pub imu: GpsImu,
}

/// GPS Sensor Internal IMU
#[derive(Debug, Eq, PartialEq)]
pub struct GpsImu {
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for GpsSensor {}
unsafe impl Sync for GpsSensor {}
unsafe impl Send for GpsImu {}
unsafe impl Sync for GpsImu {}

impl GpsSensor {
    /// Create a new GPS sensor.
    pub fn new(
        port: SmartPort,
        offset: impl Into<Point2<f64>>,
        initial_pose: (impl Into<Point2<f64>>, f64),
    ) -> Result<Self, PortError> {
        port.validate_type(SmartDeviceType::Gps)?;

        let device = unsafe { port.device_handle() };

        let initial_position = initial_pose.0.into();
        let offset = offset.into();

        unsafe {
            vexDeviceGpsOriginSet(device, offset.x, offset.y);
            vexDeviceGpsInitialPositionSet(
                device,
                initial_position.x,
                initial_position.y,
                initial_pose.1,
            );
        }

        Ok(Self {
            device,
            port,
            imu: GpsImu { device },
        })
    }

    pub fn offset(&self) -> Result<Point2<f64>, PortError> {
        self.validate_port()?;

        let mut data = Point2::<f64>::default();
        unsafe { vexDeviceGpsOriginGet(self.device, &mut data.x, &mut data.y) }

        Ok(data)
    }

    pub fn pose(&self) -> Result<(Point2<f64>, f64), PortError> {
        self.validate_port()?;

        let mut attitude = V5_DeviceGpsAttitude::default();
        unsafe {
            vexDeviceGpsAttitudeGet(self.device, &mut attitude, false);
        }

        let heading = unsafe { vexDeviceGpsDegreesGet(self.device) };

        Ok((
            Point2::<f64>::new(attitude.position_x, attitude.position_y),
            heading,
        ))
    }
}

impl SmartDevice for GpsSensor {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Gps
    }
}

impl GpsImu {
    fn validate_port(&self) -> Result<(), PortError> {
        validate_port(
            unsafe { (*self.device).zero_indexed_port },
            SmartDeviceType::Gps,
        )
    }

    pub fn heading(&self) -> Result<f64, PortError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceGpsDegreesGet(self.device) })
    }

    pub fn rotation(&self) -> Result<f64, PortError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceGpsHeadingGet(self.device) })
    }

    pub fn euler(&self) -> Result<mint::EulerAngles<f64, f64>, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceGpsAttitude::default();
        unsafe {
            vexDeviceGpsAttitudeGet(self.device, &mut data, false);
        }

        Ok(mint::EulerAngles {
            a: data.pitch.to_radians(),
            b: data.yaw.to_radians(),
            c: data.roll.to_radians(),
            marker: PhantomData,
        })
    }

    pub fn quaternion(&self) -> Result<mint::Quaternion<f64>, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceGpsQuaternion::default();
        unsafe {
            vexDeviceGpsQuaternionGet(self.device, &mut data);
        }

        Ok(mint::Quaternion {
            v: mint::Vector3 {
                x: data.x,
                y: data.y,
                z: data.z,
            },
            s: data.w,
        })
    }

    pub fn accel(&self) -> Result<mint::Vector3<f64>, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceGpsRaw::default();
        unsafe {
            vexDeviceGpsRawAccelGet(self.device, &mut data);
        }

        Ok(mint::Vector3 {
            x: data.x,
            y: data.y,
            z: data.z,
        })
    }

    pub fn gyro_rate(&self) -> Result<mint::Vector3<f64>, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceGpsRaw::default();
        unsafe {
            vexDeviceGpsRawGyroGet(self.device, &mut data);
        }

        Ok(mint::Vector3 {
            x: data.x,
            y: data.y,
            z: data.z,
        })
    }
}
