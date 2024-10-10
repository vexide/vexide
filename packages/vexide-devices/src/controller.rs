//! V5 Controller
//!
//! This module allows you to read from the buttons and joysticks on the controller and write to the controller's display.

use alloc::ffi::CString;
use core::{cell::RefCell, time::Duration};

use snafu::Snafu;
use vex_sdk::{
    vexControllerConnectionStatusGet, vexControllerGet, vexControllerTextSet, V5_ControllerId,
    V5_ControllerIndex, V5_ControllerStatus,
};
use vexide_core::{competition, competition::CompetitionMode};

/// Represents the state of a button on the controller.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonState {
    prev_is_pressed: bool,
    is_pressed: bool,
}

impl ButtonState {
    /// Returns true if the button state is [`Pressed`](ButtonState::Pressed).
    pub const fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns true if the button state is [`Released`](ButtonState::Released).
    pub const fn is_released(&self) -> bool {
        !self.is_pressed
    }

    /// Returns true if the button state was released in the previous call to [`Controller::state`], but is now pressed.
    pub const fn is_now_pressed(&self) -> bool {
        !self.prev_is_pressed && self.is_pressed
    }

    /// Returns true if the button state was pressed in the previous call to [`Controller::state`], but is now released.
    pub const fn is_now_released(&self) -> bool {
        self.prev_is_pressed && !self.is_pressed
    }
}

/// Stores how far the joystick is away from the center (at *(0, 0)*) from -1 to 1.
/// On the x axis left is negative, and right is positive.
/// On the y axis down is negative, and up is positive.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct JoystickState {
    x_raw: i8,
    y_raw: i8,
}

impl JoystickState {
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
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct ControllerState {
    /// Left Joystick
    pub left_stick: JoystickState,
    /// Right Joystick
    pub right_stick: JoystickState,

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

/// This type stores the "pressed" states of every controller button.
///
/// This exists to efficiently cache previous button states with `Controller::update`, since
/// each `ButtonState` needs to know about its previous state from the last `Controller::update`
/// call in order to allow for `ButtonState::is_now_pressed` and `ButtonState::is_now_released`.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
struct ButtonStates {
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    left_trigger_1: bool,
    left_trigger_2: bool,
    right_trigger_1: bool,
    right_trigger_2: bool,
}

fn validate_connection(id: ControllerId) -> Result<(), ControllerError> {
    if unsafe {
        vexControllerConnectionStatusGet(id.into()) == V5_ControllerStatus::kV5ControllerOffline
    } {
        return Err(ControllerError::Offline);
    }

    Ok(())
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

/// The basic type for a controller.
/// Used to get the state of its joysticks and controllers.
#[derive(Debug, Eq, PartialEq)]
pub struct Controller {
    id: ControllerId,
    prev_button_states: RefCell<ButtonStates>,

    /// Controller Screen
    pub screen: ControllerScreen,
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
            prev_button_states: RefCell::new(ButtonStates {
                a: false,
                b: false,
                x: false,
                y: false,
                up: false,
                down: false,
                left: false,
                right: false,
                left_trigger_1: false,
                left_trigger_2: false,
                right_trigger_1: false,
                right_trigger_2: false,
            }),
            screen: ControllerScreen { id },
        }
    }

    /// Returns the current state of all buttons and joysticks on the controller.
    ///
    /// # Note
    ///
    /// If the current competition mode is not driver control, this function will error.
    pub fn state(&self) -> Result<ControllerState, ControllerError> {
        if competition::mode() != CompetitionMode::Driver {
            return Err(ControllerError::CompetitionControl);
        }
        validate_connection(self.id)?;

        // Get all current button states
        let button_states = ButtonStates {
            a: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonA) } != 0,
            b: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonB) } != 0,
            x: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonX) } != 0,
            y: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonY) } != 0,
            up: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonUp) } != 0,
            down: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonDown) } != 0,
            left: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonLeft) } != 0,
            right: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonRight) }
                != 0,
            left_trigger_1: unsafe {
                vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonL1)
            } != 0,
            left_trigger_2: unsafe {
                vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonL2)
            } != 0,
            right_trigger_1: unsafe {
                vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonR1)
            } != 0,
            right_trigger_2: unsafe {
                vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonR2)
            } != 0,
        };

        // Swap the current button states with the previous states, getting the previous states in the process.
        let prev_button_states = self.prev_button_states.replace(button_states.clone());

        Ok(ControllerState {
            left_stick: JoystickState {
                x_raw: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Axis4) as _ },
                y_raw: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Axis3) as _ },
            },
            right_stick: JoystickState {
                x_raw: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Axis1) as _ },
                y_raw: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Axis2) as _ },
            },
            button_a: ButtonState {
                is_pressed: button_states.a,
                prev_is_pressed: prev_button_states.a,
            },
            button_b: ButtonState {
                is_pressed: button_states.b,
                prev_is_pressed: prev_button_states.b,
            },
            button_x: ButtonState {
                is_pressed: button_states.x,
                prev_is_pressed: prev_button_states.x,
            },
            button_y: ButtonState {
                is_pressed: button_states.y,
                prev_is_pressed: prev_button_states.y,
            },
            button_up: ButtonState {
                is_pressed: button_states.up,
                prev_is_pressed: prev_button_states.up,
            },
            button_down: ButtonState {
                is_pressed: button_states.down,
                prev_is_pressed: prev_button_states.down,
            },
            button_left: ButtonState {
                is_pressed: button_states.left,
                prev_is_pressed: prev_button_states.left,
            },
            button_right: ButtonState {
                is_pressed: button_states.right,
                prev_is_pressed: prev_button_states.right,
            },
            left_trigger_1: ButtonState {
                is_pressed: button_states.left_trigger_1,
                prev_is_pressed: prev_button_states.left_trigger_1,
            },
            left_trigger_2: ButtonState {
                is_pressed: button_states.left_trigger_2,
                prev_is_pressed: prev_button_states.left_trigger_2,
            },
            right_trigger_1: ButtonState {
                is_pressed: button_states.right_trigger_1,
                prev_is_pressed: prev_button_states.right_trigger_1,
            },
            right_trigger_2: ButtonState {
                is_pressed: button_states.right_trigger_2,
                prev_is_pressed: prev_button_states.right_trigger_2,
            },
        })
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
