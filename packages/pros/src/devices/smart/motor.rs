//! Motors and gearsets.
//!
//! The motor API is similar to that of [`sensors`](crate::sensors).
//! Multiple motors can be created on the same port and they are thread safe.
//!
//! Motors can be created with the [`Motor::new`] function.
//! Once created they can be controlled with one three functions:
//! [`Motor::set_output`], [`Motor::set_raw_output`], and [`Motor::set_voltage`].
//! [`Motor::set_output`] takes in a f32 from -1 to 1 for ease of use with [`Controller`](crate::controller::Controller)s.
//! [`Motor::set_raw_output`] takes in an i8 from -127 to 127.
//! [`Motor::set_voltage`] takes in an i16 from -12000 to 12000.
//!
//! Example of driving a single motor with a controller:
//! ```rust
//! # use pros::prelude::*;
//! let motor = Motor::new(1, BrakeMode::Brake).unwrap();
//! let controller = Controller::Master;
//! loop {
//!     let output = controller.state().joysticks.left.y;
//!     motor.set_output(output).ok();
//! }
//! ```

use core::time::Duration;

use pros_sys::{motor_fault_e_t, motor_flag_e_t, PROS_ERR, PROS_ERR_F};
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::{
    devices::Position,
    error::{bail_on, map_errno, PortError},
};

/// The maximum voltage value that can be sent to a [`Motor`].
pub const MOTOR_MAX_VOLTAGE: f64 = 12.0;

/// The rate at which data can be read from a [`Motor`].
pub const MOTOR_READ_DATA_RATE: Duration = Duration::from_millis(10);

/// The rate at which data can be written to a [`Motor].
pub const MOTOR_WRITE_DATA_RATE: Duration = Duration::from_millis(5);

/// The basic motor struct.
#[derive(Debug, PartialEq)]
pub struct Motor {
    port: SmartPort,
    target: MotorTarget,
}

/// Represents a possible target for a [`Motor`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotorTarget {
    /// Motor is braking using a [`BrakeMode`].
    Brake(BrakeMode),

    /// Motor is attempting to hold a velocity using internal PID control.
    Velocity(i32),

    /// Motor is outputting a raw voltage.
    Voltage(f64),

    /// Motor is attempting to reach a relative position.
    RelativePosition(Position, i32),

    /// Motor is attempting to reach an absolute position.
    AbsolutePosition(Position, i32),
}

// TODO: Measure the number of counts per rotation. Fow now we assume it is 4096
// TODO: Wrap motor_modify_profiled_velocity
impl Motor {
    /// Create a new motor from a smart port index.
    pub fn new(port: SmartPort, gearset: Gearset, reversed: bool) -> Result<Self, MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_encoder_units(port.index(), pros_sys::E_MOTOR_ENCODER_DEGREES)
        });

        let mut motor = Self {
            port,
            target: MotorTarget::Voltage(0.0),
        };

        motor.set_gearset(gearset)?;
        motor.set_reversed(reversed)?;

        Ok(motor)
    }

    /// Sets the target that the motor should attempt to reach.
    ///
    /// This could be a voltage, velocity, position, or even brake mode.
    pub fn set_target(&mut self, target: MotorTarget) -> Result<(), MotorError> {
        match target {
            MotorTarget::Brake(mode) => unsafe {
                bail_on!(
                    PROS_ERR,
                    pros_sys::motor_set_brake_mode(self.port.index(), mode.into())
                );
                bail_on!(PROS_ERR, pros_sys::motor_brake(self.port.index()));
            },
            MotorTarget::Velocity(rpm) => {
                bail_on!(PROS_ERR, unsafe {
                    pros_sys::motor_move_velocity(self.port.index(), rpm)
                });
            }
            MotorTarget::Voltage(volts) => {
                bail_on!(PROS_ERR, unsafe {
                    pros_sys::motor_move_voltage(self.port.index(), (volts * 1000.0) as i32)
                });
            }
            MotorTarget::AbsolutePosition(position, velocity) => {
                bail_on!(PROS_ERR, unsafe {
                    pros_sys::motor_move_absolute(
                        self.port.index(),
                        position.into_degrees(),
                        velocity,
                    )
                });
            }
            MotorTarget::RelativePosition(position, velocity) => {
                bail_on!(PROS_ERR, unsafe {
                    pros_sys::motor_move_relative(
                        self.port.index(),
                        position.into_degrees(),
                        velocity,
                    )
                });
            }
        }

        self.target = target;
        Ok(())
    }

    /// Sets the motors target to a given [`BrakeMode`].
    pub fn brake(&mut self, mode: BrakeMode) -> Result<(), MotorError> {
        self.set_target(MotorTarget::Brake(mode))
    }

    /// Spins the motor at a target velocity.
    ///
    /// This velocity corresponds to different actual speeds in RPM depending on the gearset used for the motor.
    /// Velocity is held with an internal PID controller to ensure consistent speed, as opposed to setting the
    /// motor's voltage.
    pub fn set_velocity(&mut self, rpm: i32) -> Result<(), MotorError> {
        self.set_target(MotorTarget::Velocity(rpm))
    }

    /// Sets the motor's ouput voltage.
    ///
    /// This voltage value spans from -12 (fully spinning reverse) to +12 (fully spinning forwards) volts, and
    /// controls the raw output of the motor.
    pub fn set_voltage(&mut self, volts: f64) -> Result<(), MotorError> {
        self.set_target(MotorTarget::Voltage(volts))
    }

    /// Sets an absolute position target for the motor to attempt to reach.
    pub fn set_absolute_target(
        &mut self,
        position: Position,
        velocity: i32,
    ) -> Result<(), MotorError> {
        self.set_target(MotorTarget::AbsolutePosition(position, velocity))
    }

    /// Sets an position target relative to the current position for the motor to attempt to reach.
    pub fn set_relative_target(
        &mut self,
        position: Position,
        velocity: i32,
    ) -> Result<(), MotorError> {
        self.set_target(MotorTarget::RelativePosition(position, velocity))
    }

    /// Get the current [`MotorTarget`] value that the motor is attempting to reach.
    pub fn target(&self) -> MotorTarget {
        self.target
    }

    /// Sets the gearset of the motor.
    pub fn set_gearset(&mut self, gearset: Gearset) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_gearing(self.port.index(), gearset as i32)
        });
        Ok(())
    }

    /// Gets the gearset of the motor.
    pub fn gearset(&self) -> Result<Gearset, MotorError> {
        unsafe { pros_sys::motor_get_gearing(self.port.index()).try_into() }
    }

    /// Gets the estimated angular velocity (RPM) of the motor.
    pub fn velocity(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_actual_velocity(self.port.index())
        }))
    }

    /// Returns the power drawn by the motor in Watts.
    pub fn power(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_power(self.port.index())
        }))
    }

    /// Returns the torque output of the motor in Nm.
    pub fn torque(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_torque(self.port.index())
        }))
    }

    /// Returns the voltage the motor is drawing in volts.
    pub fn voltage(&self) -> Result<f64, MotorError> {
        // docs say this function returns PROS_ERR_F but it actually returns PROS_ERR
        let millivolts = bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_get_voltage(self.port.index())
        });
        Ok(millivolts as f64 / 1000.0)
    }

    /// Returns the current position of the motor.
    pub fn position(&self) -> Result<Position, MotorError> {
        Ok(Position::from_degrees(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_position(self.port.index())
        })))
    }

    /// Returns the raw position data recorded by the motor at a given timestamp.
    pub fn raw_position(&self, timestamp: Duration) -> Result<i32, MotorError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_get_raw_position(self.port.index(), timestamp.as_millis() as *const u32)
        }))
    }

    /// Returns the electrical current draw of the motor in amps.
    pub fn current(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_get_current_draw(self.port.index())
        }) as f64
            / 1000.0)
    }

    /// Gets the efficiency of the motor in percent.
    ///
    /// An efficiency of 100% means that the motor is moving electrically while
    /// drawing no electrical power, and an efficiency of 0% means that the motor
    /// is drawing power but not moving.
    pub fn efficiency(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_efficiency(self.port.index())
        }))
    }

    /// Sets the current encoder position to zero without moving the motor.
    /// Analogous to taring or resetting the encoder to the current position.
    pub fn zero(&mut self) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_tare_position(self.port.index())
        });
        Ok(())
    }

    /// Sets the current encoder position to the given position without moving the motor.
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given position.
    pub fn set_position(&mut self, position: Position) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_zero_position(self.port.index(), position.into_degrees())
        });
        Ok(())
    }

    pub fn set_current_limit(&mut self, limit: f64) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_current_limit(self.port.index(), (limit * 1000.0) as i32)
        });
        Ok(())
    }

    pub fn set_voltage_limit(&mut self, limit: i32) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_voltage_limit(self.port.index(), limit)
        });
        Ok(())
    }

    fn current_limit(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_get_current_limit(self.port.index())
        }) as f64
            / 1000.0)
    }

    fn voltage_limit(&self) -> Result<Option<i32>, MotorError> {
        let raw_limit = bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_get_voltage_limit(self.port.index())
        });

        Ok(match raw_limit {
            0 => None,
            limited => Some(limited),
        })
    }

    /// Get the status flagss of a motor.
    pub fn status(&self) -> Result<MotorStatus, MotorError> {
        unsafe { pros_sys::motor_get_flags(self.port.index()).try_into() }
    }

    /// Check if the motor's stopped flag is set.
    pub fn is_stopped(&self) -> Result<bool, MotorError> {
        Ok(self.status()?.is_stopped())
    }

    /// Check if the motor's zeroed flag is set.
    pub fn is_zeroed(&self) -> Result<bool, MotorError> {
        Ok(self.status()?.is_zeroed())
    }

    /// Get the faults flags of the motor.
    pub fn faults(&self) -> Result<MotorFaults, MotorError> {
        unsafe { pros_sys::motor_get_faults(self.port.index()).try_into() }
    }

    /// Check if the motor's over temperature flag is set.
    pub fn is_over_temperature(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.is_over_temperature())
    }

    /// Check if the motor's overcurrent flag is set.
    pub fn is_over_current(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.is_over_current())
    }

    /// Check if a H-bridge (motor driver) fault has occurred.
    pub fn is_driver_fault(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.is_driver_fault())
    }

    /// Check if the motor's H-bridge has an overucrrent fault.
    pub fn is_driver_over_current(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.is_driver_over_current())
    }

    /// Reverse this motor's output values.
    pub fn set_reversed(&mut self, reversed: bool) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_reversed(self.port.index(), reversed)
        });
        Ok(())
    }

    /// Check if this motor has been reversed.
    pub fn is_reversed(&self) -> Result<bool, MotorError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_is_reversed(self.port.index())
        }) == 1)
    }

    /// Returns a future that completes when the motor reports that it has stopped.
    pub const fn wait_until_stopped(&self) -> MotorStoppedFuture<'_> {
        MotorStoppedFuture { motor: self }
    }

    /// Adjusts the internal tuning constants of the motor when using velocity control.
    ///
    /// # Hardware Safety
    ///
    /// Retuning internal motor control is **dangerous**, and can result in permanent hardware damage
    /// to smart motors if done incorrectly. Use these functions entirely at your own risk.
    ///
    /// VEX has chosen not to disclose the default constants used by smart motors, and currently
    /// has no plans to do so. As such, the units and finer details of [`MotorTuningConstants`] are not
    /// well-known or understood, as we have no reference for what these constants should look
    /// like.
    #[cfg(feature = "dangerous_motor_tuning")]
    pub fn set_velocity_tuning_constants(
        &mut self,
        constants: MotorTuningConstants,
    ) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            #[allow(deprecated)]
            pros_sys::motor_set_pos_pid_full(self.port.index(), constants.into())
        });
        Ok(())
    }

    /// Adjusts the internal tuning constants of the motor when using position control.
    ///
    /// # Hardware Safety
    ///
    /// Retuning internal motor control is **dangerous**, and can result in permanent hardware damage
    /// to smart motors if done incorrectly. Use these functions entirely at your own risk.
    ///
    /// VEX has chosen not to disclose the default constants used by smart motors, and currently
    /// has no plans to do so. As such, the units and finer details of [`MotorTuningConstants`] are not
    /// well-known or understood, as we have no reference for what these constants should look
    /// like.
    #[cfg(feature = "dangerous_motor_tuning")]
    pub fn set_position_tuning_constants(
        &mut self,
        constants: MotorTuningConstants,
    ) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            #[allow(deprecated)]
            pros_sys::motor_set_vel_pid_full(self.port.index(), constants.into())
        });
        Ok(())
    }
}

impl SmartDevice for Motor {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Motor
    }
}

/// Determines how a motor should act when braking.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
pub enum BrakeMode {
    /// Motor never brakes.
    None = pros_sys::E_MOTOR_BRAKE_COAST,
    /// Motor uses regenerative braking to slow down faster.
    Brake = pros_sys::E_MOTOR_BRAKE_BRAKE,
    /// Motor exerts force to hold the same position.
    Hold = pros_sys::E_MOTOR_BRAKE_HOLD,
}

impl TryFrom<pros_sys::motor_brake_mode_e_t> for BrakeMode {
    type Error = MotorError;

    fn try_from(value: pros_sys::motor_brake_mode_e_t) -> Result<Self, MotorError> {
        bail_on!(PROS_ERR, value);

        Ok(match value {
            pros_sys::E_MOTOR_BRAKE_COAST => Self::None,
            pros_sys::E_MOTOR_BRAKE_BRAKE => Self::Brake,
            pros_sys::E_MOTOR_BRAKE_HOLD => Self::Hold,
            _ => unreachable!(),
        })
    }
}

impl From<BrakeMode> for pros_sys::motor_brake_mode_e_t {
    fn from(value: BrakeMode) -> pros_sys::motor_brake_mode_e_t {
        value as _
    }
}

/// The fault flags returned by a [`Motor`].
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct MotorFaults(pub u32);

impl MotorFaults {
    /// Checks if the motor's temperature is above its limit.
    pub fn is_over_temperature(&self) -> bool {
        self.0 & pros_sys::E_MOTOR_FAULT_MOTOR_OVER_TEMP != 0
    }

    /// Check if the motor's H-bridge has encountered a fault.
    pub fn is_driver_fault(&self) -> bool {
        self.0 & pros_sys::E_MOTOR_FAULT_DRIVER_FAULT != 0
    }

    /// Check if the motor is over current.
    pub fn is_over_current(&self) -> bool {
        self.0 & pros_sys::E_MOTOR_FAULT_OVER_CURRENT != 0
    }

    /// Check if the motor's H-bridge is over current.
    pub fn is_driver_over_current(&self) -> bool {
        self.0 & pros_sys::E_MOTOR_FAULT_DRV_OVER_CURRENT != 0
    }
}

impl TryFrom<motor_fault_e_t> for MotorFaults {
    type Error = MotorError;

    fn try_from(value: motor_fault_e_t) -> Result<Self, Self::Error> {
        Ok(Self(bail_on!(PROS_ERR as _, value)))
    }
}

/// The status flags returned by a [`Motor`].
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct MotorStatus(pub u32);

impl MotorStatus {
    /// Check if the motor is currently stopped.
    pub fn is_stopped(&self) -> bool {
        self.0 & pros_sys::E_MOTOR_FLAGS_ZERO_VELOCITY != 0
    }

    /// Check if the motor is at its zero position.
    pub fn is_zeroed(&self) -> bool {
        self.0 & pros_sys::E_MOTOR_FLAGS_ZERO_POSITION != 0
    }
}

impl TryFrom<motor_flag_e_t> for MotorStatus {
    type Error = MotorError;

    fn try_from(value: motor_flag_e_t) -> Result<Self, Self::Error> {
        let flags = bail_on!(PROS_ERR as _, value);

        if flags & pros_sys::E_MOTOR_FLAGS_BUSY == 0 {
            Ok(Self(flags))
        } else {
            Err(MotorError::Busy)
        }
    }
}

/// Internal gearset used by VEX smart motors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Gearset {
    /// 36:1 gear ratio
    Red = pros_sys::E_MOTOR_GEAR_RED,
    /// 18:1 gear ratio
    Green = pros_sys::E_MOTOR_GEAR_GREEN,
    /// 6:1 gear ratio
    Blue = pros_sys::E_MOTOR_GEAR_BLUE,
}

impl Gearset {
    /// 36:1 gear ratio
    pub const RATIO_36: Gearset = Gearset::Red;
    /// 18:1 gear ratio
    pub const RATIO_18: Gearset = Gearset::Green;
    /// 6:1 gear ratio
    pub const RATIO_6: Gearset = Gearset::Blue;

    /// 100 rpm
    pub const RPM_100: Gearset = Gearset::Red;
    /// 200 rpm
    pub const RPM_200: Gearset = Gearset::Green;
    /// 600 rpm
    pub const RPM_600: Gearset = Gearset::Blue;
}

impl From<Gearset> for pros_sys::motor_gearset_e_t {
    fn from(value: Gearset) -> Self {
        value as _
    }
}

impl TryFrom<pros_sys::motor_gearset_e_t> for Gearset {
    type Error = MotorError;

    fn try_from(value: pros_sys::motor_gearset_e_t) -> Result<Self, MotorError> {
        bail_on!(PROS_ERR, value);

        Ok(match value {
            pros_sys::E_MOTOR_GEAR_RED => Self::Red,
            pros_sys::E_MOTOR_GEAR_GREEN => Self::Green,
            pros_sys::E_MOTOR_GEAR_BLUE => Self::Blue,
            _ => unreachable!(),
        })
    }
}

/// Holds the information about a Motor's position or velocity PID controls.
///
/// These values are in 4.4 format, meaning that a value of 0x20 represents 2.0,
/// 0x21 represents 2.0625, 0x22 represents 2.125, etc.
#[cfg(feature = "dangerous_motor_tuning")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotorTuningConstants {
    /// The feedforward constant.
    pub kf: f64,

    /// The proportional constant.
    pub kp: f64,

    /// The integral constant.
    pub ki: f64,

    /// The derivative constant.
    pub kd: f64,

    /// A constant used for filtering the profile acceleration.
    pub filter: f64,

    /// The integral limit.
    pub limit: f64,

    /// The threshold for determining if a position movement has
    /// reached its goal. This has no effect for velocity PID calculations.
    pub threshold: f64,

    /// The rate at whsich the PID computation is run in ms.
    pub loopspeed: Duration,
}

#[cfg(feature = "dangerous_motor_tuning")]
impl From<MotorTuningConstants> for pros_sys::motor_pid_full_s_t {
    fn from(value: MotorTuningConstants) -> Self {
        unsafe {
            // Docs incorrectly claim that this function can set errno.
            // It can't. <https://github.com/purduesigbots/pros/blob/master/src/devices/vdml_motors.c#L250>.
            #[allow(deprecated)]
            pros_sys::motor_convert_pid_full(
                value.kf,
                value.kp,
                value.ki,
                value.kd,
                value.filter,
                value.limit,
                value.threshold,
                value.loopspeed.as_millis() as f64,
            )
        }
    }
}

/// A future that completes when the motor reports that it has stopped.
/// Created by [`Motor::wait_until_stopped`]
#[derive(Debug)]
pub struct MotorStoppedFuture<'a> {
    motor: &'a Motor,
}

impl<'a> core::future::Future for MotorStoppedFuture<'a> {
    type Output = crate::Result;
    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        match self.motor.status()?.is_stopped() {
            true => core::task::Poll::Ready(Ok(())),
            false => {
                cx.waker().wake_by_ref();
                core::task::Poll::Pending
            }
        }
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when using a motor.
pub enum MotorError {
    /// Failed to communicate with the motor while attempting to read flags.
    Busy,
    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}

map_errno! {
    MotorError {}
    inherit PortError;
}
