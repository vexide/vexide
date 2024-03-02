//! Utilities for getting what state of the competition the robot is in.

use pros_sys::misc::{COMPETITION_AUTONOMOUS, COMPETITION_CONNECTED, COMPETITION_DISABLED};

// TODO: change this to use PROS' internal version once we switch to PROS 4.
const COMPETITION_SYSTEM: u8 = 1 << 3;

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

    /// The Opcontrol competition mode.
    ///
    /// When in opcontrol mode, all device access is available including access to
    /// controller joystick values for reading user-input from drive team members.
    ///
    /// Robots may be placed into opcontrol mode at any point in the competition after
    /// connecting, but are typically placed into this mode following the autonomous
    /// period.
    Opcontrol,
}

/// Represents a type of system used to control competition state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionSystem {
    /// Competition state is controlled by a VEX Field Controller.
    FieldControl,

    /// Competition state is controlled by a VEXnet competition switch.
    CompetitionSwitch,
}

/// Gets the current competition mode, or phase.
pub fn mode() -> CompetitionMode {
    let status = unsafe { pros_sys::misc::competition_get_status() };

    if status & COMPETITION_DISABLED != 0 {
        CompetitionMode::Disabled
    } else if status & COMPETITION_AUTONOMOUS != 0 {
        CompetitionMode::Autonomous
    } else {
        CompetitionMode::Opcontrol
    }
}

/// Checks if the robot is connected to a competition control system.
pub fn connected() -> bool {
    let status = unsafe { pros_sys::misc::competition_get_status() };

    status & COMPETITION_CONNECTED != 0
}

/// Gets the type of system currently controlling the robot's competition state, or [`None`] if the robot
/// is not tethered to a competition controller.
pub fn system() -> Option<CompetitionSystem> {
    let status = unsafe { pros_sys::misc::competition_get_status() };

    if status & COMPETITION_CONNECTED != 0 {
        if status & COMPETITION_SYSTEM == 0 {
            Some(CompetitionSystem::FieldControl)
        } else {
            Some(CompetitionSystem::CompetitionSwitch)
        }
    } else {
        None
    }
}
