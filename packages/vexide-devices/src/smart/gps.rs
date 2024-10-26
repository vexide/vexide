//! GPS Sensor
//!
//! This module provides an interface to interact with the VEX V5 GPS Sensor,
//! which uses computer vision and an inertial measurement unit (IMU) to provide absolute
//! position tracking within a VEX Robotics Competition field.
//!
//! # Hardware Description
//!
//! The GPS sensor combines a monochrome camera and an IMU for robust position tracking
//! through visual odometry. It works by detecting QR-like patterns on the field perimeter,
//! using both the pattern sequence's and apparent size for position determination. The
//! integrated IMU provides motion tracking for position estimation when visual tracking
//! is unavailable or unreliable.
//!
//! The sensor has specific operating ranges: it requires a minimum
//! distance of 20 inches from the field perimeter for reliable readings, has a deadzone
//! between 0-13.5 inches, and maintains accuracy up to 12 feet from the perimeter.
//!
//! Sensor fusion between the camera and IMU helps maintain position tracking through
//! dead zones and areas of inconsistent visual detection.
//!
//! Further information about the sensor's method of operation can be found in [IFI's patent](https://docs.google.com/viewerng/viewer?url=https://patentimages.storage.googleapis.com/4f/74/30/eccf334da0ae38/WO2020219788A1.pdf).

use core::{marker::PhantomData, time::Duration};

use vex_sdk::{
    vexDeviceGpsAttitudeGet, vexDeviceGpsDataRateSet, vexDeviceGpsDegreesGet, vexDeviceGpsErrorGet,
    vexDeviceGpsHeadingGet, vexDeviceGpsInitialPositionSet, vexDeviceGpsOriginGet,
    vexDeviceGpsOriginSet, vexDeviceGpsQuaternionGet, vexDeviceGpsRawAccelGet,
    vexDeviceGpsRawGyroGet, vexDeviceGpsRotationGet, vexDeviceGpsStatusGet, V5_DeviceGpsAttitude,
    V5_DeviceGpsQuaternion, V5_DeviceGpsRaw, V5_DeviceT,
};

use super::{validate_port, SmartDevice, SmartDeviceType, SmartPort};
use crate::{geometry::Point2, PortError};

/// A GPS sensor plugged into a Smart Port.
#[derive(Debug, PartialEq)]
pub struct GpsSensor {
    port: SmartPort,
    device: V5_DeviceT,

    /// Internal IMU
    pub imu: GpsImu,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for GpsSensor {}
unsafe impl Sync for GpsSensor {}

impl GpsSensor {
    /// Creates a new GPS sensor from a [`SmartPort`].
    ///
    /// # Configuration
    ///
    /// The sensor requires two parameters to be initially configured, passed as arguments ot this function:
    ///
    /// - `offset`: The physical offset of the sensor from the robot's center of rotation.
    /// - `initial_pose`: The inital position and heading of the robot.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the specified port.
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
                360.0 - initial_pose.1,
            );
        }

        Ok(Self {
            device,
            imu: GpsImu {
                device,
                port_number: port.number(),
                rotation_offset: Default::default(),
                heading_offset: Default::default(),
            },
            port,
        })
    }

    /// Returns the physical offset of the sensor from the robot's center of rotation.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn offset(&self) -> Result<Point2<f64>, PortError> {
        self.validate_port()?;

        let mut data = Point2::<f64>::default();
        unsafe { vexDeviceGpsOriginGet(self.device, &mut data.x, &mut data.y) }

        Ok(data)
    }

    /// Returns the currently computed pose (heading and position) from the sensor.
    ///
    /// # Important note about heading!
    ///
    /// The heading returned here is in a different angle system from [`GpsImu::heading`]! The heading
    /// returned by this function increases as the sensor turns **counterclockwise**, while the opposite
    /// is true for [`GpsImu`]. This is done to make it easier to use trig functions with the coordinates
    /// returned by the sensor, as anything involving cartesian coordinates are expected to be in standard
    /// unit circle angles. In addition, this function is not affected by [`GpsImu::reset_heading`] or
    /// [`GpsImu::set_heading`].
    ///
    /// > You should **never** attempt to use the [`GpsImu`] angles when dealing with position data from this sensor
    /// > unless you understand exactly what you're doing.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn pose(&self) -> Result<(Point2<f64>, f64), PortError> {
        self.validate_port()?;

        let mut attitude = V5_DeviceGpsAttitude::default();
        unsafe {
            vexDeviceGpsAttitudeGet(self.device, &mut attitude, false);
        }

        let heading = (360.0
            - unsafe {
                vexDeviceGpsRotationGet(self.device) + vexDeviceGpsDegreesGet(self.device)
            })
            % 360.0;

        Ok((
            Point2::<f64>::new(attitude.position_x, attitude.position_y),
            heading,
        ))
    }

    /// Returns the RMS (Root Mean Squared) error for the GPS position reading in meters.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn error(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGpsErrorGet(self.device) })
    }

    /// Returns the internal status code of the sensor.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn status(&self) -> Result<u32, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGpsStatusGet(self.device) })
    }
}

impl SmartDevice for GpsSensor {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Gps
    }
}
impl From<GpsSensor> for SmartPort {
    fn from(device: GpsSensor) -> Self {
        device.port
    }
}

/// GPS Sensor Internal IMU
#[derive(Debug, PartialEq)]
pub struct GpsImu {
    port_number: u8,
    device: V5_DeviceT,
    rotation_offset: f64,
    heading_offset: f64,
}

// I'm sure you know the drill at this point...
unsafe impl Send for GpsImu {}
unsafe impl Sync for GpsImu {}

impl GpsImu {
    /// The maximum value that can be returned by [`Self::heading`].
    pub const MAX_HEADING: f64 = 360.0;

    fn validate_port(&self) -> Result<(), PortError> {
        validate_port(self.port_number, SmartDeviceType::Gps)
    }

    /// Returns the IMU's yaw angle bounded by [0, 360) degrees.
    ///
    /// Clockwise rotations are represented with positive degree values, while counterclockwise rotations are
    /// represented with negative ones.
    ///
    /// # Important
    ///
    /// This value does not take into account the initial heading passed into [`GpsSensor::new`], and additionally
    /// uses a different angle system compared to the main [`GpsSensor`] struct (with the positive direction being
    /// clockwise). As such, this should not be used for doing any kind of math in tandem with the GPS sensor's
    /// position readings. Prefer using [`GpsSensor::pose`] for that.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn heading(&self) -> Result<f64, PortError> {
        self.validate_port()?;
        Ok(
            (unsafe { vexDeviceGpsDegreesGet(self.device) } - self.heading_offset)
                % Self::MAX_HEADING,
        )
    }

    /// Returns the total number of degrees the IMU has spun about the z-axis.
    ///
    /// This value is theoretically unbounded. Clockwise rotations are represented with positive degree values,
    /// while counterclockwise rotations are represented with negative ones.
    ///
    /// # Important
    ///
    /// This value does not take into account the initial heading passed into [`GpsSensor::new`], and additionally
    /// uses a different angle system compared to the main [`GpsSensor`] struct (with the positive direction being
    /// clockwise). As such, this should not be used for doing any kind of math in tandem with the GPS sensor's
    /// position readings. Prefer using [`GpsSensor::pose`] for that.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn rotation(&self) -> Result<f64, PortError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceGpsHeadingGet(self.device) } - self.rotation_offset)
    }

    /// Returns the Euler angles (pitch, yaw, roll) representing the IMU's orientation.
    ///
    /// # Important
    ///
    /// This value does not take into account the initial heading passed into [`GpsSensor::new`], and additionally
    /// uses a different angle system compared to the main [`GpsSensor`] struct (with the positive direction being
    /// clockwise). As such, this should not be used for doing any kind of math in tandem with the GPS sensor's
    /// position readings. Prefer using [`GpsSensor::pose`] for that.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
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

    /// Returns a quaternion representing the IMU's orientation.
    ///
    /// # Important
    ///
    /// This value does not take into account the initial heading passed into [`GpsSensor::new`], and additionally
    /// uses a different angle system compared to the main [`GpsSensor`] struct (with the positive direction being
    /// clockwise). As such, this should not be used for doing any kind of math in tandem with the GPS sensor's
    /// position readings. Prefer using [`GpsSensor::pose`] for that.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
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

    /// Returns the IMU's raw accelerometer values.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
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

    /// Returns the IMU's raw gyroscope values.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
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

    /// Resets the current reading of the IMU's heading to zero.
    ///
    /// # Important
    ///
    /// This has no effect on the "heading" value returned by [`GpsSensor::pose`]. See the notes
    /// on that function for more information.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn reset_heading(&mut self) -> Result<(), PortError> {
        self.set_heading(Default::default())
    }

    /// Resets the current reading of the IMU's rotation to zero.
    ///
    /// # Important
    ///
    /// This has no effect on the "heading" value returned by [`GpsSensor::pose`]. See the notes
    /// on that function for more information.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn reset_rotation(&mut self) -> Result<(), PortError> {
        self.set_rotation(Default::default())
    }

    /// Sets the current reading of the IMU's rotation to target value.
    ///
    /// # Important
    ///
    /// This has no effect on the "heading" value returned by [`GpsSensor::pose`]. See the notes
    /// on that function for more information.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn set_rotation(&mut self, rotation: f64) -> Result<(), PortError> {
        self.validate_port()?;

        self.rotation_offset = rotation - unsafe { vexDeviceGpsHeadingGet(self.device) };

        Ok(())
    }

    /// Sets the current reading of the IMU's heading to target value.
    ///
    /// Target will default to 360 if above 360 and default to 0 if below 0.
    ///
    /// # Important
    ///
    /// This has no effect on the "heading" value returned by [`GpsSensor::pose`]. See the notes
    /// on that function for more information.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn set_heading(&mut self, heading: f64) -> Result<(), PortError> {
        self.validate_port()?;

        self.heading_offset = heading - unsafe { vexDeviceGpsDegreesGet(self.device) };

        Ok(())
    }

    /// Sets the computation speed of the IMU.
    ///
    /// # Errors
    ///
    /// An error is returned if a GPS sensor is not currently connected to the Smart Port.
    pub fn set_computation_interval(&mut self, interval: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe {
            vexDeviceGpsDataRateSet(self.device, interval.as_millis() as u32);
        }

        Ok(())
    }
}
