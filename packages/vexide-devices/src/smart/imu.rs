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
//! # Calibration
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
//! due to the aformentioned unpredictable behavior. As such, it is vital that the IMU maintain a stable
//! connection to the Brain and voltage supply during operation.

use core::{
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bitflags::bitflags;
use snafu::Snafu;
use vex_sdk::{
    vexDeviceGetByIndex, vexDeviceImuAttitudeGet, vexDeviceImuDataRateSet, vexDeviceImuDegreesGet,
    vexDeviceImuHeadingGet, vexDeviceImuQuaternionGet, vexDeviceImuRawAccelGet,
    vexDeviceImuRawGyroGet, vexDeviceImuReset, vexDeviceImuStatusGet, V5ImuOrientationMode,
    V5_DeviceImuAttitude, V5_DeviceImuQuaternion, V5_DeviceImuRaw, V5_DeviceT,
};
use vexide_core::time::Instant;

use super::{validate_port, SmartDevice, SmartDeviceType, SmartPort};
use crate::{
    geometry::{EulerAngles, Quaternion, Vector3},
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
        if self.is_calibrating()? {
            return Err(InertialError::StillCalibrating);
        }
        Ok(())
    }

    /// Returns the internal status code of the intertial sensor.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    pub fn status(&self) -> Result<InertialStatus, InertialError> {
        self.validate_port()?;

        let bits = unsafe { vexDeviceImuStatusGet(self.device) };

        if bits == InertialStatus::STATUS_ERROR {
            return Err(InertialError::BadStatus);
        }

        Ok(InertialStatus::from_bits_retain(bits))
    }

    /// Returns `true` if the sensor is currently calibrating.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    pub fn is_calibrating(&self) -> Result<bool, InertialError> {
        Ok(self.status()?.contains(InertialStatus::CALIBRATING))
    }

    /// Returns `true` if the sensor was calibrated using auto-calibration.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
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
    pub fn physical_orientation(&self) -> Result<InertialOrientation, InertialError> {
        Ok(self.status()?.physical_orientation())
    }

    /// Calibrate the IMU asynchronously.
    ///
    /// Returns an [`InertialCalibrateFuture`] that resolves once the calibration operation has finished.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::CalibrationTimedOut`] if a 3-second timeout is exceeded.
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    pub fn calibrate(&mut self) -> InertialCalibrateFuture {
        InertialCalibrateFuture::Calibrate(self.port.number())
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
    pub fn rotation(&self) -> Result<f64, InertialError> {
        self.validate()?;
        Ok(unsafe { vexDeviceImuHeadingGet(self.device) } - self.rotation_offset)
    }

    /// Returns the Inertial Sensor’s yaw angle bounded from [0, 360) degrees.
    ///
    /// Clockwise rotations are represented with positive degree values, while counterclockwise rotations are
    /// represented with negative ones.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
    pub fn heading(&self) -> Result<f64, InertialError> {
        self.validate()?;
        Ok(
            (unsafe { vexDeviceImuDegreesGet(self.device) } - self.heading_offset)
                % Self::MAX_HEADING,
        )
    }

    /// Returns a quaternion representing the Inertial Sensor’s current orientation.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
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

    /// Returns the Euler angles (pitch, yaw, roll) representing the Inertial Sensor’s orientation.
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
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

    /// Returns the sensor's raw acceleration readings in g (~9.8 m/s/s).
    ///
    /// # Errors
    ///
    /// - An [`InertialError::Port`] error is returned if there is not an inertial sensor connected to the port.
    /// - An [`InertialError::BadStatus`] error is returned if the inertial sensor failed to report its status.
    /// - An [`InertialError::StillCalibrating`] error is returned if the sensor is currently calibrating and cannot yet be used.
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
    /// Future is currently waiting for the IMU to report [`InertialStatus::CALIBRATING`], indicating
    /// that it has started calibration.
    Start,

    /// Waiting for calibration to end.
    End,
}

/// Future that calibrates an IMU
/// created with [`InertialSensor::calibrate`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[derive(Debug, Clone, Copy)]
pub enum InertialCalibrateFuture {
    /// Calibrate the IMU
    Calibrate(u8),
    /// Wait for the IMU to either begin calibrating or end calibration, depending on the
    /// designated [`CalibrationPhase`].
    Waiting(u8, Instant, CalibrationPhase),
}

impl core::future::Future for InertialCalibrateFuture {
    type Output = Result<(), InertialError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match *self {
            Self::Calibrate(port) => {
                if let Err(err) = validate_port(port, SmartDeviceType::Imu) {
                    // IMU isn't plugged in, no need to go any further.
                    Poll::Ready(Err(InertialError::Port { source: err }))
                } else {
                    // Request that VEXos calibrate the IMU, and transition to pending state.
                    unsafe { vexDeviceImuReset(vexDeviceGetByIndex(u32::from(port - 1))) }

                    // Change to waiting for calibration to start.
                    *self = Self::Waiting(port, Instant::now(), CalibrationPhase::Start);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Self::Waiting(port, timestamp, phase) => {
                if timestamp.elapsed()
                    > match phase {
                        CalibrationPhase::Start => InertialSensor::CALIBRATION_START_TIMEOUT,
                        CalibrationPhase::End => InertialSensor::CALIBRATION_END_TIMEOUT,
                    }
                {
                    // Calibration took too long and exceeded timeout.
                    return Poll::Ready(Err(InertialError::CalibrationTimedOut));
                }

                let status = InertialStatus::from_bits_retain(
                    if let Err(err) = validate_port(port, SmartDeviceType::Imu) {
                        // IMU got unplugged, so we'll resolve early.
                        return Poll::Ready(Err(InertialError::Port { source: err }));
                    } else {
                        // Get status flags from VEXos.
                        let flags = unsafe {
                            vexDeviceImuStatusGet(vexDeviceGetByIndex(u32::from(port - 1)))
                        };

                        // 0xFF is returned when the sensor fails to report flags.
                        if flags == InertialStatus::STATUS_ERROR {
                            return Poll::Ready(Err(InertialError::BadStatus));
                        }

                        flags
                    },
                );

                if status.contains(InertialStatus::CALIBRATING) && phase == CalibrationPhase::Start
                {
                    // Calibration has started, so we'll change to waiting for it to end.
                    *self = Self::Waiting(port, Instant::now(), CalibrationPhase::End);
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                } else if !status.contains(InertialStatus::CALIBRATING)
                    && phase == CalibrationPhase::End
                {
                    // Calibration has finished.
                    return Poll::Ready(Ok(()));
                }

                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

/// Errors that can occur when interacting with an Inertial Sensor.
#[derive(Debug, Snafu)]
pub enum InertialError {
    /// The sensor took longer than three seconds to calibrate.
    CalibrationTimedOut,
    /// The sensor is still calibrating.
    StillCalibrating,
    /// The sensor failed to report its status flags (returned 0xFF).
    BadStatus,
    #[snafu(display("{source}"), context(false))]
    /// Generic port related error.
    Port {
        /// The source of the error.
        source: PortError,
    },
}
