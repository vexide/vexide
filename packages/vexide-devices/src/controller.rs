//! Controller support.
//!
//! This module allows you to read from the buttons and joysticks on the controller and write to the controller's display.

use alloc::ffi::CString;
use core::time::Duration;

use snafu::Snafu;
use vex_sdk::{
    vexControllerConnectionStatusGet, vexControllerGet, vexControllerTextSet, V5_ControllerId,
    V5_ControllerIndex, V5_ControllerStatus,
};
use vexide_core::{competition, competition::CompetitionMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents the state of a button on the controller.
pub enum ButtonState {
    /// The button is pressed.
    Pressed,
    /// The button is released.
    Released,
}
impl ButtonState {
    /// Returns true if the button state is [`Pressed`](ButtonState::Pressed).
    pub const fn is_pressed(&self) -> bool {
        matches!(self, Self::Pressed)
    }
    /// Returns true if the button state is [`Released`](ButtonState::Released).
    pub const fn is_released(&self) -> bool {
        matches!(self, Self::Released)
    }
}

/// Stores how far the joystick is away from the center (at *(0, 0)*) from -1 to 1.
/// On the x axis left is negative, and right is positive.
/// On the y axis down is negative, and up is positive.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Joystick {
    x_raw: i8,
    y_raw: i8,
}
impl Joystick {
    /// Gets the value of the joystick position on its x-axis from [-1, 1].
    pub fn x(&self) -> f64 {
        self.x_raw as f64 / 127.0
    }
    /// Gets the value of the joystick position on its y-axis from [-1, 1].
    pub fn y(&self) -> f64 {
        self.y_raw as f64 / 127.0
    }

    /// The raw value of the joystick position on its x-axis from [-128, 127].
    pub const fn x_raw(&self) -> i8 {
        self.x_raw
    }
    /// The raw value of the joystick position on its x-axis from [-128, 127].
    pub const fn y_raw(&self) -> i8 {
        self.y_raw
    }
}

/// Holds a snapshot of the state of the controller.
/// Returned by [`Controller::state`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ControllerState {
    /// Left Joystick
    pub left_stick: Joystick,
    /// Right Joystick
    pub right_stick: Joystick,

    /// Button A
    pub button_a: ButtonState,
    /// Button B
    pub button_b: ButtonState,
    /// Button X
    pub button_x: ButtonState,
    /// Button Y
    pub button_y: ButtonState,

    /// Button Up
    pub button_up: ButtonState,
    /// Button Down
    pub button_down: ButtonState,
    /// Button Left
    pub button_left: ButtonState,
    /// Button Right
    pub button_right: ButtonState,

    /// Top Left Trigger
    pub left_trigger_1: ButtonState,
    /// Bottom Left Trigger
    pub left_trigger_2: ButtonState,
    /// Top Right Trigger
    pub right_trigger_1: ButtonState,
    /// Bottom Right Trigger
    pub right_trigger_2: ButtonState,
}
impl Default for ControllerState {
    fn default() -> Self {
        Self {
            left_stick: Default::default(),
            right_stick: Default::default(),
            button_a: ButtonState::Released,
            button_b: ButtonState::Released,
            button_x: ButtonState::Released,
            button_y: ButtonState::Released,
            button_up: ButtonState::Released,
            button_down: ButtonState::Released,
            button_left: ButtonState::Released,
            button_right: ButtonState::Released,
            left_trigger_1: ButtonState::Released,
            left_trigger_2: ButtonState::Released,
            right_trigger_1: ButtonState::Released,
            right_trigger_2: ButtonState::Released,
        }
    }
}

fn validate_connection(id: ControllerId) -> Result<(), ControllerError> {
    if unsafe {
        vexControllerConnectionStatusGet(id.into()) == V5_ControllerStatus::kV5ControllerOffline
    } {
        return Err(ControllerError::Offline);
    }

    Ok(())
}

/// The basic type for a controller.
/// Used to get the state of its joysticks and controllers.
#[derive(Debug, Eq, PartialEq)]
pub struct Controller {
    id: ControllerId,

    /// Controller Screen
    pub screen: ControllerScreen,
}

impl Controller {
    /// Returns the current state of all buttons and joysticks on the controller.
    /// # Note
    /// If the current competition mode is not driver control, this function will error.
    pub fn state(&self) -> Result<ControllerState, ControllerError> {
        if competition::mode() != CompetitionMode::Driver {
            return Err(ControllerError::CompetitionControl);
        }
        validate_connection(self.id)?;

        let button_level = |channel: V5_ControllerIndex| {
            let value = unsafe { vexControllerGet(self.id.into(), channel) != 0 };
            match value {
                true => ButtonState::Pressed,
                false => ButtonState::Released,
            }
        };

        Ok(ControllerState {
            left_stick: Joystick {
                x_raw: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Axis4) as _ },
                y_raw: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Axis3) as _ },
            },
            right_stick: Joystick {
                x_raw: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Axis1) as _ },
                y_raw: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Axis2) as _ },
            },
            button_a: button_level(V5_ControllerIndex::ButtonA),
            button_b: button_level(V5_ControllerIndex::ButtonB),
            button_x: button_level(V5_ControllerIndex::ButtonX),
            button_y: button_level(V5_ControllerIndex::ButtonY),
            button_up: button_level(V5_ControllerIndex::ButtonUp),
            button_down: button_level(V5_ControllerIndex::ButtonDown),
            button_left: button_level(V5_ControllerIndex::ButtonLeft),
            button_right: button_level(V5_ControllerIndex::ButtonRight),
            left_trigger_1: button_level(V5_ControllerIndex::ButtonL1),
            left_trigger_2: button_level(V5_ControllerIndex::ButtonL2),
            right_trigger_1: button_level(V5_ControllerIndex::ButtonR1),
            right_trigger_2: button_level(V5_ControllerIndex::ButtonR2),
        })
    }
}

/// Controller LCD Console
#[derive(Debug, Eq, PartialEq)]
pub struct ControllerScreen {
    id: ControllerId,
}

impl ControllerScreen {
    /// Maximum number of characters that can be drawn to a text line.
    pub const MAX_LINE_LENGTH: usize = 14;

    /// Number of available text lines on the controller before clearing the screen.
    pub const MAX_LINES: usize = 2;

    /// Clear the contents of a specific text line.
    pub fn clear_line(&mut self, line: u8) -> Result<(), ControllerError> {
        //TODO: Older versions of VexOS clear the controller by setting the line to "                   ".
        //TODO: We should check the version and change behavior based on it.
        self.set_text("", line, 0)?;

        Ok(())
    }

    /// Clear the whole screen.
    pub fn clear_screen(&mut self) -> Result<(), ControllerError> {
        for line in 0..Self::MAX_LINES as u8 {
            self.clear_line(line)?;
        }

        Ok(())
    }

    /// Set the text contents at a specific row/column offset.
    pub fn set_text(&mut self, text: &str, line: u8, col: u8) -> Result<(), ControllerError> {
        validate_connection(self.id)?;
        if col >= Self::MAX_LINE_LENGTH as u8 {
            return Err(ControllerError::InvalidLine);
        }

        let id: V5_ControllerId = self.id.into();
        let text = CString::new(text)
            .map_err(|_| ControllerError::NonTerminatingNul)?
            .into_raw();

        unsafe {
            vexControllerTextSet(id.0 as _, (line + 1) as _, (col + 1) as _, text as *const _);
        }

        // stop rust from leaking the CString
        drop(unsafe { CString::from_raw(text) });

        Ok(())
    }
}

/// Represents an identifier for one of the two possible controllers
/// connected to the V5 brain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerId {
    /// Primary ("Master") Controller
    Primary,

    /// Partner Controller
    Partner,
}

impl From<ControllerId> for V5_ControllerId {
    fn from(id: ControllerId) -> Self {
        match id {
            ControllerId::Primary => V5_ControllerId::kControllerMaster,
            ControllerId::Partner => V5_ControllerId::kControllerPartner,
        }
    }
}

impl Controller {
    /// The update rate of the controller.
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(25);

    /// Create a new controller.
    ///
    /// # Safety
    ///
    /// Creating new `Controller`s is inherently unsafe due to the possibility of constructing
    /// more than one screen at once allowing multiple mutable references to the same
    /// hardware device. Prefer using [`Peripherals`](crate::peripherals::Peripherals) to register devices if possible.
    pub const unsafe fn new(id: ControllerId) -> Self {
        Self {
            id,
            screen: ControllerScreen { id },
        }
    }

    /// Gets the controller's connection type.
    pub fn connection(&self) -> ControllerConnection {
        unsafe { vexControllerConnectionStatusGet(self.id.into()) }.into()
    }

    /// Gets the controller's battery capacity.
    pub fn battery_capacity(&self) -> Result<i32, ControllerError> {
        validate_connection(self.id)?;

        Ok(unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::BatteryCapacity) })
    }

    /// Gets the controller's battery level.
    pub fn battery_level(&self) -> Result<i32, ControllerError> {
        validate_connection(self.id)?;

        Ok(unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::BatteryLevel) })
    }

    /// Gets the controller's flags.
    pub fn flags(&self) -> Result<i32, ControllerError> {
        validate_connection(self.id)?;

        Ok(unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Flags) })
    }

    /// Send a rumble pattern to the controller's vibration motor.
    ///
    /// This function takes a string consisting of the characters '.', '-', and ' ', where
    /// dots are short rumbles, dashes are long rumbles, and spaces are pauses. Maximum
    /// supported length is 8 characters.
    pub fn rumble(&mut self, pattern: &str) -> Result<(), ControllerError> {
        self.screen.set_text(pattern, 3, 0)
    }
}

/// Represents the state of a controller's connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerConnection {
    /// No controller is connected.
    Offline,

    /// Controller is tethered through a wired smart port connection.
    Tethered,

    /// Controller is wirelessly connected over a VEXNet radio
    VexNet,
}

impl From<V5_ControllerStatus> for ControllerConnection {
    fn from(value: V5_ControllerStatus) -> Self {
        match value {
            V5_ControllerStatus::kV5ControllerOffline => Self::Offline,
            V5_ControllerStatus::kV5ControllerTethered => Self::Tethered,
            V5_ControllerStatus::kV5ControllerVexnet => Self::VexNet,
            _ => unreachable!(),
        }
    }
}

impl From<ControllerConnection> for V5_ControllerStatus {
    fn from(value: ControllerConnection) -> Self {
        match value {
            ControllerConnection::Offline => Self::kV5ControllerOffline,
            ControllerConnection::Tethered => Self::kV5ControllerTethered,
            ControllerConnection::VexNet => Self::kV5ControllerVexnet,
        }
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with the controller.
pub enum ControllerError {
    /// The controller is not connected to the brain.
    Offline,
    /// CString::new encountered NUL (U+0000) byte in non-terminating position.
    NonTerminatingNul,
    /// Access to controller data is restricted by competition control.
    CompetitionControl,
    /// An invalid line number was given.
    InvalidLine,
}
