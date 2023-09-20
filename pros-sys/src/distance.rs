use core::ffi::*;

extern "C" {
    /** Get the currently measured distance from the sensor in mm

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Distance Sensor

    \param  port The V5 Distance Sensor port number from 1-21
    \return The distance value or PROS_ERR if the operation failed, setting
    errno.*/
    pub fn distance_get(port: u8) -> i32;
    /** Get the confidence in the distance reading

    This is a value that has a range of 0 to 63. 63 means high confidence,
    lower values imply less confidence. Confidence is only available
    when distance is > 200mm (the value 10 is returned in this scenario).

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Distance Sensor

    \param  port The V5 Distance Sensor port number from 1-21
    \return The confidence value or PROS_ERR if the operation failed, setting
    errno.*/
    pub fn distance_get_confidence(port: u8) -> i32;
    /** Get the current guess at relative object size

    This is a value that has a range of 0 to 400.
    A 18" x 30" grey card will return a value of approximately 75
    in typical room lighting.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Distance Sensor

    \param  port The V5 Distance Sensor port number from 1-21
    \return The size value or PROS_ERR if the operation failed, setting
    errno.*/
    pub fn distance_get_object_size(port: u8) -> i32;
    /** Get the current guess at relative object size

    This is a value that has a range of 0 to 400.
    A 18" x 30" grey card will return a value of approximately 75
    in typical room lighting.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Distance Sensor

    \param  port The V5 Distance Sensor port number from 1-21
    \return The size value or PROS_ERR if the operation failed, setting
    errno.*/
    pub fn distance_get_object_velocity(port: u8) -> c_double;
}
