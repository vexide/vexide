use core::ffi::*;

pub const NUM_V5_PORTS: usize = 21;
// v5 comp
pub const COMPETITION_AUTONOMOUS: u8 = 1 << 0;
pub const COMPETITION_DISABLED: u8 = 1 << 1;
pub const COMPETITION_CONNECTED: u8 = 1 << 2;
extern "C" {
    /// Get the current status of the competition control.
    /// Returns The competition control status as a mask of bits with COMPETITION_{ENABLED,AUTONOMOUS,CONNECTED}.
    pub fn competition_get_status() -> u8;
    /// Returns true if the robot is in autonomous mode, false otherwise.
    pub fn competition_is_autonomous() -> bool;
    /// Returns true if the V5 Brain is connected to competition control, false otherwise.
    pub fn competition_is_connected() -> bool;
    /// Returns true if the V5 Brain is disabled, false otherwise.
    pub fn competition_is_disabled() -> bool;
}
// controller
pub const E_CONTROLLER_MASTER: c_uint = 0;
pub const E_CONTROLLER_PARTNER: c_uint = 1;
pub type controller_id_e_t = c_uint;
pub const E_CONTROLLER_ANALOG_LEFT_X: c_uint = 0;
pub const E_CONTROLLER_ANALOG_LEFT_Y: c_uint = 1;
pub const E_CONTROLLER_ANALOG_RIGHT_X: c_uint = 2;
pub const E_CONTROLLER_ANALOG_RIGHT_Y: c_uint = 3;
pub type controller_analog_e_t = c_uint;
pub const E_CONTROLLER_DIGITAL_L1: c_uint = 6;
pub const E_CONTROLLER_DIGITAL_L2: c_uint = 7;
pub const E_CONTROLLER_DIGITAL_R1: c_uint = 8;
pub const E_CONTROLLER_DIGITAL_R2: c_uint = 9;
pub const E_CONTROLLER_DIGITAL_UP: c_uint = 10;
pub const E_CONTROLLER_DIGITAL_DOWN: c_uint = 11;
pub const E_CONTROLLER_DIGITAL_LEFT: c_uint = 12;
pub const E_CONTROLLER_DIGITAL_RIGHT: c_uint = 13;
pub const E_CONTROLLER_DIGITAL_X: c_uint = 14;
pub const E_CONTROLLER_DIGITAL_B: c_uint = 15;
pub const E_CONTROLLER_DIGITAL_Y: c_uint = 16;
pub const E_CONTROLLER_DIGITAL_A: c_uint = 17;
pub type controller_digital_e_t = c_uint;

extern "C" {
    /**
    Checks if the controller is connected.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of CONTROLLER_MASTER or CONTROLLER_PARTNER

    \return 1 if the controller is connected, 0 otherwise
    */
    pub fn controller_is_connected(id: controller_id_e_t) -> i32;
    /**
    Gets the value of an analog channel (joystick) on a controller.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of CONTROLLER_MASTER or CONTROLLER_PARTNER
    \param channel
           The analog channel to get.
           Must be one of ANALOG_LEFT_X, ANALOG_LEFT_Y, ANALOG_RIGHT_X,
           ANALOG_RIGHT_Y

    \return The current reading of the analog channel: [-127, 127].
    If the controller was not connected, then 0 is returned
    */
    pub fn controller_get_analog(id: controller_id_e_t, channel: controller_analog_e_t) -> i32;
    /**
    Gets the battery capacity of the given controller.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER

    \return The controller's battery capacity
    */
    pub fn controller_get_battery_capacity(id: controller_id_e_t) -> i32;
    /**
    Gets the battery level of the given controller.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER

    \return The controller's battery level
    */
    pub fn controller_get_battery_level(id: controller_id_e_t) -> i32;
    /**
    Checks if a digital channel (button) on the controller is currently pressed.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of CONTROLLER_MASTER or CONTROLLER_PARTNER
    \param button
           The button to read.
           Must be one of DIGITAL_{RIGHT,DOWN,LEFT,UP,A,B,Y,X,R1,R2,L1,L2}

    \return 1 if the button on the controller is pressed.
    If the controller was not connected, then 0 is returned
    */
    pub fn controller_get_digital(id: controller_id_e_t, button: controller_digital_e_t) -> i32;
    /**
    Returns a rising-edge case for a controller button press.

    This function is not thread-safe.
    Multiple tasks polling a single button may return different results under the
    same circumstances, so only one task should call this function for any given
    button. E.g., Task A calls this function for buttons 1 and 2. Task B may call
    this function for button 3, but should not for buttons 1 or 2. A typical
    use-case for this function is to call inside opcontrol to detect new button
    presses, and not in any other tasks.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of CONTROLLER_MASTER or CONTROLLER_PARTNER
    \param button
                  The button to read. Must be one of
           DIGITAL_{RIGHT,DOWN,LEFT,UP,A,B,Y,X,R1,R2,L1,L2}

    \return 1 if the button on the controller is pressed and had not been pressed
    the last time this function was called, 0 otherwise.
    */
    pub fn controller_get_digital_new_press(
        id: controller_id_e_t,
        button: controller_digital_e_t,
    ) -> i32;
    /**
    Sets text to the controller LCD screen.

    \note Controller text setting is currently in beta, so continuous, fast
    updates will not work well.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of CONTROLLER_MASTER or CONTROLLER_PARTNER
    \param line
           The line number at which the text will be displayed [0-2]
    \param col
           The column number at which the text will be displayed [0-14]
    \param fmt
           The format string to print to the controller
    \param ...
           The argument list for the format string

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn controller_print(
        id: controller_id_e_t,
        line: u8,
        col: u8,
        fmt: *const c_char,
        ...
    ) -> i32;
    /**
    Sets text to the controller LCD screen.

    \note Controller text setting is currently in beta, so continuous, fast
    updates will not work well.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of CONTROLLER_MASTER or CONTROLLER_PARTNER
    \param line
           The line number at which the text will be displayed [0-2]
    \param col
           The column number at which the text will be displayed [0-14]
    \param str
           The pre-formatted string to print to the controller

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn controller_set_text(
        id: controller_id_e_t,
        line: u8,
        col: u8,
        string: *const c_char,
    ) -> i32;
    /**
    Clears an individual line of the controller screen.

    \note Controller text setting is currently in beta, so continuous, fast
    updates will not work well.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of CONTROLLER_MASTER or CONTROLLER_PARTNER
    \param line
           The line number to clear [0-2]

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn controller_clear_line(id: controller_id_e_t, line: u8) -> i32;
    /**
    Clears all of the lines on the controller screen.

    \note Controller text setting is currently in beta, so continuous, fast
    updates will not work well. On vexOS version 1.0.0 this function will block
    for 110ms.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
           The ID of the controller (e.g. the master or partner controller).
           Must be one of CONTROLLER_MASTER or CONTROLLER_PARTNER

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn controller_clear(id: controller_id_e_t) -> i32;
    /**
    Rumble the controller.

    \note Controller rumble activation is currently in beta, so continuous, fast
    updates will not work well.

    This function uses the following values of errno when an error state is
    reached:
    EINVAL - A value other than E_CONTROLLER_MASTER or E_CONTROLLER_PARTNER is
    given.
    EACCES - Another resource is currently trying to access the controller port.

    \param id
                   The ID of the controller (e.g. the master or partner controller).
                   Must be one of CONTROLLER_MASTER or CONTROLLER_PARTNER
    \param rumble_pattern
                   A string consisting of the characters '.', '-', and ' ', where dots
                   are short rumbles, dashes are long rumbles, and spaces are pauses.
                   Maximum supported length is 8 characters.

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn controller_rumble(id: controller_id_e_t, rumble: *const c_char) -> i32;
}

// date and time
extern "C" {
    pub static mut baked_date: *const c_char;
    pub static mut baked_time: *const c_char;
}
#[repr(C)]
pub struct date_s_t {
    /// Year - 1980
    pub year: u16,
    pub day: u8,
    /// 1 = January
    pub month: u8,
}
#[repr(C)]
pub struct time_s_t {
    pub hour: u8,
    pub minute: u8,
    pub sec: u8,
    /// hundredths of a second
    pub sec_hund: u8,
}

// robot

extern "C" {
    /**
    Gets the current voltage of the battery, as reported by VEXos.

    This function uses the following values of errno when an error state is
    reached:
    EACCES - Another resource is currently trying to access the battery port.

    \return The current voltage of the battery
    */
    pub fn battery_get_voltage() -> i32;
    /**
    Gets the current current of the battery, as reported by VEXos.

    This function uses the following values of errno when an error state is
    reached:
    EACCES - Another resource is currently trying to access the battery port.

    \return The current current of the battery
    */
    pub fn battery_get_current() -> i32;
    /**
    Gets the current temperature of the battery, as reported by VEXos.

    This function uses the following values of errno when an error state is
    reached:
    EACCES - Another resource is currently trying to access the battery port.

    \return The current temperature of the battery
    */
    pub fn battery_get_temperature() -> f64;
    /**
    Gets the current capacity of the battery, as reported by VEXos.

    This function uses the following values of errno when an error state is
    reached:
    EACCES - Another resource is currently trying to access the battery port.

    \return The current capacity of the battery
    */
    pub fn battery_get_capacity() -> f64;
    /**
    Checks if the SD card is installed.

    \return 1 if the SD card is installed, 0 otherwise
    */
    pub fn usd_is_installed() -> i32;
}
