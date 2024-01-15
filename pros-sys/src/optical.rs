use core::ffi::*;

use crate::PROS_ERR;

pub const OPT_GESTURE_ERR: i8 = i8::MAX;
pub const OPT_COUNT_ERR: i16 = i16::MAX;
pub const OPT_TIME_ERR: i32 = PROS_ERR;

#[repr(C)]
pub struct optical_rgb_s_t {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub brightness: f64,
}

#[repr(C)]
pub struct optical_raw_s_t {
    pub clear: u32,
    pub red: u32,
    pub green: u32,
    pub blue: u32,
}

#[repr(C)]
pub struct optical_gesture_s_t {
    pub udata: u8,
    pub ddata: u8,
    pub ldata: u8,
    pub rdata: u8,
    pub r#type: u8,
    pub pad: u8,
    pub count: u16,
    pub time: u32,
}

pub const E_OPTICAL_DIRECTION_NO_GESTURE: c_int = 0;
pub const E_OPTICAL_DIRECTION_UP: c_int = 1;
pub const E_OPTICAL_DIRECTION_DOWN: c_int = 2;
pub const E_OPTICAL_DIRECTION_RIGHT: c_int = 3;
pub const E_OPTICAL_DIRECTION_LEFT: c_int = 4;
pub const E_OPTICAL_DIRECTION_ERROR: c_int = PROS_ERR;
pub type optical_direction_e_t = c_int;

extern "C" {

    /// Get the detected color hue
    ///
    /// This is not available if gestures are being detected. Hue has a
    /// range of 0 to 359.999
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return hue value if the operation was successful or PROS_ERR_F if the operation
    /// failed, setting errno.
    pub fn optical_get_hue(port: u8) -> f64;

    /// Get the detected color saturation
    ///
    /// This is not available if gestures are being detected. Saturation has a
    /// range of 0 to 1.0
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return saturation value if the operation was successful or PROS_ERR_F if
    /// the operation failed, setting errno.
    pub fn optical_get_saturation(port: u8) -> f64;

    /// Get the detected color brightness
    ///
    /// This is not available if gestures are being detected. Brightness has a
    /// range of 0 to 1.0
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return brightness value if the operation was successful or PROS_ERR_F if
    /// the operation failed, setting errno.
    pub fn optical_get_brightness(port: u8) -> f64;

    /// Get the detected proximity value
    ///
    /// This is not available if gestures are being detected. proximity has
    /// a range of 0 to 255.
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return poximity value if the operation was successful or PROS_ERR if
    /// the operation failed, setting errno.
    pub fn optical_get_proximity(port: u8) -> i32;

    /// Set the pwm value of the White LED
    ///
    /// value that ranges from 0 to 100
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return 1 if the operation is successful or PROS_ERR if the operation failed,
    /// setting errno.
    pub fn optical_set_led_pwm(port: u8, value: u8) -> i32;

    /// Get the pwm value of the White LED
    ///
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return LED pwm value that ranges from 0 to 100 if the operation was
    /// successful or PROS_ERR if the operation failed, setting errno.
    pub fn optical_get_led_pwm(port: u8) -> i32;

    /// Get the processed RGBC data from the sensor
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return rgb value if the operation was successful or an optical_rgb_s_t with
    /// all fields set to PROS_ERR if the operation failed, setting errno.
    pub fn optical_get_rgb(port: u8) -> optical_rgb_s_t;

    /// Get the raw, unprocessed RGBC data from the sensor
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return raw rgb value if the operation was successful or an optical_raw_s_t
    /// with all fields set to PROS_ERR if the operation failed, setting errno.
    pub fn optical_get_raw(port: u8) -> optical_raw_s_t;

    /// Get the most recent gesture data from the sensor
    ///
    /// Gestures will be cleared after 500mS
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return gesture value if the operation was successful or PROS_ERR if
    /// the operation failed, setting errno.
    pub fn optical_get_gesture(port: u8) -> optical_direction_e_t;

    /// Get the most recent raw gesture data from the sensor
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return gesture value if the operation was successful or an optical_gesture_s_t
    /// with all fields set to PROS_ERR if the operation failed, setting errno.
    pub fn optical_get_gesture_raw(port: u8) -> optical_gesture_s_t;

    /// Enable gesture detection on the sensor
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return 1 if the operation is successful or PROS_ERR if the operation failed,
    /// setting errno.
    pub fn optical_enable_gesture(port: u8) -> i32;

    /// Disable gesture detection on the sensor
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return 1 if the operation is successful or PROS_ERR if the operation failed,
    /// setting errno.
    pub fn optical_disable_gesture(port: u8) -> i32;

    /// Get integration time (update rate) of the optical sensor in milliseconds, with
    /// minimum time being
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \return Integration time in milliseconds if the operation is successful
    /// or PROS_ERR if the operation failed, setting errno.
    pub fn optical_get_integration_time(port: u8) -> f64;

    /// Set integration time (update rate) of the optical sensor in milliseconds.
    ///
    /// This function uses the following values of errno when an error state is
    /// reached:
    /// ENXIO - The given value is not within the range of V5 ports (1-21).
    /// ENODEV - The port cannot be configured as an Optical Sensor
    ///
    /// \param port
    ///        The V5 Optical Sensor port number from 1-21
    /// \param time
    ///        The desired integration time in milliseconds
    /// \return 1 if the operation is successful or PROS_ERR if the operation failed,
    /// setting errno.
    pub fn optical_set_integration_time(port: u8, time: f64) -> i32;
}
