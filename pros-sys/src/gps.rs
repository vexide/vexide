#[repr(packed, C)]
pub struct gps_status_s_t {
    ///< X Position (meters)
    pub x: f64,
    ///< Y Position (meters)
    pub y: f64,
    ///< Perceived Pitch based on GPS + IMU
    pub pitch: f64,
    ///< Perceived Roll based on GPS + IMU
    pub roll: f64,
    ///< Perceived Yaw based on GPS + IMU
    pub yaw: f64,
}

#[repr(C)]
pub struct gps_raw_s {
    ///< Perceived Pitch based on GPS + IMU
    pub x: f64,
    ///< Perceived Roll based on GPS + IMU
    pub y: f64,
    ///< Perceived Yaw based on GPS + IMU
    pub z: f64,
}

pub type gps_accel_s_t = gps_raw_s;
pub type gps_gyro_s_t = gps_raw_s;

extern "C" {
    /**
    Set the GPS's offset relative to the center of turning in meters,
    as well as its initial position.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21
    \param  xOffset
                     Cartesian 4-Quadrant X offset from center of turning (meters)
    \param  yOffset
                     Cartesian 4-Quadrant Y offset from center of turning (meters)
    \param  xInitial
                     Initial 4-Quadrant X Position, with (0,0) being at the center of the field (meters)
    \param  yInitial
                     Initial 4-Quadrant Y Position, with (0,0) being at the center of the field (meters)
    \param  headingInitial
                  Heading with 0 being north on the field, in degrees [0,360) going clockwise
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn gps_initialize_full(
        port: u8,
        xInitial: f64,
        yInitial: f64,
        headingInitial: f64,
        xOffset: f64,
        yOffset: f64,
    ) -> i32;
    /**
    Set the GPS's offset relative to the center of turning in meters.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21
    \param  xOffset
                     Cartesian 4-Quadrant X offset from center of turning (meters)
    \param  yOffset
                     Cartesian 4-Quadrant Y offset from center of turning (meters)
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn gps_set_offset(port: u8, xOffset: f64, yOffset: f64) -> i32;
    /**
    Get the GPS's location relative to the center of turning/origin in meters.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21
    \param  xOffset
                     Pointer to cartesian 4-Quadrant X offset from center of turning (meters)
    \param  yOffset
                     Pointer to cartesian 4-Quadrant Y offset from center of turning (meters)
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn gps_get_offset(port: u8, xOffset: *mut f64, yOffset: *mut f64) -> i32;
    /**
    Sets the robot's location relative to the center of the field in meters.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21
    \param  xInitial
                     Initial 4-Quadrant X Position, with (0,0) being at the center of the field (meters)
    \param  yInitial
                     Initial 4-Quadrant Y Position, with (0,0) being at the center of the field (meters)
    \param  headingInitial
                  Heading with 0 being north on the field, in degrees [0,360) going clockwise
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn gps_set_position(port: u8, xInitial: f64, yInitial: f64, headingInitial: f64) -> i32;
    /**
    Set the GPS sensor's data rate in milliseconds, only applies to IMU on GPS.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21
    \param  rate
                     Data rate in milliseconds (Minimum: 5 ms)
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn gps_set_data_rate(port: u8, rate: u32) -> i32;
    /**
    Get the possible RMS (Root Mean Squared) error in meters for GPS position.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21

    \return Possible RMS (Root Mean Squared) error in meters for GPS position.
    If the operation failed, returns PROS_ERR_F and errno is set.
    */
    pub fn gps_get_error(port: u8) -> f64;
    /**
    Gets the position and roll, yaw, and pitch of the GPS.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21

    \return A struct (gps_status_s_t) containing values mentioned above.
    If the operation failed, all the structure's members are filled with
    PROS_ERR_F and errno is set.
    */
    pub fn gps_get_status(port: u8) -> gps_status_s_t;
    /**
    Get the heading in [0,360) degree values.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21

    \return The heading in [0,360) degree values. If the operation failed,
    returns PROS_ERR_F and errno is set.
    */
    pub fn gps_get_heading(port: u8) -> f64;
    /**
    Get the heading in the max double value and min double value scale.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21

    \return The heading in [DOUBLE_MIN, DOUBLE_MAX] values. If the operation
    fails, returns PROS_ERR_F and errno is set.
    */
    pub fn gps_get_heading_raw(port: u8) -> f64;
    /**
    Gets the GPS sensor's elapsed rotation value

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21
    \return The elapsed heading in degrees. If the operation fails, returns
    PROS_ERR_F and errno is set.
    */
    pub fn gps_get_rotation(port: u8) -> f64;
    /**
    Set the GPS sensor's rotation value to target value

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21
    \param  target
                     Target rotation value to set rotation value to
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn gps_set_rotation(port: u8, targe: f64) -> i32;
    /**
    Tare the GPS sensor's rotation value

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn gps_tare_rotation(port: u8) -> i32;
    /**
    Get the GPS's raw gyroscope values

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as a GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS port number from 1-21
    \return The raw gyroscope values. If the operation failed, all the
    structure's members are filled with PROS_ERR_F and errno is set.
    */
    pub fn gps_get_gyro_rate(port: u8) -> gps_gyro_s_t;
    /**
    Get the GPS's raw accelerometer values

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an GPS
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 GPS's port number from 1-21
    \return The raw accelerometer values. If the operation failed, all the
    structure's members are filled with PROS_ERR_F and errno is set.
    */
    pub fn gps_get_accel(port: u8) -> gps_accel_s_t;
}
