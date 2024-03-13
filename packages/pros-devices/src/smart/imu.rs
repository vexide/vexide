//! Inertial sensor (IMU) device.

use core::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bitflags::bitflags;
use pros_core::{
    bail_on,
    error::{take_errno, FromErrno, PortError},
    map_errno,
    time::Instant,
};
use pros_sys::{PROS_ERR, PROS_ERR_F};
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceType, SmartPort};

/// Represents a smart port configured as a V5 inertial sensor (IMU)
#[derive(Debug, Eq, PartialEq)]
pub struct InertialSensor {
    port: SmartPort,
}

impl InertialSensor {
    /// The timeout for the IMU to calibrate.
    pub const CALIBRATION_TIMEOUT: Duration = Duration::from_secs(3);

    /// The minimum data rate that you can set an IMU to.
    pub const MIN_DATA_RATE: Duration = Duration::from_millis(5);

    /// Create a new inertial sensor from a smart port index.
    pub const fn new(port: SmartPort) -> Self {
        Self { port }
    }

    /// Calibrate IMU.
    ///
    /// This takes approximately 2 seconds, and is blocking until the IMU status flag is set properly.
    /// There is additionally a 3 second timeout that will return [`InertialError::CalibrationTimedOut`] if the timeout is exceeded.
    pub fn calibrate_blocking(&mut self) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_reset_blocking(self.port.index())
        });
        Ok(())
    }

    /// Calibrate IMU asynchronously.
    ///
    /// Returns an [`InertialCalibrateFuture`] that is be polled until the IMU status flag reports the sensor as
    /// no longer calibrating.
    /// There a 3 second timeout that will return [`InertialError::CalibrationTimedOut`] if the timeout is exceeded.
    pub fn calibrate(&mut self) -> InertialCalibrateFuture {
        InertialCalibrateFuture::Calibrate(self.port.index())
    }

    /// Check if the Intertial Sensor is currently calibrating.
    pub fn is_calibrating(&mut self) -> Result<bool, InertialError> {
        Ok(self.status()?.contains(InertialStatus::CALIBRATING))
    }

    /// Get the total number of degrees the Inertial Sensor has spun about the z-axis.
    ///
    /// This value is theoretically unbounded. Clockwise rotations are represented with positive degree values,
    /// while counterclockwise rotations are represented with negative ones.
    pub fn rotation(&self) -> Result<f64, InertialError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::imu_get_rotation(self.port.index())
        }))
    }

    /// Get the Inertial Sensor’s heading relative to the initial direction of its x-axis.
    ///
    /// This value is bounded by [0, 360) degrees. Clockwise rotations are represented with positive degree values,
    /// while counterclockwise rotations are represented with negative ones.
    pub fn heading(&self) -> Result<f64, InertialError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::imu_get_heading(self.port.index())
        }))
    }

    /// Get the Inertial Sensor’s pitch angle bounded by (-180, 180) degrees.
    pub fn pitch(&self) -> Result<f64, InertialError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::imu_get_pitch(self.port.index())
        }))
    }

    /// Get the Inertial Sensor’s roll angle bounded by (-180, 180) degrees.
    pub fn roll(&self) -> Result<f64, InertialError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::imu_get_roll(self.port.index())
        }))
    }

    /// Get the Inertial Sensor’s yaw angle bounded by (-180, 180) degrees.
    pub fn yaw(&self) -> Result<f64, InertialError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::imu_get_yaw(self.port.index())
        }))
    }

    /// Read the inertial sensor's status code.
    pub fn status(&self) -> Result<InertialStatus, InertialError> {
        let bits = bail_on!(pros_sys::E_IMU_STATUS_ERROR, unsafe {
            pros_sys::imu_get_status(self.port.index())
        });

        Ok(InertialStatus::from_bits_retain(bits))
    }

    /// Get a quaternion representing the Inertial Sensor’s orientation.
    pub fn quaternion(&self) -> Result<Quaternion, InertialError> {
        unsafe { pros_sys::imu_get_quaternion(self.port.index()).try_into() }
    }

    /// Get the Euler angles representing the Inertial Sensor’s orientation.
    pub fn euler(&self) -> Result<Euler, InertialError> {
        unsafe { pros_sys::imu_get_euler(self.port.index()).try_into() }
    }

    /// Get the Inertial Sensor’s raw gyroscope values.
    pub fn gyro_rate(&self) -> Result<InertialRaw, InertialError> {
        unsafe { pros_sys::imu_get_gyro_rate(self.port.index()).try_into() }
    }

    /// Get the Inertial Sensor’s raw accelerometer values.
    pub fn accel(&self) -> Result<InertialRaw, InertialError> {
        unsafe { pros_sys::imu_get_accel(self.port.index()).try_into() }
    }

    /// Resets the current reading of the Inertial Sensor’s heading to zero.
    pub fn zero_heading(&mut self) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_tare_heading(self.port.index())
        });
        Ok(())
    }

    /// Resets the current reading of the Inertial Sensor’s rotation to zero.
    pub fn zero_rotation(&mut self) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_tare_rotation(self.port.index())
        });
        Ok(())
    }

    /// Resets the current reading of the Inertial Sensor’s pitch to zero.
    pub fn zero_pitch(&mut self) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_tare_pitch(self.port.index())
        });
        Ok(())
    }

    /// Resets the current reading of the Inertial Sensor’s roll to zero.
    pub fn zero_roll(&mut self) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_tare_roll(self.port.index())
        });
        Ok(())
    }

    /// Resets the current reading of the Inertial Sensor’s yaw to zero.
    pub fn zero_yaw(&mut self) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_tare_yaw(self.port.index())
        });
        Ok(())
    }

    /// Reset all 3 euler values of the Inertial Sensor to 0.
    pub fn zero_euler(&mut self) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_tare_euler(self.port.index())
        });
        Ok(())
    }

    /// Resets all 5 values of the Inertial Sensor to 0.
    pub fn zero(&mut self) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe { pros_sys::imu_tare(self.port.index()) });
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s euler values to target euler values.
    ///
    /// Will default to +/- 180 if target exceeds +/- 180.
    pub fn set_euler(&mut self, euler: Euler) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_set_euler(self.port.index(), euler.into())
        });
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s rotation to target value.
    pub fn set_rotation(&mut self, rotation: f64) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_set_rotation(self.port.index(), rotation)
        });
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s heading to target value.
    ///
    /// Target will default to 360 if above 360 and default to 0 if below 0.
    pub fn set_heading(&mut self, heading: f64) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_set_heading(self.port.index(), heading)
        });
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s pitch to target value.
    ///
    /// Will default to +/- 180 if target exceeds +/- 180.
    pub fn set_pitch(&mut self, pitch: f64) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_set_pitch(self.port.index(), pitch)
        });
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s roll to target value
    ///
    /// Will default to +/- 180 if target exceeds +/- 180.
    pub fn set_roll(&mut self, roll: f64) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_set_roll(self.port.index(), roll)
        });
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s yaw to target value.
    ///
    /// Will default to +/- 180 if target exceeds +/- 180.
    pub fn set_yaw(&mut self, yaw: f64) -> Result<(), InertialError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::imu_set_yaw(self.port.index(), yaw)
        });
        Ok(())
    }

    /// Sets the update rate of the IMU.
    ///
    /// This duration must be above [`Self::MIN_DATA_RATE`] (5 milliseconds).
    pub fn set_data_rate(&mut self, data_rate: Duration) -> Result<(), InertialError> {
        unsafe {
            let rate_ms = if data_rate > Self::MIN_DATA_RATE {
                if let Ok(rate) = u32::try_from(data_rate.as_millis()) {
                    rate
                } else {
                    return Err(InertialError::InvalidDataRate);
                }
            } else {
                return Err(InertialError::InvalidDataRate);
            };

            bail_on!(
                PROS_ERR,
                pros_sys::imu_set_data_rate(self.port.index(), rate_ms)
            );
        }
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

impl TryFrom<pros_sys::quaternion_s_t> for Quaternion {
    type Error = InertialError;

    fn try_from(value: pros_sys::quaternion_s_t) -> Result<Quaternion, InertialError> {
        Ok(Self {
            x: bail_on!(PROS_ERR_F, value.x),
            y: value.y,
            z: value.z,
            w: value.w,
        })
    }
}

impl From<Quaternion> for pros_sys::quaternion_s_t {
    fn from(value: Quaternion) -> Self {
        pros_sys::quaternion_s_t {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
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

impl TryFrom<pros_sys::euler_s_t> for Euler {
    type Error = InertialError;

    fn try_from(value: pros_sys::euler_s_t) -> Result<Euler, InertialError> {
        Ok(Self {
            pitch: bail_on!(PROS_ERR_F, value.pitch),
            roll: value.roll,
            yaw: value.yaw,
        })
    }
}

impl From<Euler> for pros_sys::euler_s_t {
    fn from(val: Euler) -> Self {
        pros_sys::euler_s_t {
            pitch: val.pitch,
            roll: val.roll,
            yaw: val.yaw,
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

impl TryFrom<pros_sys::imu_raw_s> for InertialRaw {
    type Error = InertialError;

    fn try_from(value: pros_sys::imu_raw_s) -> Result<InertialRaw, InertialError> {
        Ok(Self {
            x: bail_on!(PROS_ERR_F, value.x),
            y: value.y,
            z: value.z,
        })
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

#[derive(Debug, Clone, Copy)]
/// Future that calibrates an IMU
/// created with [`InertialSensor::calibrate`].
pub enum InertialCalibrateFuture {
    /// Calibrate the IMU
    Calibrate(u8),
    /// Wait for the IMU to finish calibrating
    Waiting(u8, Instant),
}

impl core::future::Future for InertialCalibrateFuture {
    type Output = Result<(), InertialError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match *self {
            Self::Calibrate(port) => match unsafe { pros_sys::imu_reset(port) } {
                PROS_ERR => {
                    let errno = take_errno();
                    Poll::Ready(Err(InertialError::from_errno(errno)
                        .unwrap_or_else(|| panic!("Unknown errno code {errno}"))))
                }
                _ => {
                    *self = Self::Waiting(port, Instant::now());
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            },
            Self::Waiting(port, timestamp) => {
                let is_calibrating = match unsafe { pros_sys::imu_get_status(port) } {
                    pros_sys::E_IMU_STATUS_ERROR => {
                        let errno = take_errno();
                        return Poll::Ready(Err(InertialError::from_errno(take_errno())
                            .unwrap_or_else(|| panic!("Unknown errno code {errno}"))));
                    }
                    value => (value & pros_sys::E_IMU_STATUS_CALIBRATING) != 0,
                };

                if !is_calibrating {
                    return Poll::Ready(Ok(()));
                } else if timestamp.elapsed() > InertialSensor::CALIBRATION_TIMEOUT {
                    return Poll::Ready(Err(InertialError::CalibrationTimedOut));
                }

                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with an Inertial Sensor.
pub enum InertialError {
    /// The inertial sensor spent too long calibrating.
    CalibrationTimedOut,
    /// Invalid sensor data rate, expected >= 5 milliseconds.
    InvalidDataRate,
    #[snafu(display("{source}"), context(false))]
    /// Generic port related error.
    Port {
        /// The source of the error.
        source: PortError,
    },
}

map_errno! {
    InertialError {
        EAGAIN => Self::CalibrationTimedOut,
    }
    inherit PortError;
}
