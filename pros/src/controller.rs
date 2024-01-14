//! Read from the buttons and joysticks on the controller and write to the controller's display.
//!
//! Controllers are identified by their id, which is either 0 (master) or 1 (partner).
//! State of a controller can be checked by calling [`Controller::state`] which will return a struct with all of the buttons' and joysticks' state.

use alloc::{ffi::CString, vec::Vec};

use pros_sys::{controller_id_e_t, PROS_ERR};
use snafu::Snafu;

use crate::error::{bail_on, map_errno};

/// Holds whether or not the buttons on the controller are pressed or not
pub struct Buttons {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub left_trigger_1: bool,
    pub left_trigger_2: bool,
    pub right_trigger_1: bool,
    pub right_trigger_2: bool,
}

/// Stores how far the joystick is away from the center (at *(0, 0)*) from -1 to 1.
/// On the x axis left is negative, and right is positive.
/// On the y axis down is negative, and up is positive.
pub struct Joystick {
    pub x: f32,
    pub y: f32,
}

/// Stores both joysticks on the controller.
pub struct Joysticks {
    pub left: Joystick,
    pub right: Joystick,
}

/// Stores the current state of the controller; the joysticks and buttons.
pub struct ControllerState {
    pub joysticks: Joysticks,
    pub buttons: Buttons,
}

/// Represents one line on the controller console.
pub struct ControllerLine {
    controller: Controller,
    line: u8,
}

impl ControllerLine {
    pub const MAX_TEXT_LEN: usize = 14;
    pub const MAX_LINE_NUM: u8 = 2;
    pub fn try_print(&self, text: impl Into<Vec<u8>>) -> Result<(), ControllerError> {
        let text = text.into();
        let text_len = text.len();
        assert!(
            text_len > ControllerLine::MAX_TEXT_LEN,
            "Printed text is too long to fit on controller display ({text_len} > {})",
            Self::MAX_TEXT_LEN
        );
        let c_text = CString::new(text).expect("parameter `text` should not contain null bytes");
        bail_on!(PROS_ERR, unsafe {
            pros_sys::controller_set_text(self.controller.id(), self.line, 0, c_text.as_ptr())
        });
        Ok(())
    }
    pub fn print(&self, text: impl Into<Vec<u8>>) {
        self.try_print(text).unwrap();
    }
}

/// A digital channel (button) on the VEX controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ControllerButton {
    A = pros_sys::E_CONTROLLER_DIGITAL_A,
    B = pros_sys::E_CONTROLLER_DIGITAL_B,
    X = pros_sys::E_CONTROLLER_DIGITAL_X,
    Y = pros_sys::E_CONTROLLER_DIGITAL_Y,
    Up = pros_sys::E_CONTROLLER_DIGITAL_UP,
    Down = pros_sys::E_CONTROLLER_DIGITAL_DOWN,
    Left = pros_sys::E_CONTROLLER_DIGITAL_LEFT,
    Right = pros_sys::E_CONTROLLER_DIGITAL_RIGHT,
    LeftTrigger1 = pros_sys::E_CONTROLLER_DIGITAL_L1,
    LeftTrigger2 = pros_sys::E_CONTROLLER_DIGITAL_L2,
    RightTrigger1 = pros_sys::E_CONTROLLER_DIGITAL_R1,
    RightTrigger2 = pros_sys::E_CONTROLLER_DIGITAL_R2,
}

/// An analog channel (joystick axis) on the VEX controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum JoystickAxis {
    LeftX = pros_sys::E_CONTROLLER_ANALOG_LEFT_X,
    LeftY = pros_sys::E_CONTROLLER_ANALOG_LEFT_Y,
    RightX = pros_sys::E_CONTROLLER_ANALOG_RIGHT_X,
    RightY = pros_sys::E_CONTROLLER_ANALOG_RIGHT_Y,
}

/// The basic type for a controller.
/// Used to get the state of its joysticks and controllers.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum Controller {
    Master = pros_sys::E_CONTROLLER_MASTER,
    Partner = pros_sys::E_CONTROLLER_PARTNER,
}

impl Controller {
    fn id(&self) -> controller_id_e_t {
        *self as controller_id_e_t
    }

    pub fn line(&self, line_num: u8) -> ControllerLine {
        assert!(
            line_num > ControllerLine::MAX_LINE_NUM,
            "Line number is too large for controller display ({line_num} > {})",
            ControllerLine::MAX_LINE_NUM
        );

        ControllerLine {
            controller: *self,
            line: line_num,
        }
    }

    /// Gets the current state of the controller in its entirety.
    pub fn state(&self) -> ControllerState {
        ControllerState {
            joysticks: unsafe {
                Joysticks {
                    left: Joystick {
                        x: pros_sys::controller_get_analog(
                            self.id(),
                            pros_sys::E_CONTROLLER_ANALOG_LEFT_X,
                        ) as f32
                            / 127.0,
                        y: pros_sys::controller_get_analog(
                            self.id(),
                            pros_sys::E_CONTROLLER_ANALOG_LEFT_Y,
                        ) as f32
                            / 127.0,
                    },
                    right: Joystick {
                        x: pros_sys::controller_get_analog(
                            self.id(),
                            pros_sys::E_CONTROLLER_ANALOG_RIGHT_X,
                        ) as f32
                            / 127.0,
                        y: pros_sys::controller_get_analog(
                            self.id(),
                            pros_sys::E_CONTROLLER_ANALOG_RIGHT_Y,
                        ) as f32
                            / 127.0,
                    },
                }
            },
            buttons: unsafe {
                Buttons {
                    a: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_A,
                    ) == 1,
                    b: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_B,
                    ) == 1,
                    x: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_X,
                    ) == 1,
                    y: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_Y,
                    ) == 1,
                    up: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_UP,
                    ) == 1,
                    down: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_DOWN,
                    ) == 1,
                    left: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_LEFT,
                    ) == 1,
                    right: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_RIGHT,
                    ) == 1,
                    left_trigger_1: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_L1,
                    ) == 1,
                    left_trigger_2: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_L2,
                    ) == 1,
                    right_trigger_1: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_R1,
                    ) == 1,
                    right_trigger_2: pros_sys::controller_get_digital(
                        self.id(),
                        pros_sys::E_CONTROLLER_DIGITAL_R2,
                    ) == 1,
                }
            },
        }
    }

    /// Gets the state of a specific button on the controller.
    pub fn button(&self, button: ControllerButton) -> bool {
        unsafe { pros_sys::controller_get_digital(self.id(), button as u32) == 1 }
    }

    /// Gets the state of a specific joystick axis on the controller.
    pub fn joystick_axis(&self, axis: JoystickAxis) -> f32 {
        unsafe { pros_sys::controller_get_analog(self.id(), axis as u32) as f32 / 127.0 }
    }
}

#[derive(Debug, Snafu)]
pub enum ControllerError {
    #[snafu(display("Another resource is already using the controller"))]
    ConcurrentAccess,
}

map_errno! {
    ControllerError {
        EACCES => Self::ConcurrentAccess,
    }
}
