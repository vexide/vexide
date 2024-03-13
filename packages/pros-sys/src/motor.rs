use core::ffi::*;

pub const E_MOTOR_FAULT_NO_FAULTS: u32 = 0x00;
/// Analogous to motor_is_over_temp()
pub const E_MOTOR_FAULT_MOTOR_OVER_TEMP: u32 = 0x01;
/// Indicates a motor h-bridge fault
pub const E_MOTOR_FAULT_DRIVER_FAULT: u32 = 0x02;
/// Analogous to motor_is_over_current()
pub const E_MOTOR_FAULT_OVER_CURRENT: u32 = 0x04;
/// Indicates an h-bridge over current
pub const E_MOTOR_FAULT_DRV_OVER_CURRENT: u32 = 0x08;
pub type motor_fault_e_t = u32;

pub const E_MOTOR_FLAGS_NONE: u32 = 0x00;
/// Cannot currently communicate to the motor
pub const E_MOTOR_FLAGS_BUSY: u32 = 0x01;
/// Analogous to motor_is_stopped()
pub const E_MOTOR_FLAGS_ZERO_VELOCITY: u32 = 0x02;
pub const E_MOTOR_FLAGS_ZERO_POSITION: u32 = 0x04;
pub type motor_flag_e_t = u32;

/// Motor coasts when stopped, traditional behavior
pub const E_MOTOR_BRAKE_COAST: i32 = 0;
/// Motor brakes when stopped
pub const E_MOTOR_BRAKE_BRAKE: i32 = 1;
/// Motor actively holds position when stopped
pub const E_MOTOR_BRAKE_HOLD: i32 = 2;
pub const E_MOTOR_BRAKE_INVALID: i32 = i32::MAX;
pub type motor_brake_mode_e_t = i32;

/// Position is recorded as angle in degrees as a floating point number
pub const E_MOTOR_ENCODER_DEGREES: i32 = 0;
/// Position is recorded as angle in rotations as a floating point number
pub const E_MOTOR_ENCODER_ROTATIONS: i32 = 1;
/// Position is recorded as raw encoder ticks as a whole number
pub const E_MOTOR_ENCODER_COUNTS: i32 = 2;
pub const E_MOTOR_ENCODER_INVALID: i32 = i32::MAX;
pub type motor_encoder_units_e_t = i32;

/// 36:1, 100 RPM, Red gear set
pub const E_MOTOR_GEARSET_36: i32 = 0;
pub const E_MOTOR_GEAR_RED: i32 = E_MOTOR_GEARSET_36;
pub const E_MOTOR_GEAR_100: i32 = E_MOTOR_GEARSET_36;
/// 18:1, 200 RPM, Green gear set
pub const E_MOTOR_GEARSET_18: i32 = 1;
pub const E_MOTOR_GEAR_GREEN: i32 = E_MOTOR_GEARSET_18;
pub const E_MOTOR_GEAR_200: i32 = E_MOTOR_GEARSET_18;
/// 6:1, 600 RPM, Blue gear set
pub const E_MOTOR_GEARSET_06: i32 = 2;
pub const E_MOTOR_GEAR_BLUE: i32 = E_MOTOR_GEARSET_06;
pub const E_MOTOR_GEAR_600: i32 = E_MOTOR_GEARSET_06;
pub const E_MOTOR_GEARSET_INVALID: i32 = i32::MAX;
pub type motor_gearset_e_t = i32;

/**
Holds the information about a Motor's position or velocity PID controls.

These values are in 4.4 format, meaning that a value of 0x20 represents 2.0,
0x21 represents 2.0625, 0x22 represents 2.125, etc.
*/
#[repr(C)]
pub struct motor_pid_full_s_t {
    /// The feedforward constant
    pub kf: u8,
    /// The proportional constant
    pub kp: u8,
    /// The integral constants
    pub ki: u8,
    /// The derivative constant
    pub kd: u8,
    /// A constant used for filtering the profile acceleration
    pub filter: u8,
    /// The integral limit
    pub limit: u16,
    /// The threshold for determining if a position movement has
    /// reached its goal. This has no effect for velocity PID calculations.
    pub threshold: u8,
    /// The rate at which the PID computation is run in ms
    pub loopspeed: u8,
}

/**
Holds just the constants for a Motor's position or velocity PID controls.

These values are in 4.4 format, meaning that a value of 0x20 represents 2.0,
0x21 represents 2.0625, 0x22 represents 2.125, etc.
*/
#[repr(C)]
pub struct motor_pid_s_t {
    /// The feedforward constant
    pub kf: u8,
    /// The proportional constant
    pub kp: u8,
    /// The integral constant
    pub ki: u8,
    /// The derivative constant
    pub kd: u8,
}

extern "C" {
    /**
    Sets the voltage for the motor from -127 to 127.

    This is designed to map easily to the input from the controller's analog
    stick for simple opcontrol use. The actual behavior of the motor is analogous
    to use of motor_move_voltage(), or motorSet() from the PROS 2 API.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param voltage
           The new motor voltage from -127 to 127

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_move(port: i8, voltage: i32) -> i32;
    /**
    Stops the motor using the currently configured brake mode.

    This function sets motor velocity to zero, which will cause it to act
    according to the set brake mode. If brake mode is set to MOTOR_BRAKE_HOLD,
    this function may behave differently than calling motor_move_absolute(port, 0)
    or motor_move_relative(port, 0).

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_brake(port: i8) -> i32;
    /**
    Sets the target absolute position for the motor to move to.

    This movement is relative to the position of the motor when initialized or
    the position when it was most recently reset with motor_set_zero_position().

    \note This function simply sets the target for the motor, it does not block
    program execution until the movement finishes.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param position
           The absolute position to move to in the motor's encoder units
    \param velocity
           The maximum allowable velocity for the movement in RPM

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_move_absolute(port: i8, position: c_double, velocity: i32) -> i32;
    /**
    Sets the relative target position for the motor to move to.

    This movement is relative to the current position of the motor as given in
    motor_get_position(). Providing 10.0 as the position parameter would result
    in the motor moving clockwise 10 units, no matter what the current position
    is.

    \note This function simply sets the target for the motor, it does not block
    program execution until the movement finishes.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param position
           The relative position to move to in the motor's encoder units
    \param velocity
           The maximum allowable velocity for the movement in RPM

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_move_relative(port: i8, position: c_double, velocity: i32) -> i32;
    /**
    Sets the velocity for the motor.

    This velocity corresponds to different actual speeds depending on the gearset
    used for the motor. This results in a range of +-100 for E_MOTOR_GEARSET_36,
    +-200 for E_MOTOR_GEARSET_18, and +-600 for E_MOTOR_GEARSET_6. The velocity
    is held with PID to ensure consistent speed, as opposed to setting the
    motor's voltage.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param velocity
           The new motor velocity from +-100, +-200, or +-600 depending on the
           motor's gearset

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_move_velocity(port: i8, velocity: i32) -> i32;
    /**
    Sets the output voltage for the motor from -12000 to 12000 in millivolts

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param voltage
           The new voltage value from -12000 to 12000

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_move_voltage(port: i8, voltage: i32) -> i32;
    /**
    Changes the output velocity for a profiled movement (motor_move_absolute or
    motor_move_relative). This will have no effect if the motor is not following
    a profiled movement.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param velocity
           The new motor velocity from +-100, +-200, or +-600 depending on the
           motor's gearset

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_modify_profiled_velocity(port: i8, velocity: i32) -> i32;
    /**
    Gets the target position set for the motor by the user.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The target position in its encoder units or PROS_ERR_F if the
    operation failed, setting errno.
    */
    pub fn motor_get_target_position(port: i8) -> c_double;
    /**
    Gets the velocity commanded to the motor by the user.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The commanded motor velocity from +-100, +-200, or +-600, or PROS_ERR
    if the operation failed, setting errno.
    */
    pub fn motor_get_target_velocity(port: i8) -> i32;
    /**
    Gets the actual velocity of the motor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The motor's actual velocity in RPM or PROS_ERR_F if the operation
    failed, setting errno.
    */
    pub fn motor_get_actual_velocity(port: i8) -> c_double;
    /**
    Gets the current drawn by the motor in mA.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The motor's current in mA or PROS_ERR if the operation failed,
    setting errno.
    */
    pub fn motor_get_current_draw(port: i8) -> i32;
    /**
    Gets the direction of movement for the motor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return 1 for moving in the positive direction, -1 for moving in the
    negative direction, or PROS_ERR if the operation failed, setting errno.
    */
    pub fn motor_get_direction(port: i8) -> i32;
    /**
    Gets the efficiency of the motor in percent.

    An efficiency of 100% means that the motor is moving electrically while
    drawing no electrical power, and an efficiency of 0% means that the motor
    is drawing power but not moving.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The motor's efficiency in percent or PROS_ERR_F if the operation
    failed, setting errno.
    */
    pub fn motor_get_efficiency(port: i8) -> c_double;
    /**
    Checks if the motor is drawing over its current limit.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return 1 if the motor's current limit is being exceeded and 0 if the current
    limit is not exceeded, or PROS_ERR if the operation failed, setting errno.
    */
    pub fn motor_is_over_current(port: i8) -> i32;
    /**
    Checks if the motor's temperature is above its limit.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return 1 if the temperature limit is exceeded and 0 if the the temperature
    is below the limit, or PROS_ERR if the operation failed, setting errno.
    */
    pub fn motor_is_over_temp(port: i8) -> i32;
    /**
    Checks if the motor is stopped.

    \note Although this function forwards data from the motor, the motor
    presently does not provide any value. This function returns PROS_ERR with
    errno set to ENOSYS.

    \param port
           The V5 port number from 1-21

    \return 1 if the motor is not moving, 0 if the motor is moving, or PROS_ERR
    if the operation failed, setting errno
    */
    pub fn motor_is_stopped(port: i8) -> i32;
    /**
    Checks if the motor is at its zero position.

    \note Although this function forwards data from the motor, the motor
    presently does not provide any value. This function returns PROS_ERR with
    errno set to ENOSYS.

    \param port
           The V5 port number from 1-21

    \return 1 if the motor is at zero absolute position, 0 if the motor has
    moved from its absolute zero, or PROS_ERR if the operation failed,
    setting errno
    */
    pub fn motor_get_zero_position_flag(port: i8) -> i32;

    /**
    Gets the faults experienced by the motor.

    Compare this bitfield to the bitmasks in motor_fault_e_t.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return A bitfield containing the motor's faults.
    */
    pub fn motor_get_faults(port: i8) -> motor_fault_e_t;
    /**
    Gets the flags set by the motor's operation.

    Compare this bitfield to the bitmasks in motor_flag_e_t.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return A bitfield containing the motor's flags.
    */
    pub fn motor_get_flags(port: i8) -> motor_flag_e_t;

    /**
    Gets the raw encoder count of the motor at a given timestamp.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param\[in] timestamp
               A pointer to a time in milliseconds for which the encoder count
               will be returned. If NULL, the timestamp at which the encoder
               count was read will not be supplied

    \return The raw encoder count at the given timestamp or PROS_ERR if the
    operation failed.
    */
    pub fn motor_get_raw_position(port: i8, timestamp: *const u32) -> i32;
    /**
    Gets the absolute position of the motor in its encoder units.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The motor's absolute position in its encoder units or PROS_ERR_F
    if the operation failed, setting errno.
    */
    pub fn motor_get_position(port: i8) -> c_double;
    /**
    Gets the power drawn by the motor in Watts.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The motor's power draw in Watts or PROS_ERR_F if the operation
    failed, setting errno.
    */
    pub fn motor_get_power(port: i8) -> c_double;
    /**
    Gets the temperature of the motor in degrees Celsius.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The motor's temperature in degrees Celsius or PROS_ERR_F if the
    operation failed, setting errno.
    */
    pub fn motor_get_temperature(port: i8) -> c_double;
    /**
    Gets the torque generated by the motor in Newton Meters (Nm).

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The motor's torque in Nm or PROS_ERR_F if the operation failed,
    setting errno.
    */
    pub fn motor_get_torque(port: i8) -> c_double;
    /**
    Gets the voltage delivered to the motor in millivolts.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The motor's voltage in mV or PROS_ERR_F if the operation failed,
    setting errno.
    */
    pub fn motor_get_voltage(port: i8) -> i32;

    /**
    Sets the position for the motor in its encoder units.

    This will be the future reference point for the motor's "absolute" position.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param position
           The new reference position in its encoder units

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_set_zero_position(port: i8, position: c_double) -> i32;
    /**
    Sets the "absolute" zero position of the motor to its current position.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_tare_position(port: i8) -> i32;
    /**
    Sets one of motor_brake_mode_e_t to the motor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param mode
           The motor_brake_mode_e_t to set for the motor

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_set_brake_mode(port: i8, mode: motor_brake_mode_e_t) -> i32;
    /**
    Sets the current limit for the motor in mA.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param limit
           The new current limit in mA

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_set_current_limit(port: i8, limit: i32) -> i32;
    /**
    Sets one of motor_encoder_units_e_t for the motor encoder.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param units
           The new motor encoder units

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_set_encoder_units(port: i8, units: motor_encoder_units_e_t) -> i32;
    /**
    Sets one of motor_gearset_e_t for the motor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param gearset
           The new motor gearset

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_set_gearing(port: i8, gearset: motor_gearset_e_t) -> i32;

    /**
    Takes in floating point values and returns a properly formatted pid struct.
    The motor_pid_s_t struct is in 4.4 format, i.e. 0x20 is 2.0, 0x21 is 2.0625,
    etc.
    This function will convert the floating point values to the nearest 4.4
    value.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param kf
           The feedforward constant
    \param kp
           The proportional constant
    \param ki
           The integral constant
    \param kd
           The derivative constant

    \return A motor_pid_s_t struct formatted properly in 4.4.
    */
    #[deprecated(
        note = "Changing these values is not supported by VEX and may lead to permanent motor damage."
    )]
    pub fn motor_convert_pid(
        kf: c_double,
        kp: c_double,
        ki: c_double,
        kd: c_double,
    ) -> motor_pid_s_t;
    /**
    Takes in floating point values and returns a properly formatted pid struct.
    The motor_pid_s_t struct is in 4.4 format, i.e. 0x20 is 2.0, 0x21 is 2.0625,
    etc.
    This function will convert the floating point values to the nearest 4.4
    value.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param kf
           The feedforward constant
    \param kp
           The proportional constant
    \param ki
           The integral constant
    \param kd
           The derivative constant
    \param filter
           A constant used for filtering the profile acceleration
    \param limit
           The integral limit
    \param threshold
           The threshold for determining if a position movement has reached its
           goal. This has no effect for velocity PID calculations.
    \param loopspeed
           The rate at which the PID computation is run in ms

    \return A motor_pid_s_t struct formatted properly in 4.4.
    */
    #[deprecated(
        note = "Changing these values is not supported by VEX and may lead to permanent motor damage."
    )]
    pub fn motor_convert_pid_full(
        kf: c_double,
        kp: c_double,
        ki: c_double,
        kd: c_double,
        filter: c_double,
        limit: c_double,
        threshold: c_double,
        loopspeed: c_double,
    ) -> motor_pid_full_s_t;
    /**
    Sets one of motor_pid_s_t for the motor. This intended to just modify the
    main PID constants.

    Only non-zero values of the struct will change the existing motor constants.

    \note This feature is in beta, it is advised to use caution when modifying
    the PID values. The motor could be damaged by particularly large constants.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param pid
           The new motor PID constants

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    #[deprecated(
        note = "Changing these values is not supported by VEX and may lead to permanent motor damage."
    )]
    pub fn motor_set_pos_pid(port: i8, pid: motor_pid_s_t) -> i32;
    /**
    Sets one of motor_pid_full_s_t for the motor.

    Only non-zero values of the struct will change the existing motor constants.

    \note This feature is in beta, it is advised to use caution when modifying
    the PID values. The motor could be damaged by particularly large constants.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param pid
           The new motor PID constants

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    #[deprecated(
        note = "Changing these values is not supported by VEX and may lead to permanent motor damage."
    )]
    pub fn motor_set_pos_pid_full(port: i8, pid: motor_pid_full_s_t) -> i32;
    /**
    Sets one of motor_pid_s_t for the motor. This intended to just modify the
    main PID constants.

    Only non-zero values of the struct will change the existing motor constants.

    \note This feature is in beta, it is advised to use caution when modifying
    the PID values. The motor could be damaged by particularly large constants.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param pid
           The new motor PID constants

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    #[deprecated(
        note = "Changing these values is not supported by VEX and may lead to permanent motor damage."
    )]
    pub fn motor_set_vel_pid(port: i8, pid: motor_pid_s_t) -> i32;
    /**
    Sets one of motor_pid_full_s_t for the motor.

    Only non-zero values of the struct will change the existing motor constants.

    \note This feature is in beta, it is advised to use caution when modifying
    the PID values. The motor could be damaged by particularly large constants.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param pid
           The new motor PID constants

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    #[deprecated(
        note = "Changing these values is not supported by VEX and may lead to permanent motor damage."
    )]
    pub fn motor_set_vel_pid_full(port: i8, pid: motor_pid_full_s_t) -> i32;
    /**
    Gets the position PID that was set for the motor. This function will return
    zero for all of the parameters if the motor_set_pos_pid() or
    motor_set_pos_pid_full() functions have not been used.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    Additionally, in an error state all values of the returned struct are set
    to their negative maximum values.

    \param port
           The V5 port number from 1-21

    \return A motor_pid_full_s_t containing the position PID constants last set
    to the given motor
    */
    #[deprecated(
        note = "Changing these values is not supported by VEX and may lead to permanent motor damage."
    )]
    pub fn motor_get_pos_pid(port: i8) -> motor_pid_full_s_t;
    /**
    Gets the velocity PID that was set for the motor. This function will return
    zero for all of the parameters if the motor_set_vel_pid() or
    motor_set_vel_pid_full() functions have not been used.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    Additionally, in an error state all values of the returned struct are set
    to their negative maximum values.

    \param port
           The V5 port number from 1-21

    \return A motor_pid_full_s_t containing the velocity PID constants last set
    to the given motor
    */
    #[deprecated(
        note = "Changing these values is not supported by VEX and may lead to permanent motor damage."
    )]
    pub fn motor_get_vel_pid(port: i8) -> motor_pid_full_s_t;
    /**
    Sets the reverse flag for the motor.

    This will invert its movements and the values returned for its position.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param reverse
           True reverses the motor, false is default

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_set_reversed(port: i8, reversed: bool) -> i32;
    /**
    Sets the voltage limit for the motor in Volts.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21
    \param limit
           The new voltage limit in Volts

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn motor_set_voltage_limit(port: i8, limit: i32) -> i32;
    /**
    Gets the brake mode that was set for the motor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return One of motor_brake_mode_e_t, according to what was set for the motor,
    or E_MOTOR_BRAKE_INVALID if the operation failed, setting errno.
    */
    pub fn motor_get_brake_mode(port: i8) -> motor_brake_mode_e_t;
    /**
    Gets the current limit for the motor in mA.

    The default value is 2500 mA.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return The motor's current limit in mA or PROS_ERR if the operation failed,
    setting errno.
    */
    pub fn motor_get_current_limit(port: i8) -> i32;
    /**
    Gets the encoder units that were set for the motor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return One of motor_encoder_units_e_t according to what is set for the motor
    or E_MOTOR_ENCODER_INVALID if the operation failed.
    */
    pub fn motor_get_encoder_units(port: i8) -> motor_encoder_units_e_t;
    /**
    Gets the gearset that was set for the motor.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return One of motor_gearset_e_t according to what is set for the motor,
    or E_GEARSET_INVALID if the operation failed.
    */
    pub fn motor_get_gearing(port: i8) -> motor_gearset_e_t;
    /**
    Gets the operation direction of the motor as set by the user.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a motor

    \param port
           The V5 port number from 1-21

    \return 1 if the motor has been reversed and 0 if the motor was not reversed,
    or PROS_ERR if the operation failed, setting errno.
    */
    pub fn motor_is_reversed(port: i8) -> i32;
    /**
       Gets the voltage limit set by the user.

       Default value is 0V, which means that there is no software limitation imposed
       on the voltage.

       This function uses the following values of errno when an error state is
       reached:
       ENXIO - The given value is not within the range of V5 ports (1-21).
       ENODEV - The port cannot be configured as a motor

       \param port
              The V5 port number from 1-21

       \return The motor's voltage limit in V or PROS_ERR if the operation failed,
       setting errno.
    */
    pub fn motor_get_voltage_limit(port: i8) -> i32;
}
