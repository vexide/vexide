//! GPS Sensor
//!
//! This module provides an interface to interact with the VEX V5 Game Position System (GPS)
//! Sensor, which uses computer vision and an inertial measurement unit (IMU) to provide
//! absolute position tracking within a VEX Robotics Competition field.
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

use mint::{EulerAngles, IntraZYX, Quaternion, Vector3};
use vex_sdk::{
    vexDeviceGpsAttitudeGet, vexDeviceGpsDataRateSet, vexDeviceGpsDegreesGet, vexDeviceGpsErrorGet,
    vexDeviceGpsHeadingGet, vexDeviceGpsInitialPositionSet, vexDeviceGpsOriginGet,
    vexDeviceGpsOriginSet, vexDeviceGpsQuaternionGet, vexDeviceGpsRawAccelGet,
    vexDeviceGpsRawGyroGet, vexDeviceGpsStatusGet, V5_DeviceGpsAttitude, V5_DeviceGpsQuaternion,
    V5_DeviceGpsRaw, V5_DeviceT,
};

use super::{PortError, SmartDevice, SmartDeviceType, SmartPort};
use crate::math::{Angle, Point2};

/// A GPS sensor plugged into a Smart Port.
#[derive(Debug, PartialEq)]
pub struct GpsSensor {
    port: SmartPort,
    device: V5_DeviceT,
    rotation_offset: f64,
    heading_offset: f64,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for GpsSensor {}
unsafe impl Sync for GpsSensor {}

impl GpsSensor {
    /// Creates a new GPS sensor from a [`SmartPort`].
    ///
    /// # Sensor Configuration
    ///
    /// The sensor requires three measurements to be made at the start of a match, passed as arguments to this function:
    ///
    /// ## Sensor Offset
    ///
    /// `offset` is the physical offset of the sensor's mounting location from a reference point on the robot.
    ///
    /// Offset defines the exact point on the robot that is considered a "source of truth" for the robot's position.
    /// For example, if you considered the center of your robot to be the reference point for coordinates, then this
    /// value would be the signed 4-quadrant x and y offset from that point on your robot in meters. Similarly, if you
    /// considered the sensor itself to be the robot's origin of tracking, then this value would simply be
    /// `Point2 { x: 0.0, y: 0.0 }`.
    ///
    /// ## Initial Robot Position
    ///
    /// `initial_position` is an estimate of the robot's initial cartesian coordinates on the field in meters. This
    /// value helpful for cases when the robot's starting point is near a field wall.
    ///
    /// When the GPS Sensor is too close to a field wall to properly read the GPS strips, the sensor will be unable
    /// to localize the robot's position due the wall's proximity limiting the view of the camera. This can cause the
    /// sensor inaccurate results at the start of a match, where robots often start directly near a wall.
    ///
    /// By providing an estimate of the robot's initial position on the field, this problem is partially mitigated by
    /// giving the sensor an initial frame of reference to use.
    ///
    /// # Initial Robot Heading
    ///
    /// `initial_heading` is a value between 0 and 360 degrees that informs the GPS of its heading at the start of the
    /// match. Similar to `initial_position`, this is useful for improving accuracy when the sensor is in close proximity
    /// to a field wall, as the sensor's rotation values are continuously checked against the GPS field strips to prevent
    /// drift over time. If the sensor starts too close to a field wall, providing an `initial_heading` can help prevent
    /// this drift at the start of the match.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Create a GPS sensor mounted 2 inches forward and 1 inch right of center
    ///     // Starting at position (0, 0) with 90 degree heading
    ///     let gps = GpsSensor::new(
    ///         // Port 1
    ///         peripherals.port_1,
    ///
    ///         // Sensor is mounted 0.225 meters to the left and 0.225 meters above the robot's tracking origin.
    ///         Point2 { x: -0.225, y: 0.225 },
    ///
    ///         // Robot's starting point is at the center of the field.
    ///         Point2 { x: 0.0, y: 0.0 },
    ///
    ///         // Robot is facing to the right initially.
    ///         90.0,
    ///     );
    /// }
    /// ```
    pub fn new(
        port: SmartPort,
        offset: impl Into<Point2<f64>>,
        initial_position: impl Into<Point2<f64>>,
        initial_heading: f64,
    ) -> Self {
        let device = unsafe { port.device_handle() };

        let initial_position = initial_position.into();
        let offset = offset.into();

        unsafe {
            vexDeviceGpsOriginSet(device, offset.x, offset.y);
            vexDeviceGpsInitialPositionSet(
                device,
                initial_position.x,
                initial_position.y,
                initial_heading,
            );
        }

        Self {
            device,
            port,
            rotation_offset: Default::default(),
            heading_offset: Default::default(),
        }
    }

    /// Returns the user-configured offset from a reference point on the robot.
    ///
    /// This offset value is passed to [`GpsSensor::new`] and can be changed using [`GpsSensor::set_offset`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///
    ///         // Initial offset value is configured here!
    ///         //
    ///         // Let's assume that the sensor is mounted 0.225 meters to the left and 0.225 meters above
    ///         // our desired tracking origin.
    ///         Point2 { x: -0.225, y: 0.225 }, // Configure offset value
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         90.0,
    ///     );
    ///
    ///     // Get the configured offset of the sensor
    ///     if let Ok(offset) = gps.offset() {
    ///         println!("GPS sensor is mounted at x={}, y={}", offset.x, offset.y); // "Sensor is mounted at x=-0.225, y=0.225"
    ///     }
    ///
    ///     // Change the offset to something new
    ///     _ = gps.set_offset(Point2 { x: 0.0, y: 0.0 });
    ///
    ///     // Get the configured offset of the sensor again
    ///     if let Ok(offset) = gps.offset() {
    ///         println!("GPS sensor is mounted at x={}, y={}", offset.x, offset.y); // "Sensor is mounted at x=0.0, y=0.0"
    ///     }
    /// }
    /// ```
    pub fn offset(&self) -> Result<Point2<f64>, PortError> {
        self.validate_port()?;

        let mut data = Point2 { x: 0.0, y: 0.0 };
        unsafe { vexDeviceGpsOriginGet(self.device, &raw mut data.x, &raw mut data.y) }

        Ok(data)
    }

    /// Adjusts the sensor's physical offset from the robot's tracking origin.
    ///
    /// This value is also configured initially through [`GpsSensor::new`].
    ///
    /// Offset defines the exact point on the robot that is considered a "source of truth" for the robot's position.
    /// For example, if you considered the center of your robot to be the reference point for coordinates, then this
    /// value would be the signed 4-quadrant x and y offset from that point on your robot in meters. Similarly, if you
    /// considered the sensor itself to be the robot's origin of tracking, then this value would simply be
    /// `Point2 { x: 0.0, y: 0.0 }`.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///
    ///         // Initial offset value is configured here!
    ///         //
    ///         // Let's assume that the sensor is mounted 0.225 meters to the left and 0.225 meters above
    ///         // our desired tracking origin.
    ///         Point2 { x: -0.225, y: 0.225 }, // Configure offset value
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         90.0,
    ///     );
    ///
    ///     // Get the configured offset of the sensor
    ///     if let Ok(offset) = gps.offset() {
    ///         println!("GPS sensor is mounted at x={}, y={}", offset.x, offset.y); // "Sensor is mounted at x=-0.225, y=0.225"
    ///     }
    ///
    ///     // Change the offset to something new
    ///     _ = gps.set_offset(Point2 { x: 0.0, y: 0.0 });
    ///
    ///     // Get the configured offset of the sensor again
    ///     if let Ok(offset) = gps.offset() {
    ///         println!("GPS sensor is mounted at x={}, y={}", offset.x, offset.y); // "Sensor is mounted at x=0.0, y=0.0"
    ///     }
    /// }
    /// ```
    pub fn set_offset(&mut self, offset: Point2<f64>) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe { vexDeviceGpsOriginSet(self.device, offset.x, offset.y) }

        Ok(())
    }

    /// Returns an estimate of the robot's location on the field as cartesian coordinates measured in meters.
    ///
    /// The reference point for a robot's position is determined by the sensor's configured [`offset`](`GpsSensor::offset`) value.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     // Get current position and heading
    ///     if let Ok(position) = gps.position() {
    ///         println!(
    ///             "Robot is at x={}, y={}",
    ///             position.x,
    ///             position.y,
    ///         );
    ///     }
    /// }
    /// ```
    pub fn position(&self) -> Result<Point2<f64>, PortError> {
        self.validate_port()?;

        let mut attitude = V5_DeviceGpsAttitude::default();
        unsafe {
            vexDeviceGpsAttitudeGet(self.device, &raw mut attitude, false);
        }

        Ok(Point2 {
            x: attitude.position_x,
            y: attitude.position_y,
        })
    }

    /// Returns the RMS (Root Mean Squared) error for the sensor's [position reading] in meters.
    ///
    /// [position reading]: GpsSensor::position
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     // Check position accuracy
    ///     if gps.error().is_ok_and(|err| err > 0.3) {
    ///         println!("Warning: GPS position accuracy is low ({}m error)", error);
    ///     }
    /// }
    /// ```
    pub fn error(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGpsErrorGet(self.device) })
    }

    /// Returns the internal status code of the sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     if let Ok(status) = gps.status() {
    ///         println!("Status: {:b}", status);
    ///     }
    /// }
    /// ```
    pub fn status(&self) -> Result<u32, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceGpsStatusGet(self.device) })
    }

    /// Returns the sensor's yaw angle bounded by [0.0, 360.0) degrees.
    ///
    /// Clockwise rotations are represented with positive degree values, while counterclockwise rotations are
    /// represented with negative ones. If a heading offset has not been set using [`GpsSensor::set_heading`],
    /// then 90 degrees will located to the right of the field.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     if let Ok(heading) = gps.heading() {
    ///         println!("Heading is {} degrees.", rotation);
    ///     }
    /// }
    /// ```
    pub fn heading(&self) -> Result<Angle, PortError> {
        self.validate_port()?;

        // The result needs to be [0, 360). Adding a significantly negative offset could take us
        // below 0. Adding a significantly positive offset could take us above 360.
        Ok(Angle::from_degrees(
            unsafe { vexDeviceGpsDegreesGet(self.device) } + self.heading_offset,
        )
        .wrapped_full())
    }

    /// Returns the total number of degrees the GPS has spun about the z-axis.
    ///
    /// This value is theoretically unbounded. Clockwise rotations are represented with positive degree values,
    /// while counterclockwise rotations are represented with negative ones. If a heading offset has not been set
    /// using [`GpsSensor::set_rotation`], then 90 degrees will located to the right of the field.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     if let Ok(rotation) = gps.rotation() {
    ///         println!("Robot has rotated {} degrees since calibration.", rotation);
    ///     }
    /// }
    /// ```
    pub fn rotation(&self) -> Result<Angle, PortError> {
        self.validate_port()?;
        Ok(Angle::from_degrees(
            unsafe { vexDeviceGpsHeadingGet(self.device) } + self.rotation_offset,
        ))
    }

    /// Returns the Euler angles (pitch, yaw, roll) representing the GPS's orientation.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     if let Ok(angles) = gps.euler() {
    ///         println!(
    ///             "yaw: {}°, pitch: {}°, roll: {}°",
    ///             angles.a.to_degrees(),
    ///             angles.b.to_degrees(),
    ///             angles.c.to_degrees(),
    ///         );
    ///     }
    /// }
    /// ```
    pub fn euler(&self) -> Result<EulerAngles<Angle, IntraZYX>, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceGpsAttitude::default();
        unsafe {
            vexDeviceGpsAttitudeGet(self.device, &raw mut data, false);
        }

        Ok(EulerAngles {
            a: Angle::from_degrees(data.pitch).wrapped_half(),
            b: Angle::from_degrees(data.yaw).wrapped_half(),
            c: Angle::from_degrees(data.roll).wrapped_half(),
            marker: PhantomData,
        })
    }

    /// Returns a quaternion representing the sensor's orientation.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     if let Ok(quaternion) = gps.quaternion() {
    ///         println!(
    ///             "x: {}, y: {}, z: {}, scalar: {}",
    ///             quaternion.v.x,
    ///             quaternion.v.y,
    ///             quaternion.v.z,
    ///             quaternion.s,
    ///         );
    ///     }
    /// }
    /// ```
    pub fn quaternion(&self) -> Result<Quaternion<f64>, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceGpsQuaternion::default();
        unsafe {
            vexDeviceGpsQuaternionGet(self.device, &raw mut data);
        }

        Ok(Quaternion {
            v: Vector3 {
                x: data.x,
                y: data.y,
                z: data.z,
            },
            s: data.w,
        })
    }

    /// Returns raw accelerometer values of the sensor's internal IMU.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     // Read out acceleration values every 10mS
    ///     loop {
    ///         if let Ok(acceleration) = gps.acceleration() {
    ///             println!(
    ///                 "x: {}G, y: {}G, z: {}G",
    ///                 acceleration.x,
    ///                 acceleration.y,
    ///                 acceleration.z,
    ///             );
    ///         }
    ///
    ///         sleep(Duration::from_millis(10)).await;
    ///     }
    /// }
    /// ```
    pub fn acceleration(&self) -> Result<Vector3<f64>, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceGpsRaw::default();
        unsafe {
            vexDeviceGpsRawAccelGet(self.device, &raw mut data);
        }

        Ok(mint::Vector3 {
            x: data.x,
            y: data.y,
            z: data.z,
        })
    }

    /// Returns the raw gyroscope values of the sensor's internal IMU.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     // Read out angular velocity values every 10mS
    ///     loop {
    ///         if let Ok(rates) = gps.gyro_rate() {
    ///             println!(
    ///                 "x: {}°/s, y: {}°/s, z: {}°/s",
    ///                 rates.x,
    ///                 rates.y,
    ///                 rates.z,
    ///             );
    ///         }
    ///
    ///         sleep(Duration::from_millis(10)).await;
    ///     }
    /// }
    /// ```
    pub fn gyro_rate(&self) -> Result<Vector3<f64>, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceGpsRaw::default();
        unsafe {
            vexDeviceGpsRawGyroGet(self.device, &raw mut data);
        }

        Ok(mint::Vector3 {
            x: data.x,
            y: data.y,
            z: data.z,
        })
    }

    /// Offsets the reading of [`GpsSensor::heading`] to zero.
    ///
    /// This method has no effect on the values returned by [`GpsSensor::position`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let mut gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     // Sleep for two seconds to allow the robot to be moved.
    ///     sleep(Duration::from_secs(2)).await;
    ///
    ///     // Store heading before reset.
    ///     let heading = gps.heading().unwrap_or_default();
    ///
    ///     // Reset heading back to zero.
    ///     _ = gps.reset_heading();
    /// }
    /// ```
    pub fn reset_heading(&mut self) -> Result<(), PortError> {
        self.set_heading(Angle::ZERO)
    }

    /// Offsets the reading of [`GpsSensor::rotation`] to zero.
    ///
    /// This method has no effect on the values returned by [`GpsSensor::position`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let mut gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     // Sleep for two seconds to allow the robot to be moved.
    ///     sleep(Duration::from_secs(2)).await;
    ///
    ///     // Store rotation before reset.
    ///     let rotation = gps.rotation().unwrap_or_default();
    ///
    ///     // Reset rotation back to zero.
    ///     _ = gps.reset_rotation();
    /// }
    /// ```
    pub fn reset_rotation(&mut self) -> Result<(), PortError> {
        self.set_rotation(Angle::ZERO)
    }

    /// Offsets the reading of [`GpsSensor::rotation`] to a specified angle value.
    ///
    /// This method has no effect on the values returned by [`GpsSensor::position`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let mut gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     // Set rotation to 90 degrees clockwise.
    ///     _ = gps.set_rotation(90.0);
    ///
    ///     println!("Rotation: {:?}", gps.rotation());
    /// }
    /// ```
    pub fn set_rotation(&mut self, rotation: Angle) -> Result<(), PortError> {
        self.validate_port()?;

        self.rotation_offset =
            rotation.as_degrees() - unsafe { vexDeviceGpsHeadingGet(self.device) };

        Ok(())
    }

    /// Offsets the reading of [`GpsSensor::heading`] to a specified angle value.
    ///
    /// Target will default to `360.0` if above `360.0` and default to `0.0` if below `0.0`.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // Assume we're starting in the middle of the field facing upwards, with the
    ///     // sensor's mounting point being our reference for position.
    ///     let mut gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     // Set heading to 90 degrees clockwise.
    ///     _ = gps.set_heading(90.0);
    ///
    ///     println!("Heading: {:?}", gps.heading());
    /// }
    /// ```
    pub fn set_heading(&mut self, heading: Angle) -> Result<(), PortError> {
        self.validate_port()?;

        self.heading_offset = heading.as_degrees() - unsafe { vexDeviceGpsDegreesGet(self.device) };

        Ok(())
    }

    /// Sets the internal computation speed of the sensor's internal IMU.
    ///
    /// This method does NOT change the rate at which user code can read data off the GPS, as the
    /// brain will only talk to the device every 10mS regardless of how fast data is being sent or
    /// computed. This also has no effect on the speed of methods such as `GpsSensor::position`, as
    /// it only changes the *internal* computation speed of the sensor's internal IMU.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was connected to the port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use std::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut gps = GpsSensor::new(
    ///         peripherals.port_1,
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         Point2 { x: 0.0, y: 0.0 },
    ///         0.0,
    ///     );
    ///
    ///     // Set to minimum interval.
    ///     _ = gps.set_data_interval(Duration::from_millis(5));
    /// }
    /// ```
    pub fn set_data_interval(&mut self, interval: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe {
            vexDeviceGpsDataRateSet(self.device, interval.as_millis() as u32);
        }

        Ok(())
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
