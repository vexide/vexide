//! V5 Smart Motors

use core::time::Duration;

use bitflags::bitflags;
use pros_core::{bail_on, error::PortError, map_errno};
use pros_sys::{PROS_ERR, PROS_ERR_F};
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceTimestamp, SmartDeviceType, SmartPort};
use crate::Position;

/// The basic motor struct.
#[derive(Debug, PartialEq)]
pub struct Motor {
    port: SmartPort,
    target: MotorControl,
}

/// Represents a possible target for a [`Motor`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotorControl {
    /// Motor is braking using a [`BrakeMode`].
    Brake(BrakeMode),

    /// Motor is outputting a raw voltage.
    Voltage(f64),

    /// Motor is attempting to hold a velocity using internal PID control.
    Velocity(i32),

    /// Motor is attempting to reach a position using internal PID control.
    Position(Position, i32),
}

/// Represents a possible direction that a motor can be configured as.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    /// Motor rotates in the forward direction.
    Forward,

    /// Motor rotates in the reverse direction.
    Reverse,
}

impl Direction {
    /// Returns `true` if the level is [`Forward`].
    pub const fn is_forward(&self) -> bool {
        match self {
            Self::Forward => true,
            Self::Reverse => false,
        }
    }

    /// Returns `true` if the level is [`Reverse`].
    pub const fn is_reverse(&self) -> bool {
        match self {
            Self::Forward => false,
            Self::Reverse => true,
        }
    }
}

impl Motor {
    /// The maximum voltage value that can be sent to a [`Motor`].
    pub const MAX_VOLTAGE: f64 = 12.0;

    /// The rate at which data can be read from a [`Motor`].
    pub const DATA_READ_RATE: Duration = Duration::from_millis(10);

    /// The rate at which data can be written to a [`Motor`].
    pub const DATA_WRITE_RATE: Duration = Duration::from_millis(5);

    /// Create a new motor from a smart port index.
    pub fn new(
        port: SmartPort,
        gearset: Gearset,
        direction: Direction,
    ) -> Result<Self, MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_encoder_units(port.index() as i8, pros_sys::E_MOTOR_ENCODER_DEGREES)
        });

        let mut motor = Self {
            port,
            target: MotorControl::Voltage(0.0),
        };

        motor.set_gearset(gearset)?;
        motor.set_direction(direction)?;

        Ok(motor)
    }

    /// Sets the target that the motor should attempt to reach.
    ///
    /// This could be a voltage, velocity, position, or even brake mode.
    pub fn set_target(&mut self, target: MotorControl) -> Result<(), MotorError> {
        match target {
            MotorControl::Brake(mode) => unsafe {
                bail_on!(
                    PROS_ERR,
                    pros_sys::motor_set_brake_mode(self.port.index() as i8, mode.into())
                );
                bail_on!(PROS_ERR, pros_sys::motor_brake(self.port.index() as i8));
            },
            MotorControl::Velocity(rpm) => unsafe {
                bail_on!(
                    PROS_ERR,
                    pros_sys::motor_set_brake_mode(
                        self.port.index() as i8,
                        pros_sys::E_MOTOR_BRAKE_COAST
                    )
                );
                bail_on!(
                    PROS_ERR,
                    pros_sys::motor_move_velocity(self.port.index() as i8, rpm)
                );
            },
            MotorControl::Voltage(volts) => {
                bail_on!(PROS_ERR, unsafe {
                    pros_sys::motor_move_voltage(self.port.index() as i8, (volts * 1000.0) as i32)
                });
            }
            MotorControl::Position(position, velocity) => unsafe {
                bail_on!(
                    PROS_ERR,
                    pros_sys::motor_set_brake_mode(
                        self.port.index() as i8,
                        pros_sys::E_MOTOR_BRAKE_COAST
                    )
                );
                bail_on!(
                    PROS_ERR,
                    pros_sys::motor_move_absolute(
                        self.port.index() as i8,
                        position.into_degrees(),
                        velocity,
                    )
                );
            },
        }

        self.target = target;
        Ok(())
    }

    /// Sets the motors target to a given [`BrakeMode`].
    pub fn brake(&mut self, mode: BrakeMode) -> Result<(), MotorError> {
        self.set_target(MotorControl::Brake(mode))
    }

    /// Spins the motor at a target velocity.
    ///
    /// This velocity corresponds to different actual speeds in RPM depending on the gearset used for the motor.
    /// Velocity is held with an internal PID controller to ensure consistent speed, as opposed to setting the
    /// motor's voltage.
    pub fn set_velocity(&mut self, rpm: i32) -> Result<(), MotorError> {
        self.set_target(MotorControl::Velocity(rpm))
    }

    /// Sets the motor's ouput voltage.
    ///
    /// This voltage value spans from -12 (fully spinning reverse) to +12 (fully spinning forwards) volts, and
    /// controls the raw output of the motor.
    pub fn set_voltage(&mut self, volts: f64) -> Result<(), MotorError> {
        self.set_target(MotorControl::Voltage(volts))
    }

    /// Sets an absolute position target for the motor to attempt to reach.
    pub fn set_position_target(
        &mut self,
        position: Position,
        velocity: i32,
    ) -> Result<(), MotorError> {
        self.set_target(MotorControl::Position(position, velocity))
    }

    /// Changes the output velocity for a profiled movement (motor_move_absolute or motor_move_relative).
    ///
    /// This will have no effect if the motor is not following a profiled movement.
    pub fn update_profiled_velocity(&mut self, velocity: i32) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_modify_profiled_velocity(self.port.index() as i8, velocity)
        });

        match self.target {
            MotorControl::Position(position, _) => {
                self.target = MotorControl::Position(position, velocity)
            }
            _ => {}
        }

        Ok(())
    }

    /// Get the current [`MotorControl`] value that the motor is attempting to use.
    pub fn target(&self) -> MotorControl {
        self.target
    }

    /// Sets the gearset of the motor.
    pub fn set_gearset(&mut self, gearset: Gearset) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_gearing(self.port.index() as i8, gearset as i32)
        });
        Ok(())
    }

    /// Gets the gearset of the motor.
    pub fn gearset(&self) -> Result<Gearset, MotorError> {
        unsafe { pros_sys::motor_get_gearing(self.port.index() as i8).try_into() }
    }

    /// Gets the estimated angular velocity (RPM) of the motor.
    pub fn velocity(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_actual_velocity(self.port.index() as i8)
        }))
    }

    /// Returns the power drawn by the motor in Watts.
    pub fn power(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_power(self.port.index() as i8)
        }))
    }

    /// Returns the torque output of the motor in Nm.
    pub fn torque(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_torque(self.port.index() as i8)
        }))
    }

    /// Returns the voltage the motor is drawing in volts.
    pub fn voltage(&self) -> Result<f64, MotorError> {
        // docs say this function returns PROS_ERR_F but it actually returns PROS_ERR
        let millivolts = bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_get_voltage(self.port.index() as i8)
        });
        Ok(millivolts as f64 / 1000.0)
    }

    /// Returns the current position of the motor.
    pub fn position(&self) -> Result<Position, MotorError> {
        Ok(Position::from_degrees(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_position(self.port.index() as i8)
        })))
    }

    /// Returns the most recently recorded raw encoder tick data from the motor's IME
    /// along with a timestamp of the internal clock of the motor indicating when the
    /// data was recorded.
    pub fn raw_position(&self) -> Result<(i32, SmartDeviceTimestamp), MotorError> {
        let timestamp = 0 as *mut u32;

        // PROS docs claim that this function gets the position *at* a recorded timestamp,
        // but in reality the "timestamp" paramater is a mutable outvalue. The function
        // outputs the most recent recorded posision AND the timestamp it was measured at,
        // rather than a position at a requested timestamp.
        let ticks = bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_get_raw_position(self.port.index() as i8, timestamp)
        });

        Ok((ticks, SmartDeviceTimestamp(unsafe { *timestamp })))
    }

    /// Returns the electrical current draw of the motor in amps.
    pub fn current(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_get_current_draw(self.port.index() as i8)
        }) as f64
            / 1000.0)
    }

    /// Gets the efficiency of the motor from a range of [0.0, 1.0].
    ///
    /// An efficiency of 1.0 means that the motor is moving electrically while
    /// drawing no electrical power, and an efficiency of 0.0 means that the motor
    /// is drawing power but not moving.
    pub fn efficiency(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR_F, unsafe {
            pros_sys::motor_get_efficiency(self.port.index() as i8)
        }) / 100.0)
    }

    /// Sets the current encoder position to zero without moving the motor.
    /// Analogous to taring or resetting the encoder to the current position.
    pub fn zero(&mut self) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_tare_position(self.port.index() as i8)
        });
        Ok(())
    }

    /// Sets the current encoder position to the given position without moving the motor.
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given position.
    pub fn set_position(&mut self, position: Position) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_zero_position(self.port.index() as i8, position.into_degrees())
        });
        Ok(())
    }

    /// Sets the current limit for the motor in amps.
    pub fn set_current_limit(&mut self, limit: f64) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_current_limit(self.port.index() as i8, (limit * 1000.0) as i32)
        });
        Ok(())
    }

    /// Sets the voltage limit for the motor in volts.
    pub fn set_voltage_limit(&mut self, limit: f64) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            // Docs claim that this function takes volts, but this is incorrect. It takes millivolts,
            // just like all other SDK voltage-related functions.
            pros_sys::motor_set_voltage_limit(self.port.index() as i8, (limit * 1000.0) as i32)
        });

        Ok(())
    }

    /// Gets the current limit for the motor in amps.
    pub fn current_limit(&self) -> Result<f64, MotorError> {
        Ok(bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_get_current_limit(self.port.index() as i8)
        }) as f64
            / 1000.0)
    }

    // /// Gets the voltage limit for the motor if one has been explicitly set.
    // /// NOTE: Broken until next kernel version due to mutex release bug.
    // pub fn voltage_limit(&self) -> Result<f64, MotorError> {
    //     // NOTE: PROS docs claim that this function will return zero if voltage is uncapped.
    //     //
    //     // From testing this does not appear to be true, so we don't need to perform any
    //     // special checks for a zero return value.
    //     Ok(bail_on!(PROS_ERR, unsafe {
    //         pros_sys::motor_get_voltage_limit(self.port.index() as i8)
    //     }) as f64
    //         / 1000.0)
    // }

    /// Get the status flags of a motor.
    pub fn status(&self) -> Result<MotorStatus, MotorError> {
        let bits = bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::motor_get_flags(self.port.index() as i8)
        });

        // For some reason, PROS doesn't set errno if this flag is returned,
        // even though it is by-definition an error (failing to retrieve flags).
        if (bits & pros_sys::E_MOTOR_FLAGS_BUSY) != 0 {
            return Err(MotorError::Busy);
        }

        Ok(MotorStatus::from_bits_retain(bits))
    }

    /// Get the fault flags of the motor.
    pub fn faults(&self) -> Result<MotorFaults, MotorError> {
        let bits = bail_on!(PROS_ERR as u32, unsafe {
            pros_sys::motor_get_faults(self.port.index() as i8)
        });

        Ok(MotorFaults::from_bits_retain(bits))
    }

    /// Check if the motor's over temperature flag is set.
    pub fn is_over_temperature(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.contains(MotorFaults::OVER_TEMPERATURE))
    }

    /// Check if the motor's overcurrent flag is set.
    pub fn is_over_current(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.contains(MotorFaults::OVER_CURRENT))
    }

    /// Check if a H-bridge (motor driver) fault has occurred.
    pub fn is_driver_fault(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.contains(MotorFaults::DRIVER_FAULT))
    }

    /// Check if the motor's H-bridge has an overucrrent fault.
    pub fn is_driver_over_current(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.contains(MotorFaults::OVER_CURRENT))
    }

    /// Set the [`Direction`] of this motor.
    pub fn set_direction(&mut self, direction: Direction) -> Result<(), MotorError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_set_reversed(self.port.index() as i8, direction.is_reverse())
        });
        Ok(())
    }

    /// Get the [`Direction`] of this motor.
    pub fn direction(&self) -> Result<Direction, MotorError> {
        let reversed = bail_on!(PROS_ERR, unsafe {
            pros_sys::motor_is_reversed(self.port.index() as i8)
        }) == 1;

        Ok(match reversed {
            false => Direction::Forward,
            true => Direction::Reverse,
        })
    }

    /// Adjusts the internal tuning constants of the motor when using velocity control.
    ///
    /// # Hardware Safety
    ///
    /// Modifying internal motor control is **dangerous**, and can result in permanent hardware damage
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
            pros_sys::motor_set_pos_pid_full(self.port.index() as i8, constants.into())
        });
        Ok(())
    }

    /// Adjusts the internal tuning constants of the motor when using position control.
    ///
    /// # Hardware Safety
    ///
    /// Modifying internal motor control is **dangerous**, and can result in permanent hardware damage
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
            pros_sys::motor_set_vel_pid_full(self.port.index() as i8, constants.into())
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

bitflags! {
    /// The fault flags returned by a [`Motor`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct MotorFaults: u32 {
        /// The motor's temperature is above its limit.
        const OVER_TEMPERATURE = pros_sys::E_MOTOR_FAULT_MOTOR_OVER_TEMP;

        /// The motor is over current.
        const OVER_CURRENT = pros_sys::E_MOTOR_FAULT_OVER_CURRENT;

        /// The motor's H-bridge has encountered a fault.
        const DRIVER_FAULT = pros_sys::E_MOTOR_FAULT_DRIVER_FAULT;

        /// The motor's H-bridge is over current.
        const DRIVER_OVER_CURRENT = pros_sys::E_MOTOR_FAULT_DRV_OVER_CURRENT;
    }
}

bitflags! {
    /// The status bits returned by a [`Motor`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct MotorStatus: u32 {
        /// The motor is currently near zero velocity.
        #[deprecated(
            since = "0.9.0",
            note = "This flag will never be set by the hardware, even though it exists in the SDK. This may change in the future."
        )]
        const ZERO_VELOCITY = pros_sys::E_MOTOR_FLAGS_ZERO_VELOCITY;

        /// The motor is at its zero position.
        #[deprecated(
            since = "0.9.0",
            note = "This flag will never be set by the hardware, even though it exists in the SDK. This may change in the future."
        )]
        const ZERO_POSITION = pros_sys::E_MOTOR_FLAGS_ZERO_POSITION;

        /// Cannot currently communicate to the motor
        const BUSY = pros_sys::E_MOTOR_FLAGS_BUSY;
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
    /// 36:1 gear ratio (alias to `Self::Red`)
    pub const RATIO_36: Gearset = Self::Red;
    /// 18:1 gear ratio (alias to `Self::Green`)
    pub const RATIO_18: Gearset = Self::Green;
    /// 6:1 gear ratio (alias to `Self::Blue`)
    pub const RATIO_6: Gearset = Self::Blue;

    /// 100 rpm gearset (alias to `Self::Red`)
    pub const RPM_100: Gearset = Self::Red;
    /// 200 rpm (alias to `Self::Green`)
    pub const RPM_200: Gearset = Self::Green;
    /// 600 rpm (alias to `Self::Blue`)
    pub const RPM_600: Gearset = Self::Blue;

    /// Rated max speed for a smart motor with a [`Red`] gearset.
    pub const MAX_RED_RPM: f64 = 100.0;
    /// Rated speed for a smart motor with a [`Green`] gearset.
    pub const MAX_GREEN_RPM: f64 = 200.0;
    /// Rated speed for a smart motor with a [`Blue`] gearset.
    pub const MAX_BLUE_RPM: f64 = 600.0;

    /// Number of encoder ticks per revolution for the [`Red`] gearset.
    pub const RED_TICKS_PER_REVOLUTION: u32 = 1800;
    /// Number of encoder ticks per revolution for the [`Green`] gearset.
    pub const GREEN_TICKS_PER_REVOLUTION: u32 = 900;
    /// Number of encoder ticks per revolution for the [`Blue`] gearset.
    pub const BLUE_TICKS_PER_REVOLUTION: u32 = 300;

    /// Get the rated maximum speed for this motor gearset.
    pub const fn max_rpm(&self) -> f64 {
        match self {
            Self::Red => Self::MAX_RED_RPM,
            Self::Green => Self::MAX_GREEN_RPM,
            Self::Blue => Self::MAX_BLUE_RPM,
        }
    }

    /// Get the number of encoder ticks per revolution for this motor gearset.
    pub const fn ticks_per_revolution(&self) -> u32 {
        match self {
            Self::Red => Self::RED_TICKS_PER_REVOLUTION,
            Self::Green => Self::GREEN_TICKS_PER_REVOLUTION,
            Self::Blue => Self::BLUE_TICKS_PER_REVOLUTION,
        }
    }
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
/// # Hardware Safety
///
/// Modifying internal motor control is **dangerous**, and can result in permanent hardware damage
/// to smart motors if done incorrectly. Use these functions entirely at your own risk.
///
/// VEX has chosen not to disclose the default constants used by smart motors, and currently
/// has no plans to do so. As such, the units and finer details of [`MotorTuningConstants`] are not
/// well-known or understood, as we have no reference for what these constants should look
/// like.
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
    ///
    /// Presumably used for anti-windup protection.
    pub integral_limit: f64,

    /// The threshold for determining if a position movement has reached its goal.
    ///
    /// This has no effect for velocity PID calculations.
    pub tolerance: f64,

    /// The rate at which the PID computation is run in ms.
    pub sample_rate: Duration,
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
                value.tolerance,
                value.sample_rate.as_millis() as f64,
            )
        }
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when using a motor.
pub enum MotorError {
    /// Failed to communicate with the motor while attempting to read flags.
    Busy,

    /// This functionality is not currently implemented in hardware, even
    /// though the SDK may support it.
    NotImplemented,

    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}

map_errno! {
    MotorError {
        ENOSYS => Self::NotImplemented,
    }
    inherit PortError;
}
