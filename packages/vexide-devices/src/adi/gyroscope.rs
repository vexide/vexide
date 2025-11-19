//! ADI Gyroscope
//!
//! This module provides an interface for interacting with an ADI gyroscope device. The gyroscope
//! can be used to measure the yaw rotation of your robot.
//!
//! # Hardware overview
//!
//! The ADI gyroscope is a (LY3100ALH MEMS motion sensor)[<https://content.vexrobotics.com/docs/276-2333-Datasheet-1011.pdf>].
//! This means that it can measure the rate of rotation up to ±1000 degrees per second.
//! VEXos only provides the calculated yaw rotation of the robot.
//!
//! The gyroscope is rated for a noise density of 0.016 dps/√Hz (degrees per second per square root
//! of Hertz). This means that we cannot determine the exact amount of noise in the sensor's
//! readings because it is unknown how often VEXos polls the gyroscope.

use core::{future::Future, task::Poll, time::Duration};

use snafu::Snafu;
use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort, PortError};
use crate::math::Angle;

/// The magic number returned by the ADI device when the gyroscope is still calibrating.
const CALIBRATING_MAGIC: i32 = -0x8000;

enum AdiGyroscopeCalibrationFutureState {
    /// Tell VEXos to start calibration for the given duration.
    Calibrate { calibration_duration: Duration },
    /// Waiting for the calibration to start.
    WaitingStart,
    /// Waiting for the calibration to end.
    WaitingEnd,
}

/// A future that calibrates an [`AdiGyroscope`] for a given duration.
pub struct CalibrateFuture<'a> {
    gyro: &'a mut AdiGyroscope,
    state: AdiGyroscopeCalibrationFutureState,
}
impl Future for CalibrateFuture<'_> {
    type Output = Result<(), PortError>;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> Poll<Self::Output> {
        let this = self.get_mut();
        match this.state {
            AdiGyroscopeCalibrationFutureState::Calibrate {
                calibration_duration,
            } => match this.gyro.port.validate_expander() {
                Ok(()) => {
                    unsafe {
                        vexDeviceAdiValueSet(
                            this.gyro.port.device_handle(),
                            this.gyro.port.index(),
                            calibration_duration.as_millis() as _,
                        );
                    }
                    this.state = AdiGyroscopeCalibrationFutureState::WaitingStart;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(error) => Poll::Ready(Err(error)),
            },
            AdiGyroscopeCalibrationFutureState::WaitingStart => match this.gyro.is_calibrating() {
                Ok(false) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Ok(true) => {
                    this.state = AdiGyroscopeCalibrationFutureState::WaitingEnd;
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(error) => Poll::Ready(Err(error)),
            },
            AdiGyroscopeCalibrationFutureState::WaitingEnd => match this.gyro.is_calibrating() {
                Ok(false) => Poll::Ready(Ok(())),
                Ok(true) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(error) => Poll::Ready(Err(error)),
            },
        }
    }
}

/// An ADI gyroscope.
#[derive(Debug, Eq, PartialEq)]
pub struct AdiGyroscope {
    port: AdiPort,
}

impl AdiGyroscope {
    /// Create a new gyroscope on the given [`AdiPort`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut gyro = AdiGyroscope::new(peripherals.adi_a);
    ///     // Do something with the gyroscope
    ///     _ = gyro.calibrate(Duration::from_secs(2)).await;
    ///     println!("{:?}", gyro.yaw());
    /// }
    /// ```
    #[must_use]
    pub fn new(port: AdiPort) -> Self {
        port.configure(AdiDeviceType::Gyro);

        Self { port }
    }

    /// Returns true if the gyroscope is still calibrating.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let gyro = AdiGyroscope::new(peripherals.adi_a);
    ///     println!("Is calibrating: {:?}", gyro.is_calibrating());
    /// }
    /// ```
    pub fn is_calibrating(&self) -> Result<bool, PortError> {
        self.port.validate_expander()?;

        let value = unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) };

        Ok(value == CALIBRATING_MAGIC)
    }

    /// Calibrates the gyroscope for a given duration.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut gyro = AdiGyroscope::new(peripherals.adi_a);
    ///     println!("Calibrating...");
    ///     println!(
    ///         "Calibration completed successfully? {:?}",
    ///         gyro.calibrate(Duration::from_secs(2)).await.is_ok()
    ///     );
    /// }
    /// ```
    pub const fn calibrate(&mut self, duration: Duration) -> CalibrateFuture<'_> {
        CalibrateFuture {
            gyro: self,
            state: AdiGyroscopeCalibrationFutureState::Calibrate {
                calibration_duration: duration,
            },
        }
    }

    /// Returns the measured yaw rotation of the gyroscope in degrees.
    ///
    /// # Errors
    ///
    /// These errors are only returned if the device is plugged into an
    /// [`AdiExpander`](crate::smart::expander::AdiExpander).
    ///
    /// - A [`PortError::Disconnected`] error is returned if no expander was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if a device other than an expander was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut gyro = AdiGyroscope::new(peripherals.adi_a);
    ///     _ = gyro.calibrate(Duration::from_secs(2)).await;
    ///
    ///     println!(
    ///         "Yaw: {}",
    ///         gyro.yaw().expect("failed to get yaw").as_degrees()
    ///     );
    /// }
    /// ```
    pub fn yaw(&self) -> Result<Angle, YawError> {
        if self.is_calibrating()? {
            return Err(YawError::StillCalibrating);
        }
        let value = unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) };
        let value = f64::from(value) / 10.0;

        Ok(Angle::from_degrees(value))
    }
}

impl AdiDevice<1> for AdiGyroscope {
    fn port_numbers(&self) -> [u8; 1] {
        [self.port.number()]
    }

    fn expander_port_number(&self) -> Option<u8> {
        self.port.expander_number()
    }

    fn device_type(&self) -> AdiDeviceType {
        AdiDeviceType::Gyro
    }
}

/// Errors that can occur when interacting with an [`AdiGyroscope`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Snafu)]
pub enum YawError {
    /// Generic ADI related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
    /// The sensor is still calibrating.
    StillCalibrating,
}
