//! Read from the buttons and joysticks on the controller and write to the controller's display.
//!
//! Controllers are identified by their id, which is either 0 (master) or 1 (partner).
//! State of a controller can be checked by calling [`Controller::state`] which will return a struct with all of the buttons' and joysticks' state.
use alloc::ffi::CString;

use snafu::Snafu;
use vex_sdk::{vexControllerConnectionStatusGet, vexControllerGet, vexControllerTextSet, V5_ControllerId, V5_ControllerIndex};

use crate::{
    adi::digital::LogicLevel,
    competition::{self, CompetitionMode},
};

fn controller_connected(id: ControllerId) -> bool {
    unsafe { vexControllerConnectionStatusGet(id.into()) as u32 != 0 }
}

/// Digital Controller Button
#[derive(Debug, Eq, PartialEq)]
pub struct Button {
    id: ControllerId,
    channel: V5_ControllerIndex,
    last_level: LogicLevel,
}

impl Button {
    fn validate(&self) -> Result<(), ControllerError> {
        if !controller_connected(self.id) {
            return Err(ControllerError::NotConnected);
        }

        Ok(())
    }

    /// Gets the current logic level of a digital input pin.
    pub fn level(&self) -> Result<LogicLevel, ControllerError> {
        self.validate()?;
        if competition::mode() != CompetitionMode::Driver {
            return Err(ControllerError::CompetitionControl);
        }

        let value =
            unsafe { vexControllerGet(self.id.into(), self.channel.try_into().unwrap()) != 0 };

        let level = match value {
            true => LogicLevel::High,
            false => LogicLevel::Low,
        };
        self.last_level = level;

        Ok(level)
    }

    /// Returrns `true` if the button is currently being pressed.
    ///
    /// This is equivalent shorthand to calling `Self::level().is_high()`.
    pub fn is_pressed(&self) -> Result<bool, ControllerError> {
        self.validate()?;
        Ok(self.level()?.is_high())
    }

    /// Returns `true` if the button has been pressed again since the last time this
    /// function was called.
    ///
    /// # Thread Safety
    ///
    /// This function is not thread-safe.
    ///
    /// Multiple tasks polling a single button may return different results under the
    /// same circumstances, so only one task should call this function for any given
    /// switch. E.g., Task A calls this function for buttons 1 and 2. Task B may call
    /// this function for button 3, but should not for buttons 1 or 2. A typical
    /// use-case for this function is to call inside opcontrol to detect new button
    /// presses, and not in any other tasks.
    pub fn was_pressed(&mut self) -> Result<bool, ControllerError> {
        self.validate()?;
        if competition::mode() != CompetitionMode::Driver {
            return Err(ControllerError::CompetitionControl);
        }
        let current_level = self.level()?;
        Ok(self.last_level.is_low() && current_level.is_high())
    }
}

/// Stores how far the joystick is away from the center (at *(0, 0)*) from -1 to 1.
/// On the x axis left is negative, and right is positive.
/// On the y axis down is negative, and up is positive.
#[derive(Debug, Eq, PartialEq)]
pub struct Joystick {
    id: ControllerId,
    x_channel: V5_ControllerIndex,
    y_channel: V5_ControllerIndex,
}

impl Joystick {
    fn validate(&self) -> Result<(), ControllerError> {
        if !controller_connected(self.id) {
            return Err(ControllerError::NotConnected);
        }

        Ok(())
    }

    /// Gets the value of the joystick position on its x-axis from [-1, 1].
    pub fn x(&self) -> Result<f32, ControllerError> {
        self.validate()?;
        Ok(self.x_raw()? as f32 / 127.0)
    }

    /// Gets the value of the joystick position on its y-axis from [-1, 1].
    pub fn y(&self) -> Result<f32, ControllerError> {
        self.validate()?;
        Ok(self.y_raw()? as f32 / 127.0)
    }

    /// Gets the raw value of the joystick position on its x-axis from [-128, 127].
    pub fn x_raw(&self) -> Result<i8, ControllerError> {
        self.validate()?;
        if competition::mode() != CompetitionMode::Driver {
            return Err(ControllerError::CompetitionControl);
        }

        Ok(unsafe { vexControllerGet(self.id.into(), self.x_channel) } as _)
    }

    /// Gets the raw value of the joystick position on its x-axis from [-128, 127].
    pub fn y_raw(&self) -> Result<i8, ControllerError> {
        self.validate()?;
        if competition::mode() != CompetitionMode::Driver {
            return Err(ControllerError::CompetitionControl);
        }

        Ok(unsafe { vexControllerGet(self.id.into(), self.y_channel) } as _)
    }
}

/// The basic type for a controller.
/// Used to get the state of its joysticks and controllers.
#[derive(Debug, Eq, PartialEq)]
pub struct Controller {
    id: ControllerId,

    /// Controller Screen
    pub screen: ControllerScreen,

    /// Left Joystick
    pub left_stick: Joystick,
    /// Right Joystick
    pub right_stick: Joystick,

    /// Button A
    pub button_a: Button,
    /// Button B
    pub button_b: Button,
    /// Button X
    pub button_x: Button,
    /// Button Y
    pub button_y: Button,

    /// Button Up
    pub button_up: Button,
    /// Button Down
    pub button_down: Button,
    /// Button Left
    pub button_left: Button,
    /// Button Right
    pub button_right: Button,

    /// Top Left Trigger
    pub left_trigger_1: Button,
    /// Bottom Left Trigger
    pub left_trigger_2: Button,
    /// Top Right Trigger
    pub right_trigger_1: Button,
    /// Bottom Right Trigger
    pub right_trigger_2: Button,
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

    fn validate(&self) -> Result<(), ControllerError> {
        if !controller_connected(self.id) {
            return Err(ControllerError::NotConnected);
        }

        Ok(())
    }

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
        self.validate()?;
        if col >= Self::MAX_LINE_LENGTH as u8 {
            return Err(ControllerError::InvalidLine);
        }

        let id: V5_ControllerId = self.id.into();
        let text = CString::new(text).map_err(|_| ControllerError::NonTerminatingNull)?.into_raw();

        unsafe { vexControllerTextSet(id as u32, (line + 1) as _, (col + 1) as _, text as *const _); }

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
    fn validate(&self) -> Result<(), ControllerError> {
        if !controller_connected(self.id) {
            return Err(ControllerError::NotConnected);
        }

        Ok(())
    }

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
            left_stick: Joystick {
                id,
                x_channel: V5_ControllerIndex::Axis1,
                y_channel: V5_ControllerIndex::Axis2,
            },
            right_stick: Joystick {
                id,
                x_channel: V5_ControllerIndex::Axis3,
                y_channel: V5_ControllerIndex::Axis4,
            },
            button_a: Button {
                id,
                channel: V5_ControllerIndex::ButtonA,
                last_level: LogicLevel::Low,
            },
            button_b: Button {
                id,
                channel: V5_ControllerIndex::ButtonB,
                last_level: LogicLevel::Low,
            },
            button_x: Button {
                id,
                channel: V5_ControllerIndex::ButtonX,
                last_level: LogicLevel::Low,
            },
            button_y: Button {
                id,
                channel: V5_ControllerIndex::ButtonY,
                last_level: LogicLevel::Low,
            },
            button_up: Button {
                id,
                channel: V5_ControllerIndex::ButtonUp,
                last_level: LogicLevel::Low,
            },
            button_down: Button {
                id,
                channel: V5_ControllerIndex::ButtonDown,
                last_level: LogicLevel::Low,
            },
            button_left: Button {
                id,
                channel: V5_ControllerIndex::ButtonLeft,
                last_level: LogicLevel::Low,
            },
            button_right: Button {
                id,
                channel: V5_ControllerIndex::ButtonRight,
                last_level: LogicLevel::Low,
            },
            left_trigger_1: Button {
                id,
                channel: V5_ControllerIndex::ButtonL1,
                last_level: LogicLevel::Low,
            },
            left_trigger_2: Button {
                id,
                channel: V5_ControllerIndex::ButtonL2,
                last_level: LogicLevel::Low,
            },
            right_trigger_1: Button {
                id,
                channel: V5_ControllerIndex::ButtonR1,
                last_level: LogicLevel::Low,
            },
            right_trigger_2: Button {
                id,
                channel: V5_ControllerIndex::ButtonR2,
                last_level: LogicLevel::Low,
            },
        }
    }

    /// Returns `true` if the controller is connected to the brain.
    pub fn is_connected(&self) -> bool {
        controller_connected(self.id)
    }

    /// Gets the controller's battery capacity.
    pub fn battery_capacity(&self) -> Result<i32, ControllerError> {
        self.validate()?;

        Ok(unsafe {
            vexControllerGet(self.id.into(), V5_ControllerIndex::BatteryCapacity)
        })
    }

    /// Gets the controller's battery level.
    pub fn battery_level(&self) -> Result<i32, ControllerError> {
        self.validate()?;

        Ok(unsafe {
            vexControllerGet(self.id.into(), V5_ControllerIndex::BatteryLevel)
        })
    }

    /// Send a rumble pattern to the controller's vibration motor.
    ///
    /// This function takes a string consisting of the characters '.', '-', and ' ', where
    /// dots are short rumbles, dashes are long rumbles, and spaces are pauses. Maximum
    /// supported length is 8 characters.
    pub fn rumble(&mut self, pattern: &str) -> Result<(), ControllerError> {
        self.validate()?;

        self.screen.set_text(pattern, 3, 0);

        Ok(())
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when interacting with the controller.
pub enum ControllerError {
    /// The controller is not connected to the brain.
    NotConnected,
    /// CString::new encountered NULL (U+0000) byte in non-terminating position.
    NonTerminatingNull,
    /// Access to controller data is restricted by competition control.
    CompetitionControl,
    /// An invalid line number was given.
    InvalidLine,
}
