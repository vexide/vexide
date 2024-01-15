use core::ffi::c_uint;

pub const ROTATION_MINIMUM_DATA_RATE: c_uint = 5;

extern "C" {
    /**
    Reset Rotation Sensor

    Reset the current absolute position to be the same as the
    Rotation Sensor angle.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param port
           The V5 Rotation Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn rotation_reset(port: u8) -> i32;
    /**
    Set the Rotation Sensor's refresh interval in milliseconds.

    The rate may be specified in increments of 5ms, and will be rounded down to
    the nearest increment. The minimum allowable refresh rate is 5ms. The default
    rate is 10ms.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param port
           The V5 Rotation Sensor port number from 1-21
    \param rate The data refresh interval in milliseconds
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn rotation_set_data_rate(port: u8, rate: u32) -> i32;
    /**
    Set the Rotation Sensor position reading to a desired rotation value

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param port
             The V5 Rotation Sensor port number from 1-21
    \param position
              The position in terms of ticks
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn rotation_set_position(port: u8, position: u32) -> i32;
    /**
    Reset the Rotation Sensor position to 0

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param port
             The V5 Rotation Sensor port number from 1-2
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn rotation_reset_position(port: u8) -> i32;
    /**
    Get the Rotation Sensor's current position in centidegrees

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param  port
                     The V5 Rotation Sensor port number from 1-21
    \return The position value or PROS_ERR_F if the operation failed, setting
    errno.
    */
    pub fn rotation_get_position(port: u8) -> i32;
    /**
    Get the Rotation Sensor's current velocity in centidegrees per second

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param  port
                     The V5 Rotation Sensor port number from 1-21
    \return The velocity value or PROS_ERR_F if the operation failed, setting
    errno.
    */
    pub fn rotation_get_velocity(port: u8) -> i32;
    /**
    Get the Rotation Sensor's current angle in centidegrees (0-36000)

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param  port
                     The V5 Rotation Sensor port number from 1-21
    \return The angle value (0-36000) or PROS_ERR_F if the operation failed, setting
    errno.
    */
    pub fn rotation_get_angle(port: u8) -> i32;
    /**
    Set the Rotation Sensor's direction reversed flag

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param  port
                     The V5 Rotation Sensor port number from 1-21
    \param  value
                     Determines if the direction of the Rotation Sensor is reversed or not.

    \return 1 if operation succeeded or PROS_ERR if the operation failed, setting
    errno.
    */
    pub fn rotation_set_reversed(port: u8, value: bool) -> i32;
    /**
    Reverse the Rotation Sensor's direction

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param  port
                     The V5 Rotation Sensor port number from 1-21

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn rotation_reverse(port: u8) -> i32;
    /**
    Initialize the Rotation Sensor with a reverse flag

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param  port
                     The V5 Rotation Sensor port number from 1-21
    \param  reverse_flag
                     Determines if the Rotation Sensor is reversed or not.

    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn rotation_init_reverse(port: u8, reverse_flag: bool) -> i32;
    /**
    Get the Rotation Sensor's reversed flag

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Rotation Sensor

    \param  port
                     The V5 Rotation Sensor port number from 1-21

    \return Boolean value of if the Rotation Sensor's direction is reversed or not
    or PROS_ERR if the operation failed, setting errno.
    */
    pub fn rotation_get_reversed(port: u8) -> i32;
}
