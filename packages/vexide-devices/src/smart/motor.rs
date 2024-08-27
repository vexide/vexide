//! V5 Smart Motors

use core::time::Duration;

use bitflags::bitflags;
use snafu::Snafu;
use vex_sdk::{
    vexDeviceMotorAbsoluteTargetSet, vexDeviceMotorBrakeModeSet, vexDeviceMotorCurrentGet,
    vexDeviceMotorCurrentLimitGet, vexDeviceMotorCurrentLimitSet, vexDeviceMotorEfficiencyGet,
    vexDeviceMotorEncoderUnitsSet, vexDeviceMotorFaultsGet, vexDeviceMotorFlagsGet,
    vexDeviceMotorGearingGet, vexDeviceMotorGearingSet, vexDeviceMotorPositionGet,
    vexDeviceMotorPositionRawGet, vexDeviceMotorPositionReset, vexDeviceMotorPositionSet,
    vexDeviceMotorPowerGet, vexDeviceMotorReverseFlagGet, vexDeviceMotorReverseFlagSet,
    vexDeviceMotorTemperatureGet, vexDeviceMotorTorqueGet, vexDeviceMotorVelocityGet,
    vexDeviceMotorVelocitySet, vexDeviceMotorVelocityUpdate, vexDeviceMotorVoltageGet,
    vexDeviceMotorVoltageLimitGet, vexDeviceMotorVoltageLimitSet, vexDeviceMotorVoltageSet,
    V5MotorBrakeMode, V5MotorGearset, V5_DeviceT,
};
#[cfg(feature = "dangerous_motor_tuning")]
use vex_sdk::{vexDeviceMotorPositionPidSet, vexDeviceMotorVelocityPidSet, V5_DeviceMotorPid};

use super::{SmartDevice, SmartDeviceTimestamp, SmartDeviceType, SmartPort};
use crate::{position::Position, PortError};

/// The basic motor struct.
#[derive(Debug, PartialEq)]
pub struct Motor {
    port: SmartPort,
    target: MotorControl,
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for Motor {}
unsafe impl Sync for Motor {}

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
    /// Returns `true` if the level is [`Forward`](Direction::Forward).
    pub const fn is_forward(&self) -> bool {
        match self {
            Self::Forward => true,
            Self::Reverse => false,
        }
    }

    /// Returns `true` if the level is [`Reverse`](Direction::Reverse).
    pub const fn is_reverse(&self) -> bool {
        match self {
            Self::Forward => false,
            Self::Reverse => true,
        }
    }
}

impl core::ops::Not for Direction {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }
}

impl Motor {
    /// The maximum voltage value that can be sent to a [`Motor`].
    pub const MAX_VOLTAGE: f64 = 12.0;

    /// The rate at which data can be read from a [`Motor`].
    pub const DATA_READ_INTERVAL: Duration = Duration::from_millis(10);

    /// The rate at which data can be written to a [`Motor`].
    pub const DATA_WRITE_INTERVAL: Duration = Duration::from_millis(5);

    /// Create a new motor from a smart port index.
    pub fn new(port: SmartPort, gearset: Gearset, direction: Direction) -> Self {
        let device = unsafe { port.device_handle() }; // SAFETY: This function is only called once on this port.

        // NOTE: SDK properly stores device state when unplugged, meaning that we can safely
        // set these without consequence even if the device is not available. This is an edge
        // case for the SDK though, and seems to just be a thing for motors and rotation sensors.
        unsafe {
            vexDeviceMotorEncoderUnitsSet(
                device,
                vex_sdk::V5MotorEncoderUnits::kMotorEncoderCounts,
            );
            vexDeviceMotorGearingSet(device, gearset.into());
            vexDeviceMotorReverseFlagSet(device, direction.is_reverse());
        }

        Self {
            port,
            target: MotorControl::Voltage(0.0),
            device,
        }
    }

    /// Sets the target that the motor should attempt to reach.
    ///
    /// This could be a voltage, velocity, position, or even brake mode.
    pub fn set_target(&mut self, target: MotorControl) -> Result<(), MotorError> {
        let gearset = self.gearset()?;
        self.target = target;

        match target {
            MotorControl::Brake(mode) => unsafe {
                vexDeviceMotorBrakeModeSet(self.device, mode.into());
                // Force motor into braking by putting it into velocity control with a 0rpm setpoint.
                vexDeviceMotorVelocitySet(self.device, 0);
            },
            MotorControl::Velocity(rpm) => unsafe {
                vexDeviceMotorBrakeModeSet(
                    self.device,
                    vex_sdk::V5MotorBrakeMode::kV5MotorBrakeModeCoast,
                );
                vexDeviceMotorVelocitySet(self.device, rpm);
            },
            MotorControl::Voltage(volts) => unsafe {
                vexDeviceMotorBrakeModeSet(
                    self.device,
                    vex_sdk::V5MotorBrakeMode::kV5MotorBrakeModeCoast,
                );
                vexDeviceMotorVoltageSet(self.device, (volts * 1000.0) as i32);
            },
            MotorControl::Position(position, velocity) => unsafe {
                vexDeviceMotorBrakeModeSet(
                    self.device,
                    vex_sdk::V5MotorBrakeMode::kV5MotorBrakeModeCoast,
                );
                vexDeviceMotorAbsoluteTargetSet(
                    self.device,
                    position.as_ticks(gearset.ticks_per_revolution()) as f64,
                    velocity,
                );
            },
        }

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
        self.validate_port()?;

        unsafe {
            vexDeviceMotorVelocityUpdate(self.device, velocity);
        }

        if let MotorControl::Position(position, _) = self.target {
            self.target = MotorControl::Position(position, velocity)
        }

        Ok(())
    }

    /// Get the current [`MotorControl`] value that the motor is attempting to use.
    pub fn target(&self) -> Result<MotorControl, MotorError> {
        self.validate_port()?;
        Ok(self.target)
    }

    /// Sets the gearset of the motor.
    pub fn set_gearset(&mut self, gearset: Gearset) -> Result<(), MotorError> {
        self.validate_port()?;
        unsafe {
            vexDeviceMotorGearingSet(self.device, gearset.into());
        }
        Ok(())
    }

    /// Gets the gearset of the motor.
    pub fn gearset(&self) -> Result<Gearset, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorGearingGet(self.device) }.into())
    }

    /// Gets the estimated angular velocity (RPM) of the motor.
    pub fn velocity(&self) -> Result<i32, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorVelocityGet(self.device) })
    }

    /// Returns the power drawn by the motor in Watts.
    pub fn power(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorPowerGet(self.device) })
    }

    /// Returns the torque output of the motor in Nm.
    pub fn torque(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorTorqueGet(self.device) })
    }

    /// Returns the voltage the motor is drawing in volts.
    pub fn voltage(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorVoltageGet(self.device) } as f64 / 1000.0)
    }

    /// Returns the current position of the motor.
    pub fn position(&self) -> Result<Position, MotorError> {
        let gearset = self.gearset()?;
        Ok(Position::from_ticks(
            unsafe { vexDeviceMotorPositionGet(self.device) } as i64,
            gearset.ticks_per_revolution(),
        ))
    }

    /// Returns the most recently recorded raw encoder tick data from the motor's IME
    /// along with a timestamp of the internal clock of the motor indicating when the
    /// data was recorded.
    pub fn raw_position(&self) -> Result<(i32, SmartDeviceTimestamp), MotorError> {
        self.validate_port()?;

        let mut timestamp: u32 = 0;
        let ticks = unsafe { vexDeviceMotorPositionRawGet(self.device, &mut timestamp) };

        Ok((ticks, SmartDeviceTimestamp(timestamp)))
    }

    /// Returns the electrical current draw of the motor in amps.
    pub fn current(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorCurrentGet(self.device) } as f64 / 1000.0)
    }

    /// Gets the efficiency of the motor from a range of [0.0, 1.0].
    ///
    /// An efficiency of 1.0 means that the motor is moving electrically while
    /// drawing no electrical power, and an efficiency of 0.0 means that the motor
    /// is drawing power but not moving.
    pub fn efficiency(&self) -> Result<f64, MotorError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceMotorEfficiencyGet(self.device) } / 100.0)
    }

    /// Sets the current encoder position to zero without moving the motor.
    /// Analogous to taring or resetting the encoder to the current position.
    pub fn reset_position(&mut self) -> Result<(), MotorError> {
        self.validate_port()?;
        unsafe { vexDeviceMotorPositionReset(self.device) }
        Ok(())
    }

    /// Sets the current encoder position to the given position without moving the motor.
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given position.
    pub fn set_position(&mut self, position: Position) -> Result<(), MotorError> {
        self.validate_port()?;
        unsafe { vexDeviceMotorPositionSet(self.device, position.as_degrees()) }
        Ok(())
    }

    /// Sets the current limit for the motor in amps.
    pub fn set_current_limit(&mut self, limit: f64) -> Result<(), MotorError> {
        self.validate_port()?;
        unsafe { vexDeviceMotorCurrentLimitSet(self.device, (limit * 1000.0) as i32) }
        Ok(())
    }

    /// Sets the voltage limit for the motor in volts.
    pub fn set_voltage_limit(&mut self, limit: f64) -> Result<(), MotorError> {
        self.validate_port()?;

        unsafe {
            vexDeviceMotorVoltageLimitSet(self.device, (limit * 1000.0) as i32);
        }

        Ok(())
    }

    /// Gets the current limit for the motor in amps.
    pub fn current_limit(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorCurrentLimitGet(self.device) } as f64 / 1000.0)
    }

    /// Gets the voltage limit for the motor if one has been explicitly set.
    pub fn voltage_limit(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorVoltageLimitGet(self.device) } as f64 / 1000.0)
    }

    /// Returns the internal teperature recorded by the motor in increments of 5°C.
    pub fn temperature(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorTemperatureGet(self.device) })
    }

    /// Get the status flags of a motor.
    pub fn status(&self) -> Result<MotorStatus, MotorError> {
        self.validate_port()?;

        let status = MotorStatus::from_bits_retain(unsafe { vexDeviceMotorFlagsGet(self.device) });

        // This is technically just a flag, but it indicates that an error occurred when trying
        // to get the flags, so we return early here.
        if status.contains(MotorStatus::BUSY) {
            return Err(MotorError::Busy);
        }

        Ok(status)
    }

    /// Get the fault flags of the motor.
    pub fn faults(&self) -> Result<MotorFaults, MotorError> {
        self.validate_port()?;

        Ok(MotorFaults::from_bits_retain(unsafe {
            vexDeviceMotorFaultsGet(self.device)
        }))
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
        self.validate_port()?;

        unsafe {
            vexDeviceMotorReverseFlagSet(self.device, direction.is_reverse());
        }

        Ok(())
    }

    /// Get the [`Direction`] of this motor.
    pub fn direction(&self) -> Result<Direction, MotorError> {
        self.validate_port()?;

        Ok(match unsafe { vexDeviceMotorReverseFlagGet(self.device) } {
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
        self.validate_port()?;

        let mut constants = V5_DeviceMotorPid::from(constants);
        unsafe { vexDeviceMotorVelocityPidSet(self.device, &mut constants) }

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
        self.validate_port()?;

        let mut constants = V5_DeviceMotorPid::from(constants);
        unsafe { vexDeviceMotorPositionPidSet(self.device, &mut constants) }

        Ok(())
    }
}

impl SmartDevice for Motor {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Motor
    }
}

/// Determines how a motor should act when braking.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BrakeMode {
    /// Motor never brakes.
    Coast,

    /// Motor uses regenerative braking to slow down faster.
    Brake,

    /// Motor exerts force to hold the same position.
    Hold,
}

impl From<V5MotorBrakeMode> for BrakeMode {
    fn from(value: V5MotorBrakeMode) -> Self {
        match value {
            V5MotorBrakeMode::kV5MotorBrakeModeBrake => Self::Brake,
            V5MotorBrakeMode::kV5MotorBrakeModeCoast => Self::Coast,
            V5MotorBrakeMode::kV5MotorBrakeModeHold => Self::Hold,
            _ => unreachable!(),
        }
    }
}

impl From<BrakeMode> for V5MotorBrakeMode {
    fn from(value: BrakeMode) -> Self {
        match value {
            BrakeMode::Brake => Self::kV5MotorBrakeModeBrake,
            BrakeMode::Coast => Self::kV5MotorBrakeModeCoast,
            BrakeMode::Hold => Self::kV5MotorBrakeModeHold,
        }
    }
}

bitflags! {
    /// The fault flags returned by a [`Motor`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct MotorFaults: u32 {
        /// The motor's temperature is above its limit.
        const OVER_TEMPERATURE = 0x01;

        /// The motor is over current.
        const OVER_CURRENT = 0x04;

        /// The motor's H-bridge has encountered a fault.
        const DRIVER_FAULT = 0x02;

        /// The motor's H-bridge is over current.
        const DRIVER_OVER_CURRENT = 0x08;
    }
}

bitflags! {
    /// The status bits returned by a [`Motor`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct MotorStatus: u32 {
        /// Failed communicate with the motor
        const BUSY = 0x01;

        /// The motor is currently near zero velocity.
        #[deprecated(
            since = "0.9.0",
            note = "This flag will never be set by the hardware, even though it exists in the SDK. This may change in the future."
        )]
        const ZERO_VELOCITY = 0x02;

        /// The motor is at its zero position.
        #[deprecated(
            since = "0.9.0",
            note = "This flag will never be set by the hardware, even though it exists in the SDK. This may change in the future."
        )]
        const ZERO_POSITION = 0x04;
    }
}

/// Internal gearset used by VEX smart motors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gearset {
    /// 36:1 gear ratio
    Red,
    /// 18:1 gear ratio
    Green,
    /// 6:1 gear ratio
    Blue,
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

    /// Rated max speed for a smart motor with a [`Red`](Gearset::Red) gearset.
    pub const MAX_RED_RPM: f64 = 100.0;
    /// Rated speed for a smart motor with a [`Green`](Gearset::Green) gearset.
    pub const MAX_GREEN_RPM: f64 = 200.0;
    /// Rated speed for a smart motor with a [`Blue`](Gearset::Blue) gearset.
    pub const MAX_BLUE_RPM: f64 = 600.0;

    /// Number of encoder ticks per revolution for the [`Red`](Gearset::Red) gearset.
    pub const RED_TICKS_PER_REVOLUTION: u32 = 1800;
    /// Number of encoder ticks per revolution for the [`Green`](Gearset::Green) gearset.
    pub const GREEN_TICKS_PER_REVOLUTION: u32 = 900;
    /// Number of encoder ticks per revolution for the [`Blue`](Gearset::Blue) gearset.
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

impl From<V5MotorGearset> for Gearset {
    fn from(value: V5MotorGearset) -> Self {
        match value {
            V5MotorGearset::kMotorGearSet_06 => Self::Blue,
            V5MotorGearset::kMotorGearSet_18 => Self::Green,
            V5MotorGearset::kMotorGearSet_36 => Self::Red,
            _ => unreachable!(),
        }
    }
}

impl From<Gearset> for V5MotorGearset {
    fn from(value: Gearset) -> Self {
        match value {
            Gearset::Blue => Self::kMotorGearSet_06,
            Gearset::Green => Self::kMotorGearSet_18,
            Gearset::Red => Self::kMotorGearSet_36,
        }
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
// #[cfg(feature = "dangerous_motor_tuning")]
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
impl From<MotorTuningConstants> for V5_DeviceMotorPid {
    fn from(value: MotorTuningConstants) -> Self {
        Self {
            kf: (value.kf * 16.0) as u8,
            kp: (value.kp * 16.0) as u8,
            ki: (value.ki * 16.0) as u8,
            kd: (value.kd * 16.0) as u8,
            filter: (value.filter * 16.0) as u8,
            limit: (value.integral_limit * 16.0) as u16,
            threshold: (value.tolerance * 16.0) as u8,
            loopspeed: (value.sample_rate.as_millis() * 16) as u8,
            ..Default::default()
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
