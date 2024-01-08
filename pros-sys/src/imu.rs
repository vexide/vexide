use core::ffi::c_uint;

pub const IMU_MINIMUM_DATA_RATE: c_uint = 5;

pub const E_IMU_STATUS_CALIBRATING: c_uint = 0x01;
pub const E_IMU_STATUS_ERROR: c_uint = 0xFF;
pub type imu_status_e_t = c_uint;

#[repr(packed, C)]
pub struct quaternion_s_t {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[repr(C)]
pub struct imu_raw_s {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

pub type imu_gyro_s_t = imu_raw_s;
pub type imu_accel_s_t = imu_raw_s;

#[repr(packed, C)]
pub struct euler_s_t {
    pub pitch: f64,
    pub roll: f64,
    pub yaw: f64,
}

extern "C" {
    /**
    Calibrate IMU

    Calibration takes approximately 2 seconds, but this function only blocks
    until the IMU status flag is set properly to E_IMU_STATUS_CALIBRATING,
    with a minimum blocking time of 5ms.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is already calibrating, or time out setting the status flag.

    \param port
           The V5 Inertial Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed setting errno.
    */
    pub fn imu_reset(port: u8) -> i32;
    /**
    Calibrate IMU and Blocks while Calibrating

    Calibration takes approximately 2 seconds and blocks during this period,
    with a timeout for this operation being set a 3 seconds as a safety margin.
    Like the other reset function, this function also blocks until the IMU
    status flag is set properly to E_IMU_STATUS_CALIBRATING, with a minimum
    blocking time of 5ms and a timeout of 1 second if it's never set.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is already calibrating, or time out setting the status flag.

    \param port
           The V5 Inertial Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed (timing out or port claim failure), setting errno.
    */
    pub fn imu_reset_blocking(port: u8) -> i32;
    /**
    Set the Inertial Sensor's refresh interval in milliseconds.

    The rate may be specified in increments of 5ms, and will be rounded down to
    the nearest increment. The minimum allowable refresh rate is 5ms. The default
    rate is 10ms.

    As values are copied into the shared memory buffer only at 10ms intervals,
    setting this value to less than 10ms does not mean that you can poll the
    sensor's values any faster. However, it will guarantee that the data is as
    recent as possible.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param port
             The V5 Inertial Sensor port number from 1-21
    \param rate The data refresh interval in milliseconds
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_set_data_rate(port: u8, rate: u32) -> i32;
    /**
    Get the total number of degrees the Inertial Sensor has spun about the z-axis

    This value is theoretically unbounded. Clockwise rotations are represented
    with positive degree values, while counterclockwise rotations are represented
    with negative ones.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The degree value or PROS_ERR_F if the operation failed, setting
    errno.
    */
    pub fn imu_get_rotation(port: u8) -> f64;
    /**
    Get the Inertial Sensor's heading relative to the initial direction of its
    x-axis

    This value is bounded by [0,360). Clockwise rotations are represented with
    positive degree values, while counterclockwise rotations are represented with
    negative ones.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The degree value or PROS_ERR_F if the operation failed, setting
    errno.
    */
    pub fn imu_get_heading(port: u8) -> f64;
    /**
    Get a quaternion representing the Inertial Sensor's orientation

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The quaternion representing the sensor's orientation. If the
    operation failed, all the quaternion's members are filled with PROS_ERR_F and
    errno is set.
    */
    pub fn imu_get_quaternion(port: u8) -> quaternion_s_t;
    /**
    Get the Euler angles representing the Inertial Sensor's orientation

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The Euler angles representing the sensor's orientation. If the
    operation failed, all the structure's members are filled with PROS_ERR_F and
    errno is set.
    */
    pub fn imu_get_euler(port: u8) -> euler_s_t;
    /**
    Get the Inertial Sensor's pitch angle bounded by (-180,180)

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The pitch angle, or PROS_ERR_F if the operation failed, setting
    errno.
    */
    pub fn imu_get_pitch(port: u8) -> f64;
    /**
    Get the Inertial Sensor's roll angle bounded by (-180,180)

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The roll angle, or PROS_ERR_F if the operation failed, setting errno.
    */
    pub fn imu_get_roll(port: u8) -> f64;
    /**
    Get the Inertial Sensor's roll angle bounded by (-180,180)

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The roll angle, or PROS_ERR_F if the operation failed, setting errno.
    */
    pub fn imu_get_yaw(port: u8) -> f64;
    /**
    Get the Inertial Sensor's raw gyroscope values

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The raw gyroscope values. If the operation failed, all the
    structure's members are filled with PROS_ERR_F and errno is set.
    */
    pub fn imu_get_gyro_rate(port: u8) -> imu_gyro_s_t;
    /**
    Get the Inertial Sensor's raw accelerometer values

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The raw accelerometer values. If the operation failed, all the
    structure's members are filled with PROS_ERR_F and errno is set.
    */
    pub fn imu_get_accel(port: u8) -> imu_accel_s_t;
    /**
    Get the Inertial Sensor's status

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return The Inertial Sensor's status code, or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_get_status(port: u8) -> imu_status_e_t;
    /**
    Resets the current reading of the Inertial Sensor's heading to zero

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_tare_heading(port: u8) -> i32;
    /**
    Resets the current reading of the Inertial Sensor's rotation to zero

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_tare_rotation(port: u8) -> i32;
    /**
    Resets the current reading of the Inertial Sensor's pitch to zero

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_tare_pitch(port: u8) -> i32;
    /**
    Resets the current reading of the Inertial Sensor's roll to zero

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_tare_roll(port: u8) -> i32;
    /**
    Resets the current reading of the Inertial Sensor's yaw to zero

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_tare_yaw(port: u8) -> i32;
    /**
    Reset all 3 euler values of the Inertial Sensor to 0.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_tare_euler(port: u8) -> i32;
    /**
    Resets all 5 values of the Inertial Sensor to 0.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_tare(port: u8) -> i32;
    /**
    Sets the current reading of the Inertial Sensor's euler values to
    target euler values. Will default to +/- 180 if target exceeds +/- 180.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                    The V5 Inertial Sensor port number from 1-21
    \param  target
                    Target euler values for the euler values to be set to
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_set_euler(port: u8, target: euler_s_t) -> i32;
    /**
    Sets the current reading of the Inertial Sensor's rotation to target value

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \param  target
                     Target value for the rotation value to be set to
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_set_rotation(port: u8, target: f64) -> i32;
    /**
    Sets the current reading of the Inertial Sensor's heading to target value
    Target will default to 360 if above 360 and default to 0 if below 0.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \param  target
                     Target value for the heading value to be set to
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_set_heading(port: u8, target: f64) -> i32;
    /**
    Sets the current reading of the Inertial Sensor's pitch to target value
    Will default to +/- 180 if target exceeds +/- 180.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \param  target
                     Target value for the pitch value to be set to
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_set_pitch(port: u8, target: f64) -> i32;
    /**
    Sets the current reading of the Inertial Sensor's roll to target value
    Will default to +/- 180 if target exceeds +/- 180.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \param  target
                     Target value for the roll value to be set to
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_set_roll(port: u8, target: f64) -> i32;
    /**
    Sets the current reading of the Inertial Sensor's yaw to target value
    Will default to +/- 180 if target exceeds +/- 180.

    This function uses the following values of errno when an error state is
    reached:
    ENXIO - The given value is not within the range of V5 ports (1-21).
    ENODEV - The port cannot be configured as an Inertial Sensor
    EAGAIN - The sensor is still calibrating

    \param  port
                     The V5 Inertial Sensor port number from 1-21
    \param  target
                     Target value for the yaw value to be set to
    \return 1 if the operation was successful or PROS_ERR if the operation
    failed, setting errno.
    */
    pub fn imu_set_yaw(port: u8, target: f64) -> i32;
}
