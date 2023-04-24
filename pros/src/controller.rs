#[repr(i32)]
pub enum ControllerId {
    Master = 0,
    Partner = 1,
}

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

/// Stores how far the joystick is away from the center (at *(0, 0)*) from -127 to 127.
/// On the x axis left is negative, and right is positive.
/// On the y axis down is negative, and up is positive.
pub struct Joystick {
    pub x: i8,
    pub y: i8,
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

/// The basic type for a controller.
/// Used to get the state of its joysticks and controllers.
pub struct Controller {
    id: u32,
}

impl Controller {
    /// Creates a new controller using an id.
    /// Use [`MASTER_CONTROLLER_ID`] if you are unsure what id means.
    pub fn new(id: ControllerId) -> Self {
        Self { id: id as _ }
    }

    pub fn state(&self) -> ControllerState {
        ControllerState {
            joysticks: unsafe {
                Joysticks {
                    left: Joystick {
                        x: pros_sys::controller_get_analog(
                            self.id,
                            pros_sys::controller_analog_e_t_E_CONTROLLER_ANALOG_LEFT_X,
                        ) as i8,
                        y: pros_sys::controller_get_analog(
                            self.id,
                            pros_sys::controller_analog_e_t_E_CONTROLLER_ANALOG_LEFT_Y,
                        ) as i8,
                    },
                    right: Joystick {
                        x: pros_sys::controller_get_analog(
                            self.id,
                            pros_sys::controller_analog_e_t_E_CONTROLLER_ANALOG_RIGHT_X,
                        ) as i8,
                        y: pros_sys::controller_get_analog(
                            self.id,
                            pros_sys::controller_analog_e_t_E_CONTROLLER_ANALOG_RIGHT_Y,
                        ) as i8,
                    },
                }
            },
            buttons: unsafe {
                Buttons {
                    a: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_A,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    b: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_B,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    x: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_X,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    y: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_Y,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    up: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_UP,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    down: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_DOWN,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    left: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_LEFT,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    right: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_RIGHT,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    left_trigger_1: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_L1,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    left_trigger_2: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_L2,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    right_trigger_1: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_R1,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                    right_trigger_2: if pros_sys::controller_get_digital(
                        self.id,
                        pros_sys::controller_digital_e_t_E_CONTROLLER_DIGITAL_R2,
                    ) == 1
                    {
                        true
                    } else {
                        false
                    },
                }
            },
        }
    }
}
