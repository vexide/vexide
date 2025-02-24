//! ADI gyroscope device.
//!
//! This module provides an interface for interacting with an ADI gyroscope device.
//! The gyroscope can be used to measure the yaw rotation of your robot.
//!
//! # Hardware overview
//!
//! The ADI gyroscope is a (LY3100ALH MEMS motion sensor)[<https://content.vexrobotics.com/docs/276-2333-Datasheet-1011.pdf>].
//! This means that it can measure the rate of rotation up to ±1000 degrees per second.
//! VEXos only provides the calculated yaw rotation of the robot.
//!
//! The gyroscope is rated for a noise density of 0.016 dps/√Hz (degrees per second per square root of Hertz).
//! This means that we cannot determine the exact amount of noise in the sensor's readings because it is unknown how often VEXos polls the gyroscope.

use core::{future::Future, task::Poll, time::Duration};

use snafu::Snafu;
use vex_sdk::{vexDeviceAdiValueGet, vexDeviceAdiValueSet};

use super::{AdiDevice, AdiDeviceType, AdiPort};
use crate::{position::Position, PortError};

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
pub struct AdiGyroscopeCalibrationFuture<'a> {
    gyro: &'a mut AdiGyroscope,
    state: AdiGyroscopeCalibrationFutureState,
}
impl Future for AdiGyroscopeCalibrationFuture<'_> {
    type Output = Result<(), AdiGyroscopeError>;

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
                Err(e) => Poll::Ready(Err(AdiGyroscopeError::Port { source: e })),
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
                Err(e) => Poll::Ready(Err(e)),
            },
            AdiGyroscopeCalibrationFutureState::WaitingEnd => match this.gyro.is_calibrating() {
                Ok(false) => Poll::Ready(Ok(())),
                Ok(true) => {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Err(e) => Poll::Ready(Err(e)),
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
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let gyro = AdiGyroscope::new(peripherals.adi_port_a());
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
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let gyro = AdiGyroscope::new(peripherals.adi_port_a());
    ///     println!("Is calibrating: {:?}", gyro.is_calibrating());
    /// }
    /// ```
    pub fn is_calibrating(&self) -> Result<bool, AdiGyroscopeError> {
        self.port.validate_expander()?;

        let value = unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) };

        Ok(value == CALIBRATING_MAGIC)
    }

    /// Calibrates the gyroscope for a given duration.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let gyro = AdiGyroscope::new(peripherals.adi_port_a());
    ///     println!("Calibrating...");
    ///     println!("Calibration completed successfully? {:?}", gyro.calibrate(Duration::from_secs(2)).await.is_ok());
    /// }
    /// ```
    pub const fn calibrate(&mut self, duration: Duration) -> AdiGyroscopeCalibrationFuture<'_> {
        AdiGyroscopeCalibrationFuture {
            gyro: self,
            state: AdiGyroscopeCalibrationFutureState::Calibrate {
                calibration_duration: duration,
            },
        }
    }

    /// Returns the measured yaw rotation of the gyroscope.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if an ADI expander device was required but not connected.
    /// - A [`AdiGyroscopeError::StillCalibrating`] error is returned if the gyroscope is still calibrating.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let gyro = AdiGyroscope::new(peripherals.adi_port_a());
    ///     _ = gyro.calibrate(Duration::from_secs(2)).await;
    ///     println!("Yaw: {:?}", gyro.yaw());
    /// }
    /// ```
    pub fn yaw(&self) -> Result<Position, AdiGyroscopeError> {
        if self.is_calibrating()? {
            return Err(AdiGyroscopeError::StillCalibrating);
        }
        let value = unsafe { vexDeviceAdiValueGet(self.port.device_handle(), self.port.index()) };
        let value = f64::from(value) / 10.0;

        Ok(Position::from_degrees(value))
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
#[derive(Debug, Snafu)]
pub enum AdiGyroscopeError {
    /// Generic ADI related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
    /// The sensor is still calibrating.
    StillCalibrating,
}
