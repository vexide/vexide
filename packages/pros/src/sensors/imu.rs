use core::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use pros_sys::{PROS_ERR, PROS_ERR_F};
use snafu::Snafu;

use crate::error::{bail_on, map_errno, take_errno, FromErrno, PortError};

pub const IMU_RESET_TIMEOUT: Duration = Duration::from_secs(3);
pub const IMU_MIN_DATA_RATE: Duration = Duration::from_millis(5);

/// Represents a smart port configured as a V5 inertial sensor (IMU)
#[derive(Debug, Clone, Copy)]
pub struct InertialSensor {
    port: u8,
}

impl InertialSensor {
    /// Create a new inertial sensor from a smart port index.
    pub fn new(port: u8) -> Result<Self, InertialError> {
        let sensor = Self { port };
        sensor.status()?;
        Ok(sensor)
    }

    /// Calibrate IMU.
    ///
    /// This takes approximately 2 seconds, and is blocking until the IMU status flag is set properly.
    /// There is additionally a 3 second timeout that will return [`InertialError::CalibrationTimedOut`] if the timeout is exceeded.
    pub fn calibrate_blocking(&self) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_reset_blocking(self.port));
        }
        Ok(())
    }

    /// Calibrate IMU asynchronously.
    ///
    /// Returns an [`InertialCalibrationFuture`] that is be polled until the IMU status flag reports the sensor as
    /// no longer calibrating.
    /// There a 3 second timeout that will return [`InertialError::CalibrationTimedOut`] if the timeout is exceeded.
    pub fn calibrate(&self) -> InertialCalibrateFuture {
        InertialCalibrateFuture::Calibrate(*self)
    }

    /// Check if the Intertial Sensor is currently calibrating.
    pub fn is_calibrating(&self) -> Result<bool, InertialError> {
        Ok(self.status()?.calibrating())
    }

    /// Get the total number of degrees the Inertial Sensor has spun about the z-axis.
    ///
    /// This value is theoretically unbounded. Clockwise rotations are represented with positive degree values,
    /// while counterclockwise rotations are represented with negative ones.
    pub fn rotation(&self) -> Result<f64, InertialError> {
        unsafe { Ok(bail_on!(PROS_ERR_F, pros_sys::imu_get_rotation(self.port))) }
    }

    /// Get the Inertial Sensor’s heading relative to the initial direction of its x-axis.
    ///
    /// This value is bounded by [0, 360) degrees. Clockwise rotations are represented with positive degree values,
    /// while counterclockwise rotations are represented with negative ones.
    pub fn heading(&self) -> Result<f64, InertialError> {
        unsafe { Ok(bail_on!(PROS_ERR_F, pros_sys::imu_get_heading(self.port))) }
    }

    /// Get the Inertial Sensor’s pitch angle bounded by (-180, 180) degrees.
    pub fn pitch(&self) -> Result<f64, InertialError> {
        unsafe { Ok(bail_on!(PROS_ERR_F, pros_sys::imu_get_pitch(self.port))) }
    }

    /// Get the Inertial Sensor’s roll angle bounded by (-180, 180) degrees.
    pub fn roll(&self) -> Result<f64, InertialError> {
        unsafe { Ok(bail_on!(PROS_ERR_F, pros_sys::imu_get_roll(self.port))) }
    }

    /// Get the Inertial Sensor’s yaw angle bounded by (-180, 180) degrees.
    pub fn yaw(&self) -> Result<f64, InertialError> {
        unsafe { Ok(bail_on!(PROS_ERR_F, pros_sys::imu_get_yaw(self.port))) }
    }

    /// Read the inertial sensor's status code.
    pub fn status(&self) -> Result<InertialStatus, InertialError> {
        unsafe {
            Ok(bail_on!(
                PROS_ERR as _,
                pros_sys::imu_get_status(self.port) as pros_sys::imu_status_e_t
            )
            .into())
        }
    }

    /// Get a quaternion representing the Inertial Sensor’s orientation.
    pub fn quaternion(&self) -> Result<Quaternion, InertialError> {
        unsafe { pros_sys::imu_get_quaternion(self.port).try_into() }
    }

    /// Get the Euler angles representing the Inertial Sensor’s orientation.
    pub fn euler(&self) -> Result<Euler, InertialError> {
        unsafe { pros_sys::imu_get_euler(self.port).try_into() }
    }

    /// Get the Inertial Sensor’s raw gyroscope values.
    pub fn gyro_rate(&self) -> Result<InertialRaw, InertialError> {
        unsafe { pros_sys::imu_get_gyro_rate(self.port).try_into() }
    }

    /// Get the Inertial Sensor’s raw accelerometer values.
    pub fn accel(&self) -> Result<InertialRaw, InertialError> {
        unsafe { pros_sys::imu_get_accel(self.port).try_into() }
    }

    /// Resets the current reading of the Inertial Sensor’s heading to zero.
    pub fn zero_heading(&self) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_tare_heading(self.port));
        }
        Ok(())
    }

    /// Resets the current reading of the Inertial Sensor’s rotation to zero.
    pub fn zero_rotation(&self) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_tare_rotation(self.port));
        }
        Ok(())
    }

    /// Resets the current reading of the Inertial Sensor’s pitch to zero.
    pub fn zero_pitch(&self) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_tare_pitch(self.port));
        }
        Ok(())
    }

    /// Resets the current reading of the Inertial Sensor’s roll to zero.
    pub fn zero_roll(&self) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_tare_roll(self.port));
        }
        Ok(())
    }

    /// Resets the current reading of the Inertial Sensor’s yaw to zero.
    pub fn zero_yaw(&self) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_tare_yaw(self.port));
        }
        Ok(())
    }

    /// Reset all 3 euler values of the Inertial Sensor to 0.
    pub fn zero_euler(&self) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_tare_euler(self.port));
        }
        Ok(())
    }

    /// Resets all 5 values of the Inertial Sensor to 0.
    pub fn zero(&self) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_tare(self.port));
        }
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s euler values to target euler values.
    ///
    /// Will default to +/- 180 if target exceeds +/- 180.
    pub fn set_euler(&self, euler: Euler) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_set_euler(self.port, euler.into()));
        }
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s rotation to target value.
    pub fn set_rotation(&self, rotation: f64) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_set_rotation(self.port, rotation));
        }
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s heading to target value.
    ///
    /// Target will default to 360 if above 360 and default to 0 if below 0.
    pub fn set_heading(&self, heading: f64) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_set_heading(self.port, heading));
        }
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s pitch to target value.
    ///
    /// Will default to +/- 180 if target exceeds +/- 180.
    pub fn set_pitch(&self, pitch: f64) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_set_pitch(self.port, pitch));
        }
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s roll to target value
    ///
    /// Will default to +/- 180 if target exceeds +/- 180.
    pub fn set_roll(&self, roll: f64) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_set_roll(self.port, roll));
        }
        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s yaw to target value.
    ///
    /// Will default to +/- 180 if target exceeds +/- 180.
    pub fn set_yaw(&self, yaw: f64) -> Result<(), InertialError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::imu_set_yaw(self.port, yaw));
        }
        Ok(())
    }

    /// Sets the update rate of the IMU.
    ///
    /// This duration must be above [`IMU_MIN_DATA_RATE`] (5 milliseconds).
    pub fn set_data_rate(&self, data_rate: Duration) -> Result<(), InertialError> {
        unsafe {
            let rate_ms = if data_rate > IMU_MIN_DATA_RATE {
                if let Ok(rate) = u32::try_from(data_rate.as_millis()) {
                    rate
                } else {
                    return Err(InertialError::InvalidDataRate);
                }
            } else {
                return Err(InertialError::InvalidDataRate);
            };

            bail_on!(PROS_ERR, pros_sys::imu_set_data_rate(self.port, rate_ms));
        }
        Ok(())
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
    fn from(val: Quaternion) -> Self {
        pros_sys::quaternion_s_t {
            x: val.x,
            y: val.y,
            z: val.z,
            w: val.w,
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
#[derive(Clone, Copy, Debug, PartialEq)]
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

/// Represents a status code returned by the Inertial Sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InertialStatus(pub u32);

impl InertialStatus {
    /// Determine if the sensor is currently calibrating.
    pub const fn calibrating(&self) -> bool {
        self.0 & pros_sys::E_IMU_STATUS_CALIBRATING != 0
    }

    /// Determine if an error state was reached when trying to get the IMU's status.
    pub const fn error(&self) -> bool {
        self.0 & pros_sys::E_IMU_STATUS_ERROR != 0
    }
}

impl From<pros_sys::imu_status_e_t> for InertialStatus {
    fn from(value: pros_sys::imu_status_e_t) -> Self {
        Self(value)
    }
}

pub enum InertialCalibrateFuture {
    Calibrate(InertialSensor),
    Waiting(InertialSensor, Duration),
}

impl core::future::Future for InertialCalibrateFuture {
    type Output = Result<(), InertialError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        loop {
            match *self {
                Self::Calibrate(imu) => match unsafe { pros_sys::imu_reset(imu.port) } {
                    PROS_ERR => {
                        let errno = take_errno();
                        return Poll::Ready(Err(InertialError::from_errno(errno)
                            .unwrap_or_else(|| panic!("Unknown errno code {errno}"))));
                    }
                    _ => {
                        *self = Self::Waiting(
                            imu,
                            Duration::from_micros(unsafe { pros_sys::rtos::micros() }),
                        );
                    }
                },
                Self::Waiting(imu, timestamp) => {
                    let elapsed =
                        Duration::from_micros(unsafe { pros_sys::rtos::micros() }) - timestamp;

                    return if elapsed > IMU_RESET_TIMEOUT {
                        Poll::Ready(Err(InertialError::CalibrationTimedOut))
                    } else if imu.is_calibrating()? {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    } else {
                        Poll::Ready(Ok(()))
                    };
                }
            }
        }
    }
}

#[derive(Debug, Snafu)]
pub enum InertialError {
    #[snafu(display("Inertial sensor is still calibrating, but exceeded calibration timeout."))]
    CalibrationTimedOut,
    #[snafu(display("Sensor data rate has a minimum duration of 5 milliseconds."))]
    InvalidDataRate,
    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    InertialError {
        EAGAIN => Self::CalibrationTimedOut,
    }
    inherit PortError;
}
