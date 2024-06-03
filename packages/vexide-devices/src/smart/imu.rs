//! Inertial sensor (IMU) device.

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

/// Represents a smart port configured as a V5 inertial sensor (IMU)
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
    /// The time limit used by the PROS kernel for bailing out of calibration. In theory, this
    /// could be as low as 2s, but is kept at 3s for margin-of-error.
    ///
    /// <https://github.com/purduesigbots/pros/blob/master/src/devices/vdml_imu.c#L31>
    pub const CALIBRATION_TIMEOUT: Duration = Duration::from_secs(3);

    /// The minimum data rate that you can set an IMU to.
    pub const MIN_DATA_INTERVAL: Duration = Duration::from_millis(5);

    /// The maximum value that can be returned by [`Self::heading`].
    pub const MAX_HEADING: f64 = 360.0;

    /// Create a new inertial sensor from a smart port index.
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

    /// Read the inertial sensor's status code.
    pub fn status(&self) -> Result<InertialStatus, InertialError> {
        self.validate_port()?;

        let bits = unsafe { vexDeviceImuStatusGet(self.device) };

        if bits == InertialStatus::STATUS_ERROR {
            return Err(InertialError::BadStatus);
        }

        Ok(InertialStatus::from_bits_retain(bits))
    }

    /// Check if the Intertial Sensor is currently calibrating.
    pub fn is_calibrating(&self) -> Result<bool, InertialError> {
        Ok(self.status()?.contains(InertialStatus::CALIBRATING))
    }

    /// Check if the Intertial Sensor was calibrated using auto-calibration.
    pub fn is_auto_calibrated(&self) -> Result<bool, InertialError> {
        Ok(self.status()?.contains(InertialStatus::AUTO_CALIBRTED))
    }

    /// Check if the Intertial Sensor was calibrated using auto-calibration.
    pub fn physical_orientation(&self) -> Result<InertialOrientation, InertialError> {
        Ok(self.status()?.physical_orientation())
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
        Ok(unsafe { vexDeviceImuHeadingGet(self.device) } - self.rotation_offset)
    }

    /// Get the Inertial Sensor’s yaw angle bounded by [0, 360) degrees.
    ///
    /// Clockwise rotations are represented with positive degree values, while counterclockwise rotations are
    /// represented with negative ones.
    pub fn heading(&self) -> Result<f64, InertialError> {
        self.validate()?;
        Ok(
            (unsafe { vexDeviceImuDegreesGet(self.device) } - self.heading_offset)
                % Self::MAX_HEADING,
        )
    }

    /// Get a quaternion representing the Inertial Sensor’s orientation.
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

    /// Get the Euler angles (pitch, yaw, roll) representing the Inertial Sensor’s orientation.
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

    /// Get the Inertial Sensor’s raw gyroscope values.
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

    /// Get the Inertial Sensor’s raw accelerometer values.
    pub fn accel(&self) -> Result<Vector3<f64>, InertialError> {
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

    /// Resets the current reading of the Inertial Sensor’s heading to zero.
    pub fn reset_heading(&mut self) -> Result<(), InertialError> {
        self.set_heading(Default::default())
    }

    /// Resets the current reading of the Inertial Sensor’s rotation to zero.
    pub fn reset_rotation(&mut self) -> Result<(), InertialError> {
        self.set_rotation(Default::default())
    }

    /// Sets the current reading of the Inertial Sensor’s rotation to target value.
    pub fn set_rotation(&mut self, rotation: f64) -> Result<(), InertialError> {
        self.validate()?;

        self.rotation_offset = rotation - unsafe { vexDeviceImuHeadingGet(self.device) };

        Ok(())
    }

    /// Sets the current reading of the Inertial Sensor’s heading to target value.
    ///
    /// Target will default to 360 if above 360 and default to 0 if below 0.
    pub fn set_heading(&mut self, heading: f64) -> Result<(), InertialError> {
        self.validate()?;

        self.heading_offset = heading - unsafe { vexDeviceImuDegreesGet(self.device) };

        Ok(())
    }

    /// Sets the computation speed of the IMU.
    ///
    /// This duration should be above [`Self::MIN_DATA_RATE`] (5 milliseconds).
    pub fn set_data_rate(&mut self, data_rate: Duration) -> Result<(), InertialError> {
        self.validate()?;

        let mut time_ms = data_rate
            .as_millis()
            .max(Self::MIN_DATA_INTERVAL.as_millis()) as u32;
        time_ms -= time_ms % 5; // Rate is in increments of 5ms - not sure if this is necessary, but PROS does it.

        unsafe { vexDeviceImuDataRateSet(self.device, time_ms) }

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
        const AUTO_CALIBRTED = 0b10000;
    }
}

impl InertialStatus {
    /// The return value of [`vexDeviceImuStatusGet`] when the device fails to report its
    /// status bits.
    pub const STATUS_ERROR: u32 = 0xFF;

    /// Returns the physical orientation of the sensor measured at calibration.
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
            }
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
                        let flags = unsafe {
                            vexDeviceImuStatusGet(vexDeviceGetByIndex((port - 1) as u32))
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
                    *self = Self::Waiting(port, timestamp, CalibrationPhase::End);
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

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with an Inertial Sensor.
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
