use crate::bindings;

/// The basic motor struct.
pub struct Motor {
    port: u8,
}

impl Motor {
    pub fn new(port: u8, brake_mode: BrakeMode) -> Self {
        unsafe {
            bindings::motor_set_encoder_units(port, bindings::motor_encoder_units_e_E_MOTOR_ENCODER_DEGREES);
            bindings::motor_set_brake_mode(port, brake_mode.into());
        }
        Self { port }
    }

    /// Takes in and i8 between -127 and 127 which is scaled to -12 to 12 Volts.
    /// Useful for driving motors with controllers.
    pub fn set_raw_output(&self, raw_output: i8) -> Result<(), MotorError> {
        unsafe {
            match bindings::motor_move(self.port, raw_output as i32) as u32 {
                1 => Ok(()),
                err => Err(MotorError::from(err)),
            }
        }
    }

    /// Takes in a voltage that must be between -12 and 12 Volts.
    pub fn set_voltage(&self, voltage: f32) -> Result<(), MotorError> {
        if voltage > 12.0 || voltage < -12.0 || voltage == f32::NAN {
            return Err(MotorError::VoltageOutOfRange);
        }
        unsafe {
            match bindings::motor_move_voltage(self.port, (voltage * 1000.0) as i32) as u32 {
                1 => Ok(()),
                err => Err(MotorError::from(err)),
            }
        }
    }

    /// Moves the motor to an absolute position, based off of the last motor zeroing.
    /// units for the velocity is RPM.
    pub fn set_position_absolute(
        &self,
        position: Position,
        velocity: i32,
    ) -> Result<(), MotorError> {
        unsafe {
            match bindings::motor_move_absolute(self.port, position.into(), velocity) as u32 {
                1 => Ok(()),
                err => Err(MotorError::from(err)),
            }
        }
    }

    /// Moves the motor to a position relative to the current position.
    /// units for velocity is RPM.
    pub fn set_position_relative(
        &self,
        position: Position,
        velocity: i32,
    ) -> Result<(), MotorError> {
        unsafe {
            match bindings::motor_move_relative(self.port, position.into(), velocity) as u32 {
                1 => Ok(()),
                err => Err(MotorError::from(err)),
            }
        }
    }

    /// Stops the motor based on the current [`BrakeMode`]
    pub fn brake(&self) -> Result<(), MotorError> {
        unsafe {
            match bindings::motor_brake(self.port) as u32 {
                1 => Ok(()),
                err => Err(MotorError::from(err)),
            }
        }
    }

    pub fn set_brake_mode(&self, brake_mode: BrakeMode) -> Result<(), MotorError> {
        unsafe {
            match bindings::motor_set_brake_mode(self.port, brake_mode.into()) as u32 {
                1 => Ok(()),
                err => Err(MotorError::from(err)),
            }
        }
    }

    //TODO: Test this, as im not entirely sure of the actuall implementation
    pub fn get_state(&self) -> Result<MotorState, MotorError> {
        unsafe {
            match bindings::motor_get_flags(self.port) as u32 {
                bindings::PROS_ERR => Err(MotorError::Unknown),
                bindings::ENXIO => Err(MotorError::PortOutOfRange),
                bindings::ENODEV => Err(MotorError::PortCannotBeConfigured),
                state => {
                    if let Ok(state) = state.try_into() {
                        Ok(state)
                    } else {
                        Err(MotorError::Unknown)
                    }
                }
            }
        }
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

impl Into<bindings::motor_brake_mode_e_t> for BrakeMode {
    fn into(self) -> bindings::motor_brake_mode_e_t {
        match self {
            Self::Brake => bindings::motor_brake_mode_e_E_MOTOR_BRAKE_BRAKE,
            Self::Hold => bindings::motor_brake_mode_e_E_MOTOR_BRAKE_HOLD,
            Self::None => bindings::motor_brake_mode_e_E_MOTOR_BRAKE_COAST,
        }
    }
}

pub enum Position {
    Degrees(f64),
    Rotations(f64),
    // Raw encoder ticks.
    Counts(i64),
}

impl Into<f64> for Position {
    fn into(self) -> f64 {
        match self {
            Self::Degrees(num) => num,
            Self::Rotations(num) => num * 360.0,
            Self::Counts(num) => num as f64 * (360.0 / 4096.0), //TODO: Measure the number of counts per rotation. Fow now we assume it is 4096
        }
    }
}

/// Represents what the physical motor is currently doing.
#[repr(i32)]
pub enum MotorState {
    None = 0,
    Busy = 1,
    Stopped = 2,
    /// the motor is at zero encoder units of rotation.
    Zeroed = 4,
}

//TODO: Test this, like mentioned above
impl TryFrom<u32> for MotorState {
    type Error = MotorError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Busy),
            2 => Ok(Self::Stopped),
            4 => Ok(Self::Zeroed),
            _ => Err(MotorError::Unknown),
        }
    }
}

pub enum MotorError {
    PortOutOfRange,
    PortCannotBeConfigured,
    VoltageOutOfRange,
    Unknown,
}

impl From<u32> for MotorError {
    fn from(value: u32) -> Self {
        match value {
            bindings::PROS_ERR => MotorError::Unknown,
            bindings::ENXIO => MotorError::PortOutOfRange,
            bindings::ENODEV => MotorError::PortCannotBeConfigured,
            _ => MotorError::Unknown,
        }
    }
}
