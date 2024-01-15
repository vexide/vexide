//! Utilities for getting what state of the competition the robot is in.
//!
//! You have the option of getting the entire state ([`get_status`]), or checking a specific one ([`is_autonomous`], etc.).
//! Once a [`CompetitionStatus`] is created by [`get_status`] it will not be updated again.

/// The current status of the robot, allowing checks to be made
/// for autonomous, disabled, and connected states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompetitionStatus(pub u8);

/// Represents a type of system used to control competition state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionSystem {
    /// Competition state is controlled by a VEX Field Controller.
    FieldControl,

    // Competition state is controlled by a VEX competition switch.
    CompetitionSwitch,
}

// TODO: Change this to use competition_is_field and competition_is_switch once we support PROS 4
const COMPETITION_SYSTEM: u8 = 1 << 3;

impl CompetitionStatus {
    pub const fn autonomous(&self) -> bool {
        self.0 & pros_sys::misc::COMPETITION_AUTONOMOUS != 0
    }
    pub const fn disabled(&self) -> bool {
        self.0 & pros_sys::misc::COMPETITION_DISABLED != 0
    }
    pub const fn connected(&self) -> bool {
        self.0 & pros_sys::misc::COMPETITION_CONNECTED != 0
    }
    pub fn system(&self) -> Option<CompetitionSystem> {
        if self.connected() {
            if self.0 & COMPETITION_SYSTEM == 0 {
                Some(CompetitionSystem::FieldControl)
            } else {
                Some(CompetitionSystem::CompetitionSwitch)
            }
        } else {
            None
        }
    }
}

/// Get the current status of the robot.
pub fn get_status() -> CompetitionStatus {
    CompetitionStatus(unsafe { pros_sys::misc::competition_get_status() })
}

/// Get the type of system currently controlling the robot's competition state, or none if the robot
/// is not tethered to a competition controller.
pub fn get_system() -> Option<CompetitionSystem> {
    get_status().system()
}

/// Check if the robot is in autonomous mode.
pub fn is_autonomous() -> bool {
    unsafe { pros_sys::misc::competition_is_autonomous() }
}

/// Check if the robot is disabled.
pub fn is_disabled() -> bool {
    unsafe { pros_sys::misc::competition_is_disabled() }
}

/// Check if the robot is connected to a VEX field or competition switch.
pub fn is_connected() -> bool {
    unsafe { pros_sys::misc::competition_is_connected() }
}
