use pros_sys::{PROS_ERR, PROS_ERR_F};
use snafu::Snafu;

use crate::{
    error::{bail_on, map_errno, PortError},
    position::Position,
};

/// The basic motor struct.
#[derive(Clone, Copy)]
pub struct Motor {
    port: u8,
}

//TODO: Implement good set_velocity and get_velocity functions.
//TODO: Measure the number of counts per rotation. Fow now we assume it is 4096
impl Motor {
    pub fn new(port: u8, brake_mode: BrakeMode) -> Result<Self, MotorError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::motor_set_encoder_units(port, pros_sys::E_MOTOR_ENCODER_DEGREES)
            );
            bail_on!(
                PROS_ERR,
                pros_sys::motor_set_brake_mode(port, brake_mode.into())
            );
        }

        Ok(Self { port })
    }

    /// Takes in a f32 from -1 to 1 that is scaled to -12 to 12 volts.
    /// Useful for driving motors with controllers.
    pub fn set_output(&self, output: f32) -> Result<(), MotorError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::motor_move(self.port, (output * 127.0) as i32)
            );
        }
        Ok(())
    }

    /// Takes in and i8 between -127 and 127 which is scaled to -12 to 12 Volts.
    pub fn set_raw_output(&self, raw_output: i8) -> Result<(), MotorError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::motor_move(self.port, raw_output as i32));
        }
        Ok(())
    }

    /// Takes in a voltage that must be between -12 and 12 Volts.
    pub fn set_voltage(&self, voltage: f32) -> Result<(), MotorError> {
        if !(-12.0..=12.0).contains(&voltage) || voltage.is_nan() {
            return Err(MotorError::VoltageOutOfRange);
        }
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::motor_move_voltage(self.port, (voltage * 1000.0) as i32)
            );
        }

        Ok(())
    }

    /// Moves the motor to an absolute position, based off of the last motor zeroing.
    /// units for the velocity is RPM.
    pub fn set_position_absolute(
        &self,
        position: Position,
        velocity: i32,
    ) -> Result<(), MotorError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::motor_move_absolute(self.port, position.into_degrees(), velocity)
            );
        };
        Ok(())
    }

    /// Moves the motor to a position relative to the current position.
    /// units for velocity is RPM.
    pub fn set_position_relative(
        &self,
        position: Position,
        velocity: i32,
    ) -> Result<(), MotorError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::motor_move_relative(self.port, position.into_degrees(), velocity)
            );
        }
        Ok(())
    }

    /// Returns the power drawn by the motor in Watts.
    pub fn power(&self) -> Result<f64, MotorError> {
        unsafe { Ok(bail_on!(PROS_ERR_F, pros_sys::motor_get_power(self.port))) }
    }

    /// Returns the torque output of the motor in Nm.
    pub fn torque(&self) -> Result<f64, MotorError> {
        unsafe { Ok(bail_on!(PROS_ERR_F, pros_sys::motor_get_torque(self.port))) }
    }

    /// Returns the voltage the motor is drawing in volts.
    pub fn voltage(&self) -> Result<f64, MotorError> {
        // docs say this function returns PROS_ERR_F but it actually returns PROS_ERR
        let millivolts = unsafe { bail_on!(PROS_ERR, pros_sys::motor_get_voltage(self.port)) };
        Ok(millivolts as f64 / 1000.0)
    }

    /// Returns the current position of the motor.
    pub fn position(&self) -> Result<Position, MotorError> {
        unsafe {
            Ok(Position::from_degrees(bail_on!(
                PROS_ERR_F,
                pros_sys::motor_get_position(self.port)
            )))
        }
    }

    /// Returns the current draw of the motor.
    pub fn current_draw(&self) -> i32 {
        unsafe { pros_sys::motor_get_current_draw(self.port) }
    }

    /// Sets the current position to zero.
    pub fn zero(&self) -> Result<(), MotorError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::motor_tare_position(self.port));
        }
        Ok(())
    }

    /// Stops the motor based on the current [`BrakeMode`]
    pub fn brake(&self) {
        unsafe {
            pros_sys::motor_brake(self.port);
        }
    }

    /// Sets the current position to the given position.
    pub fn set_zero_position(&self, position: Position) {
        unsafe {
            pros_sys::motor_set_zero_position(self.port, position.into_degrees());
        }
    }

    /// Sets how the motor should act when stopping.
    pub fn set_brake_mode(&self, brake_mode: BrakeMode) {
        unsafe {
            pros_sys::motor_set_brake_mode(self.port, brake_mode.into());
        }
    }

    //TODO: Test this, as im not entirely sure of the actual implementation
    /// Get the current state of the motor.
    pub fn get_state(&self) -> MotorState {
        unsafe { (pros_sys::motor_get_flags(self.port) as u32).into() }
    }

    /// Reverse this motor by multiplying all input by -1.
    pub fn set_reversed(&self, reversed: bool) {
        unsafe {
            pros_sys::motor_set_reversed(self.port, reversed);
        }
    }

    /// Check if this motor has been reversed.
    pub fn reversed(&self) -> bool {
        unsafe { pros_sys::motor_is_reversed(self.port) == 1 }
    }
}

/// Determines how a motor should act when braking.
pub enum BrakeMode {
    /// Motor never brakes.
    None,
    /// Motor brakes when stopped.
    Brake,
    /// Motor exerts force to hold the same position.
    Hold,
}

impl From<BrakeMode> for pros_sys::motor_brake_mode_e_t {
    fn from(other: BrakeMode) -> pros_sys::motor_brake_mode_e_t {
        match other {
            BrakeMode::Brake => pros_sys::E_MOTOR_BRAKE_BRAKE,
            BrakeMode::Hold => pros_sys::E_MOTOR_BRAKE_HOLD,
            BrakeMode::None => pros_sys::E_MOTOR_BRAKE_COAST,
        }
    }
}

/// Represents what the physical motor is currently doing.
pub struct MotorState {
    pub busy: bool,
    pub stopped: bool,
    /// the motor is at zero encoder units of rotation.
    pub zeroed: bool,
}

//TODO: Test this, like mentioned above
impl From<u32> for MotorState {
    fn from(value: u32) -> Self {
        Self {
            busy: (value & 0b001) == 0b001,
            stopped: (value & 0b010) == 0b010,
            zeroed: (value & 0b100) == 0b100,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum MotorError {
    #[snafu(display("The voltage supplied was outside of the allowed range (-12 to 12)."))]
    VoltageOutOfRange,
    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    MotorError {}
    inherit PortError;
}
