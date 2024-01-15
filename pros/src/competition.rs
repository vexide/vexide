//! Utilities for getting what state of the competition the robot is in.
//!
use pros_sys::misc::{COMPETITION_AUTONOMOUS, COMPETITION_CONNECTED, COMPETITION_DISABLED};

// TODO: change this to use PROS' internal version once we switch to PROS 4.
const COMPETITION_SYSTEM: u8 = 1 << 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionMode {
    Disabled,
    Opcontrol,
    Autonomous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompetitionSystem {
    FieldControl,
    CompetitionSwitch,
}

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

pub fn connected() -> bool {
    let status = unsafe { pros_sys::misc::competition_get_status() };

    status & COMPETITION_CONNECTED != 0
}

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
