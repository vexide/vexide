//! Inertial Sensor (IMU)
//!
//! This module provides an interface to interact with the V5 Inertial Sensor,
//! which combines a 3-axis accelerometer and 3-axis gyroscope for precise motion tracking
//! and navigation capabilities.
//!
//! # Hardware Overview
//!
//! The IMU's integrated accelerometer measures linear acceleration along three axes:
//! - X-axis: Forward/backward motion
//! - Y-axis: Side-to-side motion
//! - Z-axis: Vertical motion
//!
//! These accelerometer readings include the effect of gravity, which can be useful for
//! determining the sensor's orientation relative to the ground.
//!
//! The IMU also has a gyroscope that measures rotational velocity and position on three axes:
//! - Roll: Rotation around X-axis
//! - Pitch: Rotation around Y-axis
//! - Yaw: Rotation around Z-axis
//!
//! Like all other Smart devices, VEXos will process sensor updates every 10mS.
//!
//! # Coordinate System
//!
//! The IMU uses a NED (North-East-Down) right-handed coordinate system:
//! - X-axis: Positive towards the front of the robot (North)
//! - Y-axis: Positive towards the right of the robot (East)
//! - Z-axis: Positive downwards (towards the ground)
//!
//! This NED convention means that when the robot is on a flat surface:
//! - The Z acceleration will read approximately +9.81 m/s² (gravity)
//! - Positive roll represents clockwise rotation around the X-axis (when looking North)
//! - Positive pitch represents nose-down rotation around the Y-axis
//! - Positive yaw represents clockwise rotation around the Z-axis (when viewed from above)
//!
//! # Calibration & Mounting Considerations
//!
//! The IMU requires a calibration period to establish its reference frame in one of six
//! possible orientations (described by [`InertialOrientation`]). The sensor must be mounted
//! flat in one of these orientations. Readings will be unpredictable if the IMU is mounted at
//! an angle or was moving/disturbed during calibration.
//!
//! In addition, physical pressure on the sensor's housing or static electricity can cause issues
//! with the onboard gyroscope, so pressure-mounting the IMU or placing the IMU low to the ground
//! is undesirable.
//!
//! # Disconnect Behavior
//!
//! If the IMU loses power due to a disconnect — even momentarily, all calibration data will be lost
//! and VEXos will re-initiate calibration automatically. The robot cannot be moving when this occurs
//! due to the aforementioned unpredictable behavior. As such, it is vital that the IMU maintain a stable
//! connection to the Brain and voltage supply during operation.

use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bitflags::bitflags;
use snafu::{ensure, Snafu};
use vex_sdk::{
    vexDeviceImuAttitudeGet, vexDeviceImuDataRateSet, vexDeviceImuDegreesGet,
    vexDeviceImuHeadingGet, vexDeviceImuQuaternionGet, vexDeviceImuRawAccelGet,
    vexDeviceImuRawGyroGet, vexDeviceImuReset, vexDeviceImuStatusGet, V5ImuOrientationMode,
    V5_DeviceImuAttitude, V5_DeviceImuQuaternion, V5_DeviceImuRaw, V5_DeviceT,
};
use vexide_core::{float::Float, time::Instant};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::{
    math::{EulerAngles, Quaternion, Vector3},
    PortError,
};

/// An inertial sensor (IMU) plugged into a Smart Port.
#[derive(Debug, PartialEq)]
pub struct InertialSensor {
    port: SmartPort,
    device: V5_DeviceT,
    rotation_offset: f64,
    heading_offset: f64,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for InertialSensor {}
unsafe impl Sync for InertialSensor {}

impl InertialSensor {
    /// The maximum time that the Inertial Sensor should take to *begin* its calibration process following
    /// a call to [`InertialSensor::calibrate`].
    pub const CALIBRATION_START_TIMEOUT: Duration = Duration::from_secs(1);

    /// The maximum time that the Inertial Sensor should take to *end* its calibration process after
    /// calibration has begun.
    pub const CALIBRATION_END_TIMEOUT: Duration = Duration::from_secs(3);

    /// The minimum data rate that you can set an IMU to run at.
    pub const MIN_DATA_INTERVAL: Duration = Duration::from_millis(5);

    /// The maximum value that can be returned by [`Self::heading`].
    pub const MAX_HEADING: f64 = 360.0;

    /// Create a new inertial sensor from a [`SmartPort`].
    ///
    /// # Important
    ///
    /// <section class="warning">
    ///
    /// This sensor must be calibrated using [`InertialSensor::calibrate`] before any meaningful data
    /// can be read from it.
    ///
    /// </section>
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = InertialSensor::new(peripherals.port_1);
    /// }
    /// ```
    #[must_use]
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
            rotation_offset: 0.0,
            heading_offset: 0.0,
        }
    }

    /// Validates that the sensor is currently connected to its port, and that it isn't currently
    /// calibrating.
    fn validate(&self) -> Result<(), InertialError> {
        ensure!(!self.is_calibrating()?, StillCalibratingSnafu);
        Ok(())
    }

    /// Returns the internal status code of the inertial sensor.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
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
    ///     if let Ok(status) = sensor.status() {
    ///         println!("Status: {:b}", status.bits());
    ///     }
    /// }
    /// ```
    pub fn status(&self) -> Result<InertialStatus, InertialError> {
        self.validate_port()?;

        let bits = unsafe { vexDeviceImuStatusGet(self.device) };

        ensure!(bits != InertialStatus::STATUS_ERROR, BadStatusSnafu);

        Ok(InertialStatus::from_bits_retain(bits))
    }

    /// Returns `true` if the sensor is currently calibrating.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
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
    ///     // We haven't calibrated yet, so this is expected.
    ///     if sensor.is_calibrating() == Ok(false) {
    ///         println!("Sensor is not currently calibrating.");
    ///     }
    /// }
    /// ```
    pub fn is_calibrating(&self) -> Result<bool, InertialError> {
        Ok(self.status()?.contains(InertialStatus::CALIBRATING))
    }

    /// Returns `true` if the sensor was calibrated using auto-calibration.
    ///
    /// In some cases (such as a loss of power), VEXos will automatically decide to recalibrate the inertial sensor.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
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
    ///     if sensor.is_auto_calibrated() == Ok(true) {
    ///         println!("Sensor was automatically calibrated by VEXos.");
    ///     }
    /// }
    /// ```
    pub fn is_auto_calibrated(&self) -> Result<bool, InertialError> {
        Ok(self.status()?.contains(InertialStatus::AUTO_CALIBRATED))
    }

    /// Returns the physical orientation of the sensor as it was measured during calibration.
    ///
    /// This orientation can be one of six possible orientations aligned to two cardinal directions.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     if sensor.calibrate().await.is_ok() {
    ///         if let Ok(orientation) = sensor.physical_orientation() {
    ///             println!("Sensor was calibrated while facing: {:?}", orientation);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn physical_orientation(&self) -> Result<InertialOrientation, InertialError> {
        Ok(self.status()?.physical_orientation())
    }

    /// Calibrates the IMU.
    ///
    /// Returns an [`InertialCalibrateFuture`] that resolves once the calibration operation has finished or timed out.
    ///
    /// This method MUST be called for any meaningful gyroscope readings to be obtained. Calibration requires
    /// the sensor to be sitting completely still. If the sensor is moving during the calibration process,
    /// readings will drift from reality over time.
    ///
    /// # Errors
    ///
    /// - Calibration has a 1-second start timeout (when waiting for calibration to actually start on the sensor) and
    ///   a 3-second end timeout (when waiting for calibration to complete after it has started) as a failsafe in the
    ///   event that something goes wrong and the sensor gets stuck in a calibrating state. If either timeout
    ///   is exceeded in its respective phase of calibration, [`InertialError::CalibrationTimedOut`] will be returned.
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    ///
    /// # Examples
    ///
    /// Calibration process with error handling and a retry:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     match sensor.calibrate().await {
    ///         Ok(_) => println!("IMU calibrated successfully."),
    ///         Err(err) => {
    ///             println!("IMU failed to calibrate, retrying. Reason: {:?}", err);
    ///
    ///             // Since calibration failed, let's try one more time. If that fails,
    ///             // we just ignore the error and go on with our lives.
    ///             _ = sensor.calibrate().await;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Calibrating in a competition environment:
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use core::time::Duration;
    ///
    /// struct Robot {
    ///     imu: InertialSensor,
    /// }
    ///
    /// impl Compete for Robot {
    ///     async fn autonomous(&mut self) {
    ///         loop {
    ///             if let Ok(heading) = self.imu.heading() {
    ///                 println!("IMU Heading: {heading}°");
    ///             }
    ///
    ///             sleep(Duration::from_millis(10)).await;
    ///         }
    ///     }
    /// }
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut imu = InertialSensor::new(peripherals.port_1);
    ///
    ///     if let Err(err) = imu.calibrate().await {
    ///         // Log out a warning to terminal if calibration failed. You can also retry by
    ///         // calling it again, although this usually only happens if the sensor was unplugged.
    ///         println!("WARNING: IMU failed to calibrate! Readings might be inaccurate!");
    ///     }
    ///
    ///     Robot { imu }.compete().await;
    /// }
    /// ```
    pub const fn calibrate(&mut self) -> InertialCalibrateFuture<'_> {
        InertialCalibrateFuture {
            state: InertialCalibrateFutureState::Calibrate,
            imu: self,
        }
    }

    /// Returns the total number of degrees the Inertial Sensor has spun about the z-axis.
    ///
    /// This value is theoretically unbounded. Clockwise rotations are represented with positive degree values,
    /// while counterclockwise rotations are represented with negative ones.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Calibrate sensor, panic if calibration fails.
    ///     sensor.calibrate().await.unwrap();
    ///
    ///     // Sleep for two seconds to allow the robot to be moved.
    ///     sleep(Duration::from_secs(2)).await;
    ///
    ///     if let Ok(rotation) = sensor.rotation() {
    ///         println!("Robot has rotated {} degrees since calibration.", rotation);
    ///     }
    /// }
    /// ```
    pub fn rotation(&self) -> Result<f64, InertialError> {
        self.validate()?;
        Ok(unsafe { vexDeviceImuHeadingGet(self.device) } + self.rotation_offset)
    }

    /// Returns the Inertial Sensor’s yaw angle bounded from [0.0, 360.0) degrees.
    ///
    /// Clockwise rotations are represented with positive degree values, while counterclockwise rotations are
    /// represented with negative ones.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Calibrate sensor, panic if calibration fails.
    ///     sensor.calibrate().await.unwrap();
    ///
    ///     // Sleep for two seconds to allow the robot to be moved.
    ///     sleep(Duration::from_secs(2)).await;
    ///
    ///     if let Ok(heading) = sensor.heading() {
    ///         println!("Heading is {} degrees.", rotation);
    ///     }
    /// }
    /// ```
    pub fn heading(&self) -> Result<f64, InertialError> {
        self.validate()?;
        // The result needs to be [0, 360). Adding a significantly negative offset could take us
        // below 0. Adding a significantly positive offset could take us above 360.
        Ok(
            (unsafe { vexDeviceImuDegreesGet(self.device) } + self.heading_offset)
                .rem_euclid(Self::MAX_HEADING),
        )
    }

    /// Returns a quaternion representing the Inertial Sensor’s current orientation.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Calibrate sensor, panic if calibration fails.
    ///     sensor.calibrate().await.unwrap();
    ///
    ///     // Sleep for two seconds to allow the robot to be moved.
    ///     sleep(Duration::from_secs(2)).await;
    ///
    ///     if let Ok(quaternion) = sensor.quaternion() {
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
    pub fn quaternion(&self) -> Result<Quaternion<f64>, InertialError> {
        self.validate()?;

        let mut data = V5_DeviceImuQuaternion::default();
        unsafe {
            vexDeviceImuQuaternionGet(self.device, &mut data);
        }

        Ok(Quaternion {
            v: Vector3 {
                x: data.a,
                y: data.b,
                z: data.c,
            },
            s: data.d,
        })
    }

    /// Returns the Euler angles (pitch, yaw, roll) in radians representing the Inertial Sensor’s orientation.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Calibrate sensor, panic if calibration fails.
    ///     sensor.calibrate().await.unwrap();
    ///
    ///     // Sleep for two seconds to allow the robot to be moved.
    ///     sleep(Duration::from_secs(2)).await;
    ///
    ///     if let Ok(angles) = sensor.euler() {
    ///         println!(
    ///             "yaw: {}°, pitch: {}°, roll: {}°",
    ///             angles.a.to_degrees(),
    ///             angles.b.to_degrees(),
    ///             angles.c.to_degrees(),
    ///         );
    ///     }
    /// }
    /// ```
    pub fn euler(&self) -> Result<EulerAngles<f64, f64>, InertialError> {
        self.validate()?;

        let mut data = V5_DeviceImuAttitude::default();
        unsafe {
            vexDeviceImuAttitudeGet(self.device, &mut data);
        }

        Ok(EulerAngles {
            a: data.pitch.to_radians(),
            b: data.yaw.to_radians(),
            c: data.roll.to_radians(),
            marker: PhantomData,
        })
    }

    /// Returns the Inertial Sensor’s raw gyroscope readings in dps (degrees per second).
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Calibrate sensor, panic if calibration fails.
    ///     sensor.calibrate().await.unwrap();
    ///
    ///     // Read out angular velocity values every 10mS
    ///     loop {
    ///         if let Ok(rates) = sensor.gyro_rate() {
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
    pub fn gyro_rate(&self) -> Result<Vector3<f64>, InertialError> {
        self.validate()?;

        let mut data = V5_DeviceImuRaw::default();
        unsafe {
            vexDeviceImuRawGyroGet(self.device, &mut data);
        }

        Ok(Vector3 {
            x: data.x,
            y: data.y,
            z: data.z,
            // NOTE: data.w is unused in the SDK.
            // See: <https://github.com/purduesigbots/pros/blob/master/src/devices/vdml_imu.c#L239C63-L239C64>
        })
    }

    /// Returns the sensor's raw acceleration readings in g (multiples of ~9.8 m/s/s).
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Calibrate sensor, panic if calibration fails.
    ///     sensor.calibrate().await.unwrap();
    ///
    ///     // Read out acceleration values every 10mS
    ///     loop {
    ///         if let Ok(acceleration) = sensor.acceleration() {
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
    pub fn acceleration(&self) -> Result<Vector3<f64>, InertialError> {
        self.validate()?;

        let mut data = V5_DeviceImuRaw::default();
        unsafe {
            vexDeviceImuRawAccelGet(self.device, &mut data);
        }

        Ok(Vector3 {
            x: data.x,
            y: data.y,
            z: data.z,
            // NOTE: data.w is unused in the SDK.
            // See: <https://github.com/purduesigbots/pros/blob/master/src/devices/vdml_imu.c#L239C63-L239C64>
        })
    }

    /// Resets the current reading of the sensor's heading to zero.
    ///
    /// This only affects the value returned by [`InertialSensor::heading`] and does not effect [`InertialSensor::rotation`]
    /// or [`InertialSensor::euler`]/[`InertialSensor::quaternion`].
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Calibrate sensor, panic if calibration fails.
    ///     sensor.calibrate().await.unwrap();
    ///
    ///     // Sleep for two seconds to allow the robot to be moved.
    ///     sleep(Duration::from_secs(2)).await;
    ///
    ///     // Store heading before reset.
    ///     let heading = sensor.heading().unwrap_or_default();
    ///
    ///     // Reset heading back to zero.
    ///     _ = sensor.reset_heading();
    /// }
    /// ```
    pub fn reset_heading(&mut self) -> Result<(), InertialError> {
        self.set_heading(Default::default())
    }

    /// Resets the current reading of the sensor's rotation to zero.
    ///
    /// This only affects the value returned by [`InertialSensor::rotation`] and does not effect [`InertialSensor::heading`]
    /// or [`InertialSensor::euler`]/[`InertialSensor::quaternion`].
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    /// use core::time::Duration;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Calibrate sensor, panic if calibration fails.
    ///     sensor.calibrate().await.unwrap();
    ///
    ///     // Sleep for two seconds to allow the robot to be moved.
    ///     sleep(Duration::from_secs(2)).await;
    ///
    ///     // Store rotation before reset.
    ///     let rotation = sensor.rotation().unwrap_or_default();
    ///
    ///     // Reset heading back to zero.
    ///     _ = sensor.reset_rotation();
    /// }
    /// ```
    pub fn reset_rotation(&mut self) -> Result<(), InertialError> {
        self.set_rotation(Default::default())
    }

    /// Sets the current reading of the sensor's rotation to a given value.
    ///
    /// This only affects the value returned by [`InertialSensor::rotation`] and does not effect [`InertialSensor::heading`]
    /// or [`InertialSensor::euler`]/[`InertialSensor::quaternion`].
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Set rotation to 90 degrees clockwise.
    ///     _ = sensor.set_rotation(90.0);
    /// }
    /// ```
    pub fn set_rotation(&mut self, rotation: f64) -> Result<(), InertialError> {
        self.validate()?;

        self.rotation_offset = rotation - unsafe { vexDeviceImuHeadingGet(self.device) };

        Ok(())
    }

    /// Sets the current reading of the sensor's heading to a given value.
    ///
    /// This only affects the value returned by [`InertialSensor::heading`] and does not effect [`InertialSensor::rotation`]
    /// or [`InertialSensor::euler`]/[`InertialSensor::quaternion`].
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Set heading to 90 degrees clockwise.
    ///     _ = sensor.set_heading(90.0);
    /// }
    /// ```
    pub fn set_heading(&mut self, heading: f64) -> Result<(), InertialError> {
        self.validate()?;

        self.heading_offset = heading - unsafe { vexDeviceImuDegreesGet(self.device) };

        Ok(())
    }

    /// Sets the internal computation speed of the IMU.
    ///
    /// This method does NOT change the rate at which user code can read data off the IMU, as the brain will only talk to the
    /// device every 10mS regardless of how fast data is being sent or computed. See [`InertialSensor::UPDATE_INTERVAL`].
    ///
    /// This duration should be above [`Self::MIN_DATA_INTERVAL`] (5 milliseconds).
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = InertialSensor::new(peripherals.port_1);
    ///
    ///     // Set to minimum interval.
    ///     _ = sensor.set_data_interval(InertialSensor::MIN_DATA_INTERVAL);
    /// }
    /// ```
    pub fn set_data_interval(&mut self, interval: Duration) -> Result<(), InertialError> {
        self.validate()?;

        let mut time_ms = interval
            .as_millis()
            .max(Self::MIN_DATA_INTERVAL.as_millis()) as u32;
        time_ms -= time_ms % 5; // Rate is in increments of 5ms - not sure if this is necessary, but PROS does it.

        unsafe { vexDeviceImuDataRateSet(self.device, time_ms) }

        Ok(())
    }
}

impl SmartDevice for InertialSensor {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Imu
    }
}
impl From<InertialSensor> for SmartPort {
    fn from(device: InertialSensor) -> Self {
        device.port
    }
}

/// Represents one of six possible physical IMU orientations relative
/// to the earth's center of gravity.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InertialOrientation {
    /// Z-Axis facing up (VEX logo facing DOWN).
    ZUp,

    /// Z-Axis facing down (VEX logo facing UP).
    ZDown,

    /// X-axis facing up.
    XUp,

    /// X-axis facing down.
    XDown,

    /// Y-axis facing up.
    YUp,

    /// Y-axis facing down.
    YDown,
}

impl From<InertialOrientation> for V5ImuOrientationMode {
    fn from(value: InertialOrientation) -> Self {
        match value {
            InertialOrientation::ZUp => Self::kImuOrientationZUp,
            InertialOrientation::ZDown => Self::kImuOrientationZDown,
            InertialOrientation::XUp => Self::kImuOrientationXUp,
            InertialOrientation::XDown => Self::kImuOrientationXDown,
            InertialOrientation::YUp => Self::kImuOrientationYUp,
            InertialOrientation::YDown => Self::kImuOrientationYDown,
        }
    }
}

bitflags! {
    /// The status bits returned by an [`InertialSensor`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct InertialStatus: u32 {
        /// The sensor is currently calibrating.
        const CALIBRATING = 0b00001;

        /// The sensor is calibrated using auto-calibration.
        const AUTO_CALIBRATED = 0b10000;
    }
}

impl InertialStatus {
    /// The return value of [`vexDeviceImuStatusGet`] when the device fails to report its
    /// status bits.
    pub const STATUS_ERROR: u32 = 0xFF;

    /// Returns the physical orientation of the sensor measured at calibration.
    ///
    /// This orientation can be one of six possible orientations aligned to two cardinal directions.
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
    ///     if sensor.calibrate().await.is_ok() {
    ///         if let Ok(status) = sensor.status() {
    ///             println!("Sensor was calibrated while facing: {:?}", status.physical_orientation());
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn physical_orientation(&self) -> InertialOrientation {
        match (self.bits() >> 1) & 0b111 {
            0 => InertialOrientation::ZUp,
            1 => InertialOrientation::ZDown,
            2 => InertialOrientation::XUp,
            3 => InertialOrientation::XDown,
            4 => InertialOrientation::YUp,
            5 => InertialOrientation::YDown,
            _ => unreachable!(),
        }
    }
}

/// Defines a waiting phase in [`InertialCalibrateFuture`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CalibrationPhase {
    /// Waiting for IMU to report its status as something other than 0x0.
    ///
    /// This can happen at the start of the program where VEXos takes a bit to
    /// give us the first sensor packet.
    Status,

    /// Future is currently waiting for the IMU to report [`InertialStatus::CALIBRATING`], indicating
    /// that it has started calibration.
    Start,

    /// Waiting for calibration to end ([`InertialStatus::CALIBRATING`] to be cleared from status).
    End,
}

#[derive(Debug, Clone, Copy)]
enum InertialCalibrateFutureState {
    /// Calibrate the IMU
    Calibrate,
    /// Wait for the IMU to either begin calibrating or end calibration, depending on the
    /// designated [`CalibrationPhase`].
    Waiting(Instant, CalibrationPhase),
}

/// Future that calibrates an IMU
/// created with [`InertialSensor::calibrate`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug)]
pub struct InertialCalibrateFuture<'a> {
    state: InertialCalibrateFutureState,
    imu: &'a mut InertialSensor,
}

impl core::future::Future for InertialCalibrateFuture<'_> {
    type Output = Result<(), InertialError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let device = unsafe { this.imu.port.device_handle() };

        // Get the sensor's status flags, which tell us whether or not we are still calibrating.
        let status = InertialStatus::from_bits_retain(if let Err(err) = this.imu.validate_port() {
            // IMU got unplugged, so we'll resolve early.
            return Poll::Ready(Err(err.into()));
        } else {
            // Get status flags from VEXos.
            let flags = unsafe { vexDeviceImuStatusGet(device) };

            // 0xFF is returned when the sensor fails to report flags.
            if flags == InertialStatus::STATUS_ERROR {
                return Poll::Ready(BadStatusSnafu.fail());
            } else if flags == 0x0 {
                this.state =
                    InertialCalibrateFutureState::Waiting(Instant::now(), CalibrationPhase::Status);
            }

            flags
        });

        match this.state {
            // The "calibrate" phase begins the calibration process.
            //
            // This only happens for one poll of the future (the first one). All future polls will
            // either be waiting for calibration to start or for calibration to end.
            InertialCalibrateFutureState::Calibrate => {
                // Check if the sensor was already calibrating before we recalibrate it ourselves.
                //
                // This can happen at the start of program execution or if the sensor loses then regains power.
                // In those instances, VEXos will automatically start the calibration process without us asking.
                // Calling [`vexDeviceImuReset`] while calibration is already happening has caused bugs in our
                // testing, so we instead just want to wait until the calibration attempt has finished.
                //
                // See <https://github.com/vexide/vexide/issues/253> for more details.
                if status.contains(InertialStatus::CALIBRATING) {
                    // Sensor was already calibrating, so wait for that to finish.
                    this.state = InertialCalibrateFutureState::Waiting(
                        Instant::now(),
                        CalibrationPhase::End,
                    );
                } else {
                    // Request that VEXos calibrate the IMU, and transition to pending state.
                    unsafe { vexDeviceImuReset(device) }

                    // Change to waiting for calibration to start.
                    this.state = InertialCalibrateFutureState::Waiting(
                        Instant::now(),
                        CalibrationPhase::Start,
                    );
                }

                cx.waker().wake_by_ref();
                Poll::Pending
            }

            // In this stage, we are either waiting for the calibration status flag to be set (CalibrationPhase::Start),
            // indicating that calibration has begun, or we are waiting for the calibration status flag to be cleared,
            // indicating that calibration has finished (CalibrationFlag::End).
            InertialCalibrateFutureState::Waiting(timestamp, phase) => {
                if timestamp.elapsed()
                    > match phase {
                        CalibrationPhase::Start | CalibrationPhase::Status => {
                            InertialSensor::CALIBRATION_START_TIMEOUT
                        }
                        CalibrationPhase::End => InertialSensor::CALIBRATION_END_TIMEOUT,
                    }
                {
                    // Waiting took too long and exceeded a timeout.
                    return Poll::Ready(CalibrationTimedOutSnafu.fail());
                }

                if status.contains(InertialStatus::CALIBRATING) && phase == CalibrationPhase::Start
                {
                    // We are in the "start" phase (waiting for the flag to be set) and the flag is now set,
                    // meaning that calibration has begun.
                    //
                    // We now know that the sensor is actually calibrating, so we transition to
                    // [`CalibrationPhase::End`] and reset the timeout timestamp to wait for calibration to finish.
                    this.state = InertialCalibrateFutureState::Waiting(
                        Instant::now(),
                        CalibrationPhase::End,
                    );
                } else if !status.is_empty() && phase == CalibrationPhase::Status {
                    this.state = InertialCalibrateFutureState::Calibrate;
                } else if !status.contains(InertialStatus::CALIBRATING)
                    && phase == CalibrationPhase::End
                {
                    // The [`InertialStatus::CALIBRATING`] has been cleared, indicating that calibration is complete.
                    return Poll::Ready(Ok(()));
                }

                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

/// Errors that can occur when interacting with an Inertial Sensor.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Snafu)]
pub enum InertialError {
    /// The sensor took longer than three seconds to calibrate.
    CalibrationTimedOut,
    /// The sensor is still calibrating.
    StillCalibrating,
    /// The sensor failed to report its status flags (returned 0xFF).
    BadStatus,
    /// Generic port related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
