//! V5 Controller
//!
//! This module allows you to read from the buttons and joysticks on the controller and write to the controller's display.

use alloc::ffi::{CString, NulError};
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
    /// Returns `true` if this button is currently being pressed.
    #[must_use]
    pub const fn is_pressed(&self) -> bool {
        self.is_pressed
    }

    /// Returns `true` if this button is currently released (not being pressed).
    #[must_use]
    pub const fn is_released(&self) -> bool {
        !self.is_pressed
    }

    /// Returns `true` if the button state was released in the previous call to [`Controller::state`], but is now pressed.
    #[must_use]
    pub const fn is_now_pressed(&self) -> bool {
        !self.prev_is_pressed && self.is_pressed
    }

    /// Returns `true` if the button state was pressed in the previous call to [`Controller::state`], but is now released.
    #[must_use]
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
    /// Returns the value of the joystick position on its x-axis from [-1, 1].
    #[must_use]
    pub fn x(&self) -> f64 {
        f64::from(self.x_raw) / 127.0
    }
    /// Returns the value of the joystick position on its y-axis from [-1, 1].
    #[must_use]
    pub fn y(&self) -> f64 {
        f64::from(self.y_raw) / 127.0
    }

    /// The raw value of the joystick position on its x-axis from [-127, 127].
    #[must_use]
    pub const fn x_raw(&self) -> i8 {
        self.x_raw
    }
    /// The raw value of the joystick position on its x-axis from [-127, 127].
    #[must_use]
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

    /// Top Left Bumpeer
    pub button_l1: ButtonState,
    /// Bottom Left Bumper
    pub button_l2: ButtonState,
    /// Top Right Bumper
    pub button_r1: ButtonState,
    /// Bottom Right Bumper
    pub button_r2: ButtonState,
}

/// This type stores the "pressed" states of every controller button.
///
/// This exists to efficiently cache previous button states with `Controller::update`, since
/// each `ButtonState` needs to know about its previous state from the last `Controller::update`
/// call in order to allow for `ButtonState::is_now_pressed` and `ButtonState::is_now_released`.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "not being used as state machine"
)]
struct ButtonStates {
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    l1: bool,
    l2: bool,
    r1: bool,
    r2: bool,
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
    pub const MAX_COLUMNS: usize = 19;

    /// Number of available text lines on the controller before clearing the screen.
    pub const MAX_LINES: usize = 3;

    /// Clears the contents of a specific text line.
    ///
    /// <section class="warning">
    ///
    /// Controller text setting is a slow process, so updates faster than 10ms when on a
    /// wired connection or 50ms over VEXnet will not be applied to the controller.
    ///
    /// </section>
    ///
    /// # Errors
    ///
    /// - A [`ControllerError::Offline`] error is returned if the controller is
    ///   not connected.
    /// - A [`ControllerError::WriteBusy`] error is returned if a screen write
    ///   occurred too quickly after the previous write attempt.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut controller = peripherals.controller;
    ///
    ///     // Write to line 0
    ///     _ = controller.set_text("Hello, world!", 0, 0);
    ///
    ///     sleep(Duration::from_millis(500)).await;
    ///
    ///     // Clear line 0
    ///     _ = controller.clear_line(0);
    /// }
    /// ```
    pub fn clear_line(&mut self, line: u8) -> Result<(), ControllerError> {
        //TODO: Older versions of VexOS clear the controller by setting the line to "                   ".
        //TODO: We should check the version and change behavior based on it.
        self.set_text("", line, 0)?;

        Ok(())
    }

    /// Clears the whole screen, including the default widget displayed by the controller if
    /// it hasn't already been cleared.
    ///
    /// <section class="warning">
    ///
    /// Controller text setting is a slow process, so updates faster than 10ms when on a
    /// wired connection or 50ms over VEXnet will not be applied to the controller.
    ///
    /// </section>
    ///
    /// # Errors
    ///
    /// - A [`ControllerError::Offline`] error is returned if the controller is
    ///   not connected.
    /// - A [`ControllerError::WriteBusy`] error is returned if a screen write
    ///   occurred too quickly after the previous write attempt.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut controller = peripherals.controller;
    ///
    ///     // Remove the default widget on the controller screen that displays match time.
    ///     _ = controller.clear_screen();
    /// }
    /// ```
    pub fn clear_screen(&mut self) -> Result<(), ControllerError> {
        validate_connection(self.id)?;

        let id: V5_ControllerId = self.id.into();

        if unsafe { vexControllerTextSet(u32::from(id.0), 0, 0, c"".as_ptr().cast()) } != 1 {
            return Err(ControllerError::WriteBusy);
        }

        Ok(())
    }

    /// Set the text contents at a specific row/column offset.
    ///
    /// <section class="warning">
    ///
    /// Controller text setting is a slow process, so updates faster than 10ms when on a
    /// wired connection or 50ms over VEXnet will not be applied to the controller.
    ///
    /// </section>
    ///
    /// # Errors
    ///
    /// - A [`ControllerError::InvalidLine`] error is returned if `col` is
    ///   greater than or equal to [`Self::MAX_LINES`].
    /// - A [`ControllerError::InvalidColumn`] error is returned if `col` is
    ///   greater than or equal to [`Self::MAX_COLUMNS`].
    /// - A [`ControllerError::Nul`] error if a NUL (0x00) character was
    ///   found anywhere in the specified text.
    /// - A [`ControllerError::Offline`] error is returned if the controller is
    ///   not connected.
    /// - A [`ControllerError::WriteBusy`] error is returned if a screen write
    ///   occurred too quickly after the previous write attempt.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut controller = peripherals.controller;
    ///
    ///     _ = controller.set_text("Hello, world!", 0, 0);
    /// }
    /// ```
    pub fn set_text(&mut self, text: &str, line: u8, col: u8) -> Result<(), ControllerError> {
        validate_connection(self.id)?;

        if line > Self::MAX_LINES as u8 {
            return Err(ControllerError::InvalidLine);
        }

        if col > Self::MAX_COLUMNS as u8 {
            return Err(ControllerError::InvalidColumn);
        }

        let id: V5_ControllerId = self.id.into();
        let text = CString::new(text)?;

        if unsafe {
            vexControllerTextSet(
                u32::from(id.0),
                u32::from(line + 1),
                u32::from(col + 1),
                text.as_ptr().cast(),
            )
        } != 1
        {
            return Err(ControllerError::WriteBusy);
        }

        Ok(())
    }
}

/// Represents an identifier for one of the two possible controllers
/// connected to the V5 Brain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerId {
    /// Primary ("Master") Controller
    /// This is the controller that is connected to the Brain.
    Primary,

    /// Partner Controller
    /// This is the controller that is connected to the [primary](ControllerId::Primary) controller.
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

    /// Controller is tethered through a wired Smart Port connection.
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
    #[must_use]
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
                l1: false,
                l2: false,
                r1: false,
                r2: false,
            }),
            screen: ControllerScreen { id },
        }
    }

    /// Returns the [identifier](ControllerId) of this controller.
    ///
    /// # Examples
    ///
    /// Perform a different action based on the controller ID.
    /// ```
    /// use vexide::prelude::*;
    ///
    /// fn print_a_pressed(controller: &Controller) {
    ///     let state = controller.state().unwrap_or_default();
    ///     if state.button_a.is_pressed() {
    ///         match controller.id() {
    ///             ControllerId::Primary => println!("Primary Controller A Pressed"),
    ///             ControllerId::Partner => println!("Partner Controller A Pressed"),
    ///         }
    ///     }
    /// }
    /// ```
    #[must_use]
    pub const fn id(&self) -> ControllerId {
        self.id
    }

    /// Returns the current state of all buttons and joysticks on the controller.
    ///
    /// # Note
    ///
    /// If the current competition mode is not driver control, this function will error.
    ///
    /// # Errors
    ///
    /// - A [`ControllerError::CompetitionControl`] error is returned if access to
    ///   the controller data is being restricted by competition control.
    /// - A [`ControllerError::Offline`] error is returned if the controller is
    ///   not connected.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let controller = peripherals.primary_controller;
    ///
    ///     loop {
    ///         let state = controller.state().unwrap_or_default();
    ///         println("Left Stick X: {}", state.left_stick.x());
    ///         if state.button_a.is_now_pressed() {
    ///             println!("Button A was just pressed!");
    ///         }
    ///         if state.button_x.is_pressed() {
    ///             println!("Button X is pressed!");
    ///         }
    ///         if state.button_b.is_released() {
    ///             println!("Button B is released!");
    ///         }
    ///         sleep(Controller::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
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
            l1: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonL1) } != 0,
            l2: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonL2) } != 0,
            r1: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonR1) } != 0,
            r2: unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::ButtonR2) } != 0,
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
            button_l1: ButtonState {
                is_pressed: button_states.l1,
                prev_is_pressed: prev_button_states.l1,
            },
            button_l2: ButtonState {
                is_pressed: button_states.l2,
                prev_is_pressed: prev_button_states.l2,
            },
            button_r1: ButtonState {
                is_pressed: button_states.r1,
                prev_is_pressed: prev_button_states.r1,
            },
            button_r2: ButtonState {
                is_pressed: button_states.r2,
                prev_is_pressed: prev_button_states.r2,
            },
        })
    }

    /// Returns the controller's connection type.
    ///
    /// # Examples
    ///
    /// Print less information over a slow and unreliable VEXnet connection:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let controller = peripherals.primary_controller;
    ///     if controller.connection() != ControllerConnection::VexNet {
    ///         println!("A big info dump");
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn connection(&self) -> ControllerConnection {
        unsafe { vexControllerConnectionStatusGet(self.id.into()) }.into()
    }

    /// Returns the controller's battery capacity.
    ///
    /// # Errors
    ///
    /// - A [`ControllerError::Offline`] error is returned if the controller is
    ///   not connected.
    ///
    /// # Examples
    ///
    /// Print the controller's battery capacity:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let controller = peripherals.primary_controller;
    ///     println!("Controller battery capacity: {}", controller.battery_capacity().unwrap_or(0));
    /// }
    /// ```
    pub fn battery_capacity(&self) -> Result<i32, ControllerError> {
        validate_connection(self.id)?;

        Ok(unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::BatteryCapacity) })
    }

    /// Returns the controller's battery level.
    ///
    /// # Errors
    ///
    /// - A [`ControllerError::Offline`] error is returned if the controller is
    ///   not connected.
    ///
    /// # Examples
    ///
    /// Print a warning if the controller battery is low:
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let controller = peripherals.primary_controller;
    ///     loop {
    ///         // If the controller isn't connected, it may as well be dead.
    ///         let battery_level = controller.battery_level().unwrap_or(0);
    ///         if battery_level < 10 {
    ///             println!("WARNING: Controller battery is low!");
    ///         }
    ///         sleep(Controller::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn battery_level(&self) -> Result<i32, ControllerError> {
        validate_connection(self.id)?;

        Ok(unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::BatteryLevel) })
    }

    /// Returns the controller's flags.
    ///
    /// # Errors
    ///
    /// - A [`ControllerError::Offline`] error is returned if the controller is
    ///   not connected.
    pub fn flags(&self) -> Result<i32, ControllerError> {
        validate_connection(self.id)?;

        Ok(unsafe { vexControllerGet(self.id.into(), V5_ControllerIndex::Flags) })
    }

    /// Send a rumble pattern to the controller's vibration motor.
    ///
    /// This function takes a string consisting of the characters '.', '-', and ' ', where
    /// dots are short rumbles, dashes are long rumbles, and spaces are pauses. Maximum
    /// supported length is 8 characters.
    ///
    /// # Errors
    ///
    /// - A [`ControllerError::Nul`] error if a NUL (0x00) character was
    ///   found anywhere in the specified text.
    /// - A [`ControllerError::Offline`] error is returned if the controller is
    ///   not connected.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut controller = peripherals.primary_controller;
    ///     let _ = controller.rumble(". -. -.");
    /// }
    /// ```
    pub fn rumble(&mut self, pattern: &str) -> Result<(), ControllerError> {
        self.screen.set_text(pattern, 3, 0)
    }
}

/// Errors that can occur when interacting with the controller.
#[derive(Debug, Snafu)]
pub enum ControllerError {
    /// The controller is not connected to the Brain.
    Offline,

    /// A NUL (0x00) character was found in a string that may not contain NUL characters.
    #[snafu(transparent)]
    Nul {
        /// The source of the error.
        source: NulError,
    },

    /// Access to controller data is restricted by competition control.
    CompetitionControl,

    /// An invalid line number was given.
    InvalidLine,

    /// An invalid line number was given.
    InvalidColumn,

    /// Attempted to write a buffer to the controller's screen before the previous buffer was sent.
    WriteBusy,
}
