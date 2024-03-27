//! Inertial sensor (IMU) device.

use core::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bitflags::bitflags;
use pros_core::{error::PortError, time::Instant};
use snafu::Snafu;
use vex_sdk::{
    vexDeviceGetByIndex, vexDeviceImuAttitudeGet, vexDeviceImuDataRateSet, vexDeviceImuDegreesGet,
    vexDeviceImuHeadingGet, vexDeviceImuQuaternionGet, vexDeviceImuRawAccelGet,
    vexDeviceImuRawGyroGet, vexDeviceImuReset, vexDeviceImuStatusGet, V5_DeviceImuAttitude,
    V5_DeviceImuQuaternion, V5_DeviceImuRaw,
};

use super::{validate_port, SmartDevice, SmartDeviceInternal, SmartDeviceType, SmartPort};

/// Represents a smart port configured as a V5 inertial sensor (IMU)
#[derive(Debug, PartialEq)]
pub struct InertialSensor {
    port: SmartPort,
    rotation_offset: f64,
    heading_offset: f64,
    euler_offset: Euler,
}

impl InertialSensor {
    /// The time limit used by the PROS kernel for bailing out of calibration. In theory, this
    /// could be as low as 2s, but is kept at 3s for margin-of-error.
    ///
    /// <https://github.com/purduesigbots/pros/blob/master/src/devices/vdml_imu.c#L31>
    pub const CALIBRATION_TIMEOUT: Duration = Duration::from_secs(3);

    /// The minimum data rate that you can set an IMU to.
    pub const MIN_DATA_RATE: Duration = Duration::from_millis(5);

    /// The maximum value that can be returned by [`Self::heading`].
    pub const MAX_HEADING: f64 = 360.0;

    /// Create a new inertial sensor from a smart port index.
    pub const fn new(port: SmartPort) -> Self {
        Self {
            port,
            rotation_offset: 0.0,
            heading_offset: 0.0,
            euler_offset: Euler::default(),
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

    /// Read the inertial sensor's status code.
    pub fn status(&self) -> Result<InertialStatus, InertialError> {
        self.validate_port()?;

        Ok(InertialStatus::from_bits_retain(unsafe {
            vexDeviceImuStatusGet(self.device_handle())
        }))
    }

    /// Check if the Intertial Sensor is currently calibrating.
    pub fn is_calibrating(&mut self) -> Result<bool, InertialError> {
        Ok(self.status()?.contains(InertialStatus::CALIBRATING))
    }

    /// Calibrate IMU asynchronously.
    ///
    /// Returns an [`InertialCalibrateFuture`] that is be polled until the IMU status flag reports the sensor as
    /// no longer calibrating.
    /// There a 3 second timeout that will return [`InertialError::CalibrationTimedOut`] if the timeout is exceeded.
    pub fn calibrate(&mut self) -> InertialCalibrateFuture {
        InertialCalibrateFuture::Calibrate(self.port.index())
    }

    /// Get the total number of degrees the Inertial Sensor has spun about the z-axis.
    ///
    /// This value is theoretically unbounded. Clockwise rotations are represented with positive degree values,
    /// while counterclockwise rotations are represented with negative ones.
    pub fn rotation(&self) -> Result<f64, InertialError> {
        self.validate()?;
        Ok(unsafe { vexDeviceImuHeadingGet(self.device_handle()) } - self.rotation_offset)
    }

    /// Get the Inertial Sensor’s yaw angle bounded by [0, 360) degrees.
    ///
    /// Clockwise rotations are represented with positive degree values, while counterclockwise rotations are
    /// represented with negative ones.
    pub fn heading(&self) -> Result<f64, InertialError> {
        self.validate()?;
        Ok(
            (unsafe { vexDeviceImuDegreesGet(self.device_handle()) } - self.heading_offset)
                % Self::MAX_HEADING,
        )
    }

    /// Get a quaternion representing the Inertial Sensor’s orientation.
    pub fn quaternion(&self) -> Result<Quaternion, InertialError> {
        self.validate()?;

        let mut data = V5_DeviceImuQuaternion::default();
        unsafe {
            vexDeviceImuQuaternionGet(self.device_handle(), &mut data);
        }

        Ok(data.into())
    }

    /// Get the Euler angles (yaw, pitch, roll) representing the Inertial Sensor’s orientation.
    pub fn euler(&self) -> Result<Euler, InertialError> {
        self.validate()?;

        let mut data = V5_DeviceImuAttitude::default();
        unsafe {
            vexDeviceImuAttitudeGet(self.device_handle(), &mut data);
        }

        Ok(data.into())
    }

    /// Get the Inertial Sensor’s raw gyroscope values.
    pub fn gyro_rate(&self) -> Result<InertialRaw, InertialError> {
        self.validate()?;

        let mut data = V5_DeviceImuRaw::default();
        unsafe {
            vexDeviceImuRawGyroGet(self.device_handle(), &mut data);
        }

        Ok(data.into())
    }

    /// Get the Inertial Sensor’s raw accelerometer values.
    pub fn accel(&self) -> Result<InertialRaw, InertialError> {
        self.validate()?;

        let mut data = V5_DeviceImuRaw::default();
        unsafe {
            vexDeviceImuRawAccelGet(self.device_handle(), &mut data);
        }

        Ok(data.into())
    }

    /// Resets the current reading of the Inertial Sensor’s heading to zero.
    pub fn reset_heading(&mut self) -> Result<(), InertialError> {
        self.set_heading(Default::default())
    }

    /// Resets the current reading of the Inertial Sensor’s rotation to zero.
    pub fn reset_rotation(&mut self) -> Result<(), InertialError> {
        self.set_rotation(Default::default())
    }

    /// Reset all 3 euler values of the Inertial Sensor to 0.
    pub fn reset_euler(&mut self) -> Result<(), InertialError> {
        self.set_euler(Default::default())
    }

    /// Resets all values of the Inertial Sensor to 0.
    pub fn reset(&mut self) -> Result<(), InertialError> {
        self.reset_heading()?;
        self.reset_rotation()?;
        self.reset_euler()?;

        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s euler values to target euler values.
    pub fn set_euler(&mut self, euler: Euler) -> Result<(), InertialError> {
        self.validate()?;

        let mut attitude = V5_DeviceImuAttitude::default();
        unsafe {
            vexDeviceImuAttitudeGet(self.device_handle(), &mut attitude);
        }

        self.euler_offset = Euler {
            yaw: euler.yaw - attitude.yaw,
            pitch: euler.pitch - attitude.pitch,
            roll: euler.roll - attitude.roll,
        };

        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s rotation to target value.
    pub fn set_rotation(&mut self, rotation: f64) -> Result<(), InertialError> {
        self.validate()?;

        self.rotation_offset = rotation - unsafe { vexDeviceImuHeadingGet(self.device_handle()) };

        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s heading to target value.
    ///
    /// Target will default to 360 if above 360 and default to 0 if below 0.
    pub fn set_heading(&mut self, heading: f64) -> Result<(), InertialError> {
        self.validate()?;

        self.heading_offset = heading - unsafe { vexDeviceImuDegreesGet(self.device_handle()) };

        Ok(())
    }

    /// Sets the update rate of the IMU.
    ///
    /// This duration must be above [`Self::MIN_DATA_RATE`] (5 milliseconds).
    pub fn set_data_rate(&mut self, data_rate: Duration) -> Result<(), InertialError> {
        self.validate()?;

        let time_ms = data_rate.as_millis().max(Self::MIN_DATA_RATE.as_millis()) as u32;
        unsafe { vexDeviceImuDataRateSet(self.device_handle(), time_ms) }

        Ok(())
    }
}

impl SmartDevice for InertialSensor {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Imu
    }
}

/// Standard quaternion consisting of a vector defining an axis of rotation
/// and a rotation value about the axis.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    /// The x-component of the axis of rotation.
    pub x: f64,

    /// The y-component of the axis of rotation.
    pub y: f64,

    /// The z-component of the axis of rotation.
    pub z: f64,

    /// The magnitude of rotation about the axis.
    pub w: f64,
}

impl From<V5_DeviceImuQuaternion> for Quaternion {
    fn from(value: V5_DeviceImuQuaternion) -> Self {
        Self {
            x: value.a,
            y: value.b,
            z: value.c,
            w: value.d,
        }
    }
}

/// A 3-axis set of euler angles.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Euler {
    /// The angle measured along the pitch axis.
    pub pitch: f64,

    /// The angle measured along the roll axis.
    pub roll: f64,

    /// The angle measured along the yaw axis.
    pub yaw: f64,
}

impl From<V5_DeviceImuAttitude> for Euler {
    fn from(value: V5_DeviceImuAttitude) -> Self {
        Self {
            pitch: value.pitch,
            roll: value.roll,
            yaw: value.yaw,
        }
    }
}

/// Represents raw data reported by the IMU.
///
/// This is effectively a 3D vector containing either angular velocity or
/// acceleration values depending on the type of data requested..
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct InertialRaw {
    /// The x component of the raw data.
    pub x: f64,

    /// The y component of the raw data.
    pub y: f64,

    /// The z component of the raw data.
    pub z: f64,
}

impl From<V5_DeviceImuRaw> for InertialRaw {
    fn from(value: V5_DeviceImuRaw) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

bitflags! {
    /// The status bits returned by an [`InertialSensor`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct InertialStatus: u32 {
        /// The sensor is currently calibrating.
        const CALIBRATING = pros_sys::E_IMU_STATUS_CALIBRATING;
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
                    // Request that vexos calibrate the IMU, and transition to pending state.
                    unsafe { vexDeviceImuReset(vexDeviceGetByIndex((port - 1) as u32)) }

                    // Change to waiting for calibration to start.
                    *self = Self::Waiting(port, Instant::now(), CalibrationPhase::Start);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            Self::Waiting(port, timestamp, phase) => {
                if timestamp.elapsed() > InertialSensor::CALIBRATION_TIMEOUT {
                    // Calibration took too long and exceeded timeout.
                    return Poll::Ready(Err(InertialError::CalibrationTimedOut));
                }

                let status = InertialStatus::from_bits_retain(
                    if let Err(err) = validate_port(port, SmartDeviceType::Imu) {
                        // IMU got unplugged, so we'll resolve early.
                        return Poll::Ready(Err(InertialError::Port { source: err }));
                    } else {
                        // Get status flags from vexos.
                        unsafe { vexDeviceImuStatusGet(vexDeviceGetByIndex((port - 1) as u32)) }
                    },
                );

                if status.contains(InertialStatus::CALIBRATING) && phase == CalibrationPhase::Start {
                    // Calibration has started, so we'll change to waiting for it to end.
                    *self = Self::Waiting(port, timestamp, CalibrationPhase::End);
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                } else if !status.contains(InertialStatus::CALIBRATING) && phase == CalibrationPhase::End {
                    // Calibration has finished.
                    return Poll::Ready(Ok(()));
                }

                cx.waker().wake_by_ref();
                Poll::Pending
            },
        }
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with an Inertial Sensor.
pub enum InertialError {
    /// The inertial sensor took longer than three seconds to calibrate.
    CalibrationTimedOut,
    /// The inertial is still calibrating.
    StillCalibrating,
    #[snafu(display("{source}"), context(false))]
    /// Generic port related error.
    Port {
        /// The source of the error.
        source: PortError,
    },
}
