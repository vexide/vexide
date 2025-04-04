//! Smart Motors
//!
//! This module provides abstractions for interacting with VEX V5 Smart Motors, supporting both
//! the 11W and 5.5W variants.
//!
//! # Overview
//!
//! The V5 Smart Motors come in two variants: [an 11W model](https://www.vexrobotics.com/276-4840.html) with interchangeable gear cartridges
//! and [a 5.5W model](https://www.vexrobotics.com/276-4842.html) with a fixed gearing. The 11W motor supports three cartridge options: a red
//! cartridge providing 100 RPM, a green cartridge for 200 RPM, and a blue cartridge delivering
//! 600 RPM. The 5.5W motor comes with a non-interchangeable 200 RPM gear cartridge.
//!
//! Motor position and velocity is measured by an onboard integrated encoder.
//!
//! More in depth specs for the 11W motor can be found [here](https://kb.vex.com/hc/en-us/articles/360060929971-Understanding-V5-Smart-Motors).
//!
//! Communication between a Smart motor and the V5 Brain occur at two different intervals. While
//! the motor communicates with the Brain every 5 milliseconds (and commands can be written to
//! the motor every 5mS), the Brain only reads data from the motor every 10mS. This effectively
//! places the date *write* interval at 5mS and the data *read* interval at 10mS.
//!
//! # Current Limitations
//!
//! There are some cases where VEXos or the motor itself may decide to limit output current:
//!
//! - **Stall Prevention**: The stall current on 11W motors is limited to 2.5A. This
//!   limitation eliminates the need for automatic resetting fuses (PTC devices) in the motor, which
//!   can disrupt operation. By restricting the stall current to 2.5A, the motor effectively avoids
//!   undesirable performance dips and ensures that users do not inadvertently cause stall situations.
//!
//! - **Motor Count**: Robots that use 8 or fewer 11W motors will have the aforementioned current limit
//!   of 2.5A set for each motor. Robots using more than 8 motors, will have a lower default current limit
//!   per-motor than 2.5A. This limit is determined in VEXos by a calculation accounting for the number of
//!   motors plugged in, and the user's manually set current limits using [`Motor::set_current_limit`]. For
//!   more information regarding the current limiting behavior of VEXos, see [this forum post](https://www.vexforum.com/t/how-does-the-decreased-current-affect-the-robot-when-using-more-than-8-motors/72650/4).
//!
//! - **Temperature Management**: Motors have an onboard sensor for measuring internal temperature. If
//!   the motor determines that it is overheating, it will throttle its output current and warn the user.
//!
//! # Motor Control
//!
//! Each motor contains a sophisticated control system built around a Cortex M0 microcontroller.
//! The microcontroller continuously monitors position, speed, direction, voltage, current, and
//! temperature through integrated sensors.
//!
//! The onboard motor firmware implements a set of pre-tuned PID (Proportional-Integral-Derivative)
//! controllers operating on a 10-millisecond cycle for position and velocity control. Motors also
//! have braking functionality for holding a specific position under load.
//!
//! The [`Motor`] API can make use of these builtin control features through the [`MotorControl`] type,
//! which describes an action that the motor should perform.

use core::time::Duration;

use bitflags::bitflags;
use snafu::{ensure, Snafu};
use vex_sdk::{
    vexDeviceMotorAbsoluteTargetSet, vexDeviceMotorActualVelocityGet, vexDeviceMotorBrakeModeSet,
    vexDeviceMotorCurrentGet, vexDeviceMotorCurrentLimitGet, vexDeviceMotorCurrentLimitSet,
    vexDeviceMotorEfficiencyGet, vexDeviceMotorEncoderUnitsSet, vexDeviceMotorFaultsGet,
    vexDeviceMotorFlagsGet, vexDeviceMotorGearingGet, vexDeviceMotorGearingSet,
    vexDeviceMotorPositionGet, vexDeviceMotorPositionRawGet, vexDeviceMotorPositionReset,
    vexDeviceMotorPositionSet, vexDeviceMotorPowerGet, vexDeviceMotorReverseFlagGet,
    vexDeviceMotorReverseFlagSet, vexDeviceMotorTemperatureGet, vexDeviceMotorTorqueGet,
    vexDeviceMotorVelocitySet, vexDeviceMotorVelocityUpdate, vexDeviceMotorVoltageGet,
    vexDeviceMotorVoltageLimitGet, vexDeviceMotorVoltageLimitSet, vexDeviceMotorVoltageSet,
    V5MotorBrakeMode, V5MotorGearset, V5_DeviceT,
};
#[cfg(feature = "dangerous_motor_tuning")]
use vex_sdk::{vexDeviceMotorPositionPidSet, vexDeviceMotorVelocityPidSet, V5_DeviceMotorPid};

use super::{SmartDevice, SmartDeviceTimestamp, SmartDeviceType, SmartPort};
use crate::{position::Position, PortError};

/// A motor plugged into a Smart Port.
#[derive(Debug, PartialEq)]
pub struct Motor {
    port: SmartPort,
    target: MotorControl,
    device: V5_DeviceT,

    motor_type: MotorType,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for Motor {}
unsafe impl Sync for Motor {}

/// A possible target action for a [`Motor`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotorControl {
    /// The motor brakes using a specified [`BrakeMode`].
    Brake(BrakeMode),

    /// The motor outputs a raw voltage.
    ///
    /// # Fields
    ///
    /// - `0`: The desired output voltage of the motor
    Voltage(f64),

    /// The motor attempts to hold a velocity using its internal PID control.
    ///
    /// # Fields
    ///
    /// - `0`: The desired speed of the motor during the movement operation
    Velocity(i32),

    /// The motor attempts to reach a position using its internal PID control.
    ///
    /// # Fields
    ///
    /// - `0`: The desired position of the motor after the movement operation
    /// - `1`: The desired speed of the motor during the movement operation
    Position(Position, i32),
}

/// A possible direction that a motor can be configured as.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    /// Motor rotates in the forward direction.
    Forward,

    /// Motor rotates in the reverse direction.
    Reverse,
}

impl Direction {
    /// Returns `true` if the level is [`Forward`](Direction::Forward).
    #[must_use]
    pub const fn is_forward(&self) -> bool {
        match self {
            Self::Forward => true,
            Self::Reverse => false,
        }
    }

    /// Returns `true` if the level is [`Reverse`](Direction::Reverse).
    #[must_use]
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

/// Represents the type of a Smart motor.
/// Either a 11W (V5) or 5.5W (EXP) motor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorType {
    /// A 5.5W Smart Motor
    Exp,
    /// An 11W Smart Motor
    V5,
}
impl MotorType {
    /// Returns `true` if the motor is a 5.5W (EXP) Smart Motor.
    #[must_use]
    pub const fn is_exp(&self) -> bool {
        matches!(self, Self::Exp)
    }

    /// Returns `true` if the motor is an 11W (V5) Smart Motor.
    #[must_use]
    pub const fn is_v5(&self) -> bool {
        matches!(self, Self::V5)
    }

    /// Returns the maximum voltage for a motor of this type.
    #[must_use]
    pub const fn max_voltage(&self) -> f64 {
        match self {
            MotorType::Exp => Motor::EXP_MAX_VOLTAGE,
            MotorType::V5 => Motor::V5_MAX_VOLTAGE,
        }
    }
}

impl Motor {
    /// The maximum voltage value that can be sent to a V5 [`Motor`].
    pub const V5_MAX_VOLTAGE: f64 = 12.0;
    /// The maximum voltage value that can be sent to a EXP [`Motor`].
    pub const EXP_MAX_VOLTAGE: f64 = 8.0;

    /// The interval at which the Brain will send new packets to a [`Motor`].
    pub const WRITE_INTERVAL: Duration = Duration::from_millis(5);

    /// Create a new V5 or EXP motor.
    #[must_use]
    fn new_with_type(
        port: SmartPort,
        gearset: Gearset,
        direction: Direction,
        motor_type: MotorType,
    ) -> Self {
        let device = unsafe { port.device_handle() }; // SAFETY: This function is only called once on this port.

        // NOTE: SDK properly stores device state when unplugged, meaning that we can safely
        // set these without consequence even if the device is not available. This is an edge
        // case for the SDK though, and seems to just be a thing for motors and rotation sensors.
        unsafe {
            vexDeviceMotorEncoderUnitsSet(
                device,
                vex_sdk::V5MotorEncoderUnits::kMotorEncoderCounts,
            );

            vexDeviceMotorReverseFlagSet(device, direction.is_reverse());
            vexDeviceMotorGearingSet(device, gearset.into());
        }

        Self {
            port,
            target: MotorControl::Voltage(0.0),
            device,
            motor_type,
        }
    }

    /// Creates a new 11W (V5) Smart Motor.
    ///
    /// See [`Motor::new_exp`] to create a 5.5W (EXP) Smart Motor.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Red, Direction::Forward);
    ///     assert!(motor.is_v5());
    ///     assert_eq!(motor.max_voltage().unwrap(), Motor::V5_MAX_VOLTAGE);
    /// }
    #[must_use]
    pub fn new(port: SmartPort, gearset: Gearset, direction: Direction) -> Self {
        Self::new_with_type(port, gearset, direction, MotorType::V5)
    }
    /// Creates a new 5.5W (EXP) Smart Motor.
    ///
    /// See [`Motor::new`] to create a 11W (V5) Smart Motor.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new_exp(peripherals.port_1, Direction::Forward);
    ///     assert!(motor.is_exp());
    ///     assert_eq!(motor.max_voltage().unwrap(), Motor::EXP_MAX_VOLTAGE);
    /// }
    #[must_use]
    pub fn new_exp(port: SmartPort, direction: Direction) -> Self {
        Self::new_with_type(port, Gearset::Green, direction, MotorType::Exp)
    }

    /// Sets the target that the motor should attempt to reach.
    ///
    /// This could be a voltage, velocity, position, or even brake mode.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_target(MotorControl::Voltage(5.0));
    ///     sleep(Duration::from_secs(1)).await;
    ///     let _ = motor.set_target(MotorControl::Brake(BrakeMode::Hold));
    /// }
    /// ```
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
                // position will not reach large enough values to cause loss of precision during normal operation
                #[allow(clippy::cast_precision_loss)]
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
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.brake(BrakeMode::Hold);
    /// }
    /// ```
    pub fn brake(&mut self, mode: BrakeMode) -> Result<(), MotorError> {
        self.set_target(MotorControl::Brake(mode))
    }

    /// Spins the motor at a target velocity.
    ///
    /// This velocity corresponds to different actual speeds in RPM depending on the gearset used for the motor.
    /// Velocity is held with an internal PID controller to ensure consistent speed, as opposed to setting the
    /// motor's voltage.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Spin a motor at 100 RPM:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_velocity(100);
    ///     sleep(Duration::from_secs(1)).await;
    /// }
    /// ```
    pub fn set_velocity(&mut self, rpm: i32) -> Result<(), MotorError> {
        self.set_target(MotorControl::Velocity(rpm))
    }

    /// Sets the motor's output voltage.
    ///
    /// This voltage value spans from -12 (fully spinning reverse) to +12 (fully spinning forwards) volts, and
    /// controls the raw output of the motor.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Give the motor full power:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut v5_motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let mut exp_motor = Motor::new_exp(peripherals.port_2, Direction::Forward);
    ///     let _ = v5_motor.set_voltage(v5_motor.max_voltage());
    ///     let _ = exp_motor.set_voltage(exp_motor.max_voltage());
    /// }
    /// ```
    ///
    /// Drive the motor based on a controller joystick:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let controller = peripherals.primary_controller;
    ///     loop {
    ///         let controller_state = controller.state().unwrap_or_default();
    ///         let voltage = controller_state.left_stick.x() * motor.max_voltage();
    ///         motor.set_voltage(voltage).unwrap();
    ///     }
    /// }
    /// ```
    pub fn set_voltage(&mut self, volts: f64) -> Result<(), MotorError> {
        self.set_target(MotorControl::Voltage(volts))
    }

    /// Sets an absolute position target for the motor to attempt to reach.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    ///
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_position_target(Position::from_degrees(90.0), 200);
    /// }
    /// ```
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
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     // Set the motor's target to a Position so that changing the velocity isn't a noop.
    ///     let _ = motor.set_target(MotorControl::Position(Position::from_degrees(90.0), 200));
    ///     let _ = motor.set_profiled_velocity(100).unwrap();
    /// }
    /// ```
    pub fn set_profiled_velocity(&mut self, velocity: i32) -> Result<(), MotorError> {
        self.validate_port()?;

        unsafe {
            vexDeviceMotorVelocityUpdate(self.device, velocity);
        }

        if let MotorControl::Position(position, _) = self.target {
            self.target = MotorControl::Position(position, velocity);
        }

        Ok(())
    }

    /// Returns the current [`MotorControl`] target that the motor is attempting to use.
    /// This value is set with [`Motor::set_target`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     motor.set_target(MotorControl::Brake(BrakeMode::Hold));
    ///     let target = motor.target();
    ///     assert_eq!(target, MotorControl::Brake(BrakeMode::Hold));
    /// }
    #[must_use]
    pub const fn target(&self) -> MotorControl {
        self.target
    }

    /// Sets the gearset of an 11W motor.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    /// - A [`MotorError::SetGearsetExp`] is returned if the motor is a 5.5W EXP Smart Motor, which has no swappable gearset.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     // This must be a V5 motor
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///
    ///     // Set the motor to use the red gearset
    ///     motor.set_gearset(Gearset::Red).unwrap();
    /// }
    /// ```
    pub fn set_gearset(&mut self, gearset: Gearset) -> Result<(), MotorError> {
        ensure!(self.motor_type.is_v5(), SetGearsetExpSnafu);
        self.validate_port()?;
        unsafe {
            vexDeviceMotorGearingSet(self.device, gearset.into());
        }
        Ok(())
    }

    /// Returns the gearset of the motor
    ///
    /// For 5.5W motors, this will always be returned as [`Gearset::Green`].
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print the gearset of a motor:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// fn print_gearset(motor: &Motor) {
    ///     let Ok(gearset) = motor.gearset() else {
    ///         println!("Failed to get gearset. Is this an EXP motor?");
    ///         return;
    ///     };
    ///     match motor.gearset() {
    ///         Gearset::Green => println!("Motor is using the green gearset"),
    ///         Gearset::Red => println!("Motor is using the red gearset"),
    ///         Gearset::Blue => println!("Motor is using the blue gearset"),
    ///    }
    /// }
    ///
    pub fn gearset(&self) -> Result<Gearset, MotorError> {
        if self.motor_type.is_exp() {
            return Ok(Gearset::Green);
        }
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorGearingGet(self.device) }.into())
    }

    /// Returns the type of the motor.
    /// This does not check the hardware, it simply returns the type that the motor was created with.
    ///
    /// # Examples
    ///
    /// Match based on motor type:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// fn print_motor_type(motor: &Motor) {
    ///     match motor.motor_type() {
    ///         MotorType::Exp => println!("Motor is a 5.5W EXP Smart Motor"),
    ///         MotorType::V5 => println!("Motor is an 11W V5 Smart Motor"),
    ///     }
    /// }
    /// ```
    #[must_use]
    pub const fn motor_type(&self) -> MotorType {
        self.motor_type
    }
    /// Returns `true` if the motor is a 5.5W (EXP) Smart Motor.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new_exp(peripherals.port_1, Direction::Forward);
    ///     if motor.is_exp() {
    ///         println!("Motor is a 5.5W EXP Smart Motor");
    ///     }
    /// }
    /// ```
    #[must_use]
    pub const fn is_exp(&self) -> bool {
        self.motor_type.is_exp()
    }
    /// Returns `true` if the motor is an 11W (V5) Smart Motor.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor:: new(peripherals.port_1, Gearset::Red, Direction::Forward);
    ///     if motor.is_v5() {
    ///         println!("Motor is an 11W V5 Smart Motor");
    ///     }
    /// }
    /// ```
    #[must_use]
    pub const fn is_v5(&self) -> bool {
        self.motor_type.is_v5()
    }

    /// Returns the maximum voltage for the motor based off of its [motor type](Motor::motor_type).
    ///
    /// # Examples
    ///
    /// Run a motor at max speed, agnostic of its type:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// fn run_motor_at_max_speed(motor: &mut Motor) {
    ///     motor.set_voltage(motor.max_voltage()).unwrap();
    /// }
    #[must_use]
    pub const fn max_voltage(&self) -> f64 {
        self.motor_type.max_voltage()
    }

    /// Returns the motor's estimate of its angular velocity in rotations per minute (RPM).
    ///
    /// # Accuracy
    ///
    /// In some cases, this reported value may be noisy or inaccurate, especially for systems where accurate
    /// velocity control at high speeds is required (such as flywheels). If the accuracy of this value proves
    /// inadequate, you may opt to perform your own velocity calculations by differentiating [`Motor::position`]
    /// over the reported internal timestamp of the motor using [`Motor::timestamp`].
    ///
    /// > For more information about Smart motor velocity estimation, see [this article](https://sylvie.fyi/sylib/docs/db/d8e/md_module_writeups__velocity__estimation.html).
    ///
    /// # Note
    ///
    /// To get the current **target** velocity instead of the estimated velocity, use [`Motor::target`].
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Get the current velocity of a motor:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///
    ///     println!("{:?}", motor.velocity().unwrap());
    /// }
    /// ```
    ///
    /// Calculate acceleration of a motor:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///
    ///     let mut last_velocity = motor.velocity().unwrap();
    ///     let mut start_time = Instant::now();
    ///     loop {
    ///         let velocity = motor.velocity().unwrap();
    ///         // Make sure we don't divide by zero
    ///         let elapsed = start_time.elapsed().as_secs_f64() + 0.001;
    ///
    ///         // Calculate acceleration
    ///         let acceleration = (velocity - last_velocity) / elapsed;
    ///         println!("Velocity: {:.2} RPM, Acceleration: {:.2} RPM/s", velocity, acceleration);
    ///
    ///         last_velocity = velocity;
    ///         start_time = Instant::now();
    ///    }
    /// }
    /// ```
    pub fn velocity(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorActualVelocityGet(self.device) })
    }

    /// Returns the power drawn by the motor in Watts.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print the power drawn by a motor:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     loop {
    ///         println!("Power: {:.2}W", motor.power().unwrap());
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn power(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorPowerGet(self.device) })
    }

    /// Returns the torque output of the motor in Nm.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print the torque output of a motor:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     loop {
    ///         println!("Torque: {:.2}Nm", motor.torque().unwrap());
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn torque(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorTorqueGet(self.device) })
    }

    /// Returns the voltage the motor is drawing in volts.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print the voltage drawn by a motor:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     loop {
    ///         println!("Voltage: {:.2}V", motor.voltage().unwrap());
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn voltage(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(f64::from(unsafe { vexDeviceMotorVoltageGet(self.device) }) / 1000.0)
    }

    /// Returns the current position of the motor.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print the current position of a motor:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     loop {
    ///         println!("Position: {:?}", motor.position().unwrap());
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
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
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     loop {
    ///         let (raw_pos, _) = motor.raw_position().unwrap();
    ///         println!("Raw Position: {}", raw_pos);
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn raw_position(&self) -> Result<(i32, SmartDeviceTimestamp), MotorError> {
        self.validate_port()?;

        let mut timestamp: u32 = 0;
        let ticks = unsafe { vexDeviceMotorPositionRawGet(self.device, &mut timestamp) };

        Ok((ticks, SmartDeviceTimestamp(timestamp)))
    }

    /// Returns the electrical current draw of the motor in amps.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print the current draw of a motor:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     motor.set_voltage(motor.max_voltage()).unwrap();
    ///     loop {
    ///         println!("Current: {:.2}A", motor.current().unwrap());
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn current(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(f64::from(unsafe { vexDeviceMotorCurrentGet(self.device) }) / 1000.0)
    }

    /// Returns the efficiency of the motor from a range of [0.0, 1.0].
    ///
    /// An efficiency of 1.0 means that the motor is moving electrically while
    /// drawing no electrical power, and an efficiency of 0.0 means that the motor
    /// is drawing power but not moving.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print the efficiency of a motor:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_voltage(motor.max_voltage())
    ///     loop {
    ///         println!("Efficiency: {:.2}", motor.efficiency().unwrap());
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn efficiency(&self) -> Result<f64, MotorError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceMotorEfficiencyGet(self.device) } / 100.0)
    }

    /// Sets the current encoder position to zero without moving the motor.
    ///
    /// Analogous to taring or resetting the encoder to the current position.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Move the motor in increments of 10 degrees:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     loop {
    ///         motor.set_position_target(Position::from_degrees(10.0), 200).unwrap();
    ///         sleep(Duration::from_secs(1)).await;
    ///         motor.reset_position().unwrap();
    ///     }
    /// }
    /// ```
    pub fn reset_position(&mut self) -> Result<(), MotorError> {
        self.validate_port()?;
        unsafe { vexDeviceMotorPositionReset(self.device) }
        Ok(())
    }

    /// Sets the current encoder position to the given position without moving the motor.
    ///
    /// Analogous to taring or resetting the encoder so that the new position is equal to the given position.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Set the current position of the motor to 90 degrees:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     motor.set_position(Position::from_degrees(90.0)).unwrap();
    /// }
    /// ```
    ///
    /// Reset the position of the motor to 0 degrees (analogous to [`reset_position`](Motor::reset_position)):
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     motor.set_position(Position::from_degrees(0.0)).unwrap();
    /// }
    /// ```
    #[allow(clippy::cast_precision_loss)]
    pub fn set_position(&mut self, position: Position) -> Result<(), MotorError> {
        let gearset = self.gearset()?;

        unsafe {
            vexDeviceMotorPositionSet(
                self.device,
                // NOTE: No precision loss since ticks are not fractional.
                position.as_ticks(gearset.ticks_per_revolution()) as f64,
            );
        }

        Ok(())
    }

    /// Sets the current limit for the motor in amps.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Limit the current draw of a motor to 2.5A:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_current_limit(2.5);
    /// }
    /// ```
    pub fn set_current_limit(&mut self, limit: f64) -> Result<(), MotorError> {
        self.validate_port()?;
        unsafe { vexDeviceMotorCurrentLimitSet(self.device, (limit * 1000.0) as i32) }
        Ok(())
    }

    /// Sets the voltage limit for the motor in volts.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Limit the voltage of a motor to 4V:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_voltage_limit(4.0);
    ///     // Will appear as if the voltage was set to only 4V
    ///     let _ = motor.set_voltage(12.0);
    /// }
    /// ```
    pub fn set_voltage_limit(&mut self, limit: f64) -> Result<(), MotorError> {
        self.validate_port()?;

        unsafe {
            vexDeviceMotorVoltageLimitSet(self.device, (limit * 1000.0) as i32);
        }

        Ok(())
    }

    /// Returns the current limit for the motor in amps.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print the current limit of a motor:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     println!("Current Limit: {:.2}A", motor.current_limit().unwrap());
    /// }
    /// ```
    pub fn current_limit(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(f64::from(unsafe { vexDeviceMotorCurrentLimitGet(self.device) }) / 1000.0)
    }

    /// Returns the voltage limit for the motor if one has been explicitly set.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print the voltage limit of a motor:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     println!("Voltage Limit: {:.2}V", motor.voltage_limit().unwrap());
    /// }
    /// ```
    pub fn voltage_limit(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(f64::from(unsafe { vexDeviceMotorVoltageLimitGet(self.device) }) / 1000.0)
    }

    /// Returns the internal temperature recorded by the motor in increments of 5 °C.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Turn off the motor if it gets too hot:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_voltage(12.0);
    ///     loop {
    ///         if motor.temperature().unwrap() > 30.0 {
    ///             let _ = motor.brake(BrakeMode::Coast);
    ///         } else {
    ///             let _ = motor.set_voltage(12.0);
    ///         }
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn temperature(&self) -> Result<f64, MotorError> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceMotorTemperatureGet(self.device) })
    }

    /// Returns the status flags of a motor.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Check if a motor is "busy" (busy only occurs if communicating with the motor fails)
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// fn is_motor_busy(motor: &Motor) -> bool {
    ///     motor.status().unwrap().contains(MotorStatus::BUSY)
    /// }
    /// ```
    pub fn status(&self) -> Result<MotorStatus, MotorError> {
        self.validate_port()?;

        Ok(MotorStatus::from_bits_retain(unsafe {
            vexDeviceMotorFlagsGet(self.device)
        }))
    }

    /// Returns the fault flags of the motor.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Check if a motor is over temperature:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     loop {
    ///         let faults = motor.faults().unwrap();
    ///         println!("Faults: {:?}", faults);
    ///
    ///         if faults.contains(MotorFaults::OVER_TEMPERATURE) {
    ///             println!("Warning: Motor is over temperature");
    ///         }
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn faults(&self) -> Result<MotorFaults, MotorError> {
        self.validate_port()?;

        Ok(MotorFaults::from_bits_retain(unsafe {
            vexDeviceMotorFaultsGet(self.device)
        }))
    }

    /// Returns `true` if the motor's over temperature flag is set.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Turn off the motor if it gets too hot:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_voltage(12.0);
    ///     loop {
    ///         if let Ok(true) = motor.is_over_temperature() {
    ///             let _ = motor.brake(BrakeMode::Coast);
    ///         } else {
    ///             let _ = motor.set_voltage(12.0);
    ///         }
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn is_over_temperature(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.contains(MotorFaults::OVER_TEMPERATURE))
    }

    /// Returns `true` if the motor's over-current flag is set.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print a warning if the motor draws too much current:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_voltage(12.0);
    ///     loop {
    ///         if let Ok(true) = motor.is_over_current() {
    ///             println!("Warning: Motor is drawing too much current");
    ///         }
    ///         println!("Current: {:.2}A", motor.current().unwrap_or(0.0));
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn is_over_current(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.contains(MotorFaults::OVER_CURRENT))
    }

    /// Returns `true` if a H-bridge (motor driver) fault has occurred.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print a warning if the motor's H-bridge has a fault:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_voltage(12.0);
    ///     loop {
    ///         if let Ok(true) = motor.is_driver_fault() {
    ///             println!("Warning: Motor has a H-bridge fault");
    ///         }
    ///         println!("Current: {:.2}A", motor.current().unwrap_or(0.0));
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn is_driver_fault(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.contains(MotorFaults::DRIVER_FAULT))
    }

    /// Returns `true` if the motor's H-bridge has an over-current fault.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Print a warning if it draws too much current:
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let _ = motor.set_voltage(12.0);
    ///     loop {
    ///         if let Ok(true) = motor.is_driver_over_current() {
    ///             println!("Warning: Motor is drawing too much current");
    ///         }
    ///         println!("Current: {:.2}A", motor.current().unwrap_or(0.0));
    ///         sleep(Motor::UPDATE_INTERVAL).await;
    ///    }
    /// }
    /// ```
    pub fn is_driver_over_current(&self) -> Result<bool, MotorError> {
        Ok(self.faults()?.contains(MotorFaults::OVER_CURRENT))
    }

    /// Sets the motor to operate in a given [`Direction`].
    ///
    /// This determines which way the motor considers to be “forwards”. You can use the marking on the back of the
    /// motor as a reference:
    ///
    /// - When [`Direction::Forward`] is specified, positive velocity/voltage values will cause the motor to rotate
    ///   **with the arrow on the back**. Position will increase as the motor rotates **with the arrow**.
    /// - When [`Direction::Reverse`] is specified, positive velocity/voltage values will cause the motor to rotate
    ///   **against the arrow on the back**. Position will increase as the motor rotates **against the arrow**.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     motor.set_direction(Direction::Reverse).unwrap();
    /// }
    /// ```
    pub fn set_direction(&mut self, direction: Direction) -> Result<(), MotorError> {
        self.validate_port()?;

        unsafe {
            vexDeviceMotorReverseFlagSet(self.device, direction.is_reverse());
        }

        Ok(())
    }

    /// Returns the [`Direction`] of this motor.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// fn print_motor_direction(motor: &Motor) {
    ///     match motor.direction().unwrap() {
    ///         Direction::Forward => println!("Motor is set to forwards"),
    ///         Direction::Reverse => println!("Motor is set to reverse"),
    ///     }
    /// }
    /// ```
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
    /// to Smart motors if done incorrectly. Use these functions entirely at your own risk.
    ///
    /// VEX has chosen not to disclose the default constants used by Smart motors, and currently
    /// has no plans to do so. As such, the units and finer details of [`MotorTuningConstants`] are not
    /// well-known or understood, as we have no reference for what these constants should look
    /// like.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let constants = MotorTuningConstants {
    ///         kf: 0.0,
    ///         kp: 0.0,
    ///         ki: 0.0,
    ///         kd: 0.0,
    ///         filter: 0.0,
    ///         integral_limit: 0.0,
    ///         tolerance: 0.0,
    ///         sample_rate: 0.0,
    ///     };
    ///     motor.set_velocity_tuning_constants(constants).unwrap();
    /// }
    /// ```
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
    /// to Smart motors if done incorrectly. Use these functions entirely at your own risk.
    ///
    /// VEX has chosen not to disclose the default constants used by Smart motors, and currently
    /// has no plans to do so. As such, the units and finer details of [`MotorTuningConstants`] are not
    /// well-known or understood, as we have no reference for what these constants should look
    /// like.
    ///
    /// # Errors
    ///
    /// - A [`MotorError::Port`] error is returned if a motor device is not currently connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut motor = Motor::new(peripherals.port_1, Gearset::Green, Direction::Forward);
    ///     let constants = MotorTuningConstants {
    ///         kf: 0.0,
    ///         kp: 0.0,
    ///         ki: 0.0,
    ///         kd: 0.0,
    ///         filter: 0.0,
    ///         integral_limit: 0.0,
    ///         tolerance: 0.0,
    ///         sample_rate: 0.0,
    ///     };
    ///     motor.set_position_tuning_constants(constants).unwrap();
    /// }
    /// ```
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
impl From<Motor> for SmartPort {
    fn from(device: Motor) -> Self {
        device.port
    }
}

/// Determines the behavior a motor should use when braking with [`Motor::brake`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum BrakeMode {
    /// Motor never brakes.
    Coast,

    /// Motor uses regenerative braking to slow down faster.
    Brake,

    /// Motor exerts force holding itself in the same position.
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
        /// Failed to communicate with the motor
        const BUSY = 0x01;
    }
}

/// Internal gearset used by VEX Smart motors.
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

    /// Rated max speed for a Smart motor with a [`Red`](Gearset::Red) gearset.
    pub const MAX_RED_RPM: f64 = 100.0;
    /// Rated speed for a Smart motor with a [`Green`](Gearset::Green) gearset.
    pub const MAX_GREEN_RPM: f64 = 200.0;
    /// Rated speed for a Smart motor with a [`Blue`](Gearset::Blue) gearset.
    pub const MAX_BLUE_RPM: f64 = 600.0;

    /// Number of encoder ticks per revolution for the [`Red`](Gearset::Red) gearset.
    pub const RED_TICKS_PER_REVOLUTION: u32 = 1800;
    /// Number of encoder ticks per revolution for the [`Green`](Gearset::Green) gearset.
    pub const GREEN_TICKS_PER_REVOLUTION: u32 = 900;
    /// Number of encoder ticks per revolution for the [`Blue`](Gearset::Blue) gearset.
    pub const BLUE_TICKS_PER_REVOLUTION: u32 = 300;

    /// Returns the rated maximum speed for this motor gearset.
    #[must_use]
    pub const fn max_rpm(&self) -> f64 {
        match self {
            Self::Red => Self::MAX_RED_RPM,
            Self::Green => Self::MAX_GREEN_RPM,
            Self::Blue => Self::MAX_BLUE_RPM,
        }
    }

    /// Returns the number of encoder ticks per revolution for this motor gearset.
    #[must_use]
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
/// to Smart motors if done incorrectly. Use these functions entirely at your own risk.
///
/// VEX has chosen not to disclose the default constants used by Smart motors, and currently
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
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },

    /// EXP motors do not have customizable gearsets.
    SetGearsetExp,
}
