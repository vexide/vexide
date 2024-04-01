//! Utilities for getting competition control state.

use bitflags::bitflags;
use vex_sdk::vexCompetitionStatus;

bitflags! {
    /// The status bits returned by [`competition::state`].
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct CompetitionStatus: u32 {
        /// Robot is connected to field control (NOT competition switch)
        const SYSTEM = 1 << 3;

        /// Robot is in autonomous mode.
        const AUTONOMOUS = 1 << 0;

        /// Robot is disabled by field control.
        const DISABLED = 1 << 1;

        /// Robot is connected to competition control (either competition switch or field control).
        const CONNECTED = 1 << 2;
    }
}

/// Represents a possible mode that robots can be set in during the competition lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionMode {
    /// The Disabled competition mode.
    ///
    /// When in disabled mode, voltage commands to motors are disabled. Motors are forcibly
    /// locked to the "coast" brake mode and cannot be moved.
    ///
    /// Robots may be placed into disabled mode at any point in the competition after
    /// connecting, but are typically disabled before the autonomous period, between
    /// autonomous and opcontrol periods, and following the opcontrol period of a match.
    Disabled,

    /// The Autonomous competition mode.
    ///
    /// When in autonomous mode, all motors and sensors may be accessed, however user
    /// input from controller buttons and joysticks is not available to be read.
    ///
    /// Robots may be placed into autonomous mode at any point in the competition after
    /// connecting, but are typically placed into this mode at the start of a match.
    Autonomous,

    /// The drivercontrol competition mode.
    ///
    /// When in opcontrol mode, all device access is available including access to
    /// controller joystick values for reading user-input from drive team members.
    ///
    /// Robots may be placed into opcontrol mode at any point in the competition after
    /// connecting, but are typically placed into this mode following the autonomous
    /// period.
    Driver,
}

/// Represents a type of system used to control competition state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionSystem {
    /// Competition state is controlled by a VEX Field Controller.
    FieldControl,

    /// Competition state is controlled by a VEXnet competition switch.
    CompetitionSwitch,
}

/// Gets the current competition status flags.
pub fn status() -> CompetitionStatus {
    CompetitionStatus::from_bits_retain(unsafe { vexCompetitionStatus() })
}

/// Gets the current competition mode, or phase.
pub fn mode() -> CompetitionMode {
    let status = status();

    if status.contains(CompetitionStatus::DISABLED) {
        CompetitionMode::Disabled
    } else if status.contains(CompetitionStatus::AUTONOMOUS) {
        CompetitionMode::Autonomous
    } else {
        CompetitionMode::Driver
    }
}

/// Checks if the robot is connected to a competition control system.
pub fn connected() -> bool {
    status().contains(CompetitionStatus::CONNECTED)
}

/// Gets the type of system currently controlling the robot's competition state, or [`None`] if the robot
/// is not tethered to a competition controller.
pub fn system() -> Option<CompetitionSystem> {
    let status = status();

    if status.contains(CompetitionStatus::CONNECTED) {
        if status.contains(CompetitionStatus::SYSTEM) {
            Some(CompetitionSystem::FieldControl)
        } else {
            Some(CompetitionSystem::CompetitionSwitch)
        }
    } else {
        None
    }
}
