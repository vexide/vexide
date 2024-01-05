use core::time::Duration;
use pros_sys::{OPT_GESTURE_ERR, PROS_ERR, PROS_ERR_F};
use snafu::Snafu;

use crate::error::{bail_on, map_errno, PortError};

pub const MIN_INTEGRATION_TIME: Duration = Duration::from_millis(3);
pub const MAX_INTEGRATION_TIME: Duration = Duration::from_millis(712);

pub const MAX_LED_PWM: u8 = 100;

#[derive(Debug)]
pub struct OpticalSensor {
    port: u8,
    gesture_detection_enabled: bool,
}

impl OpticalSensor {
    /// Creates a new inertial sensor from a smart port index.
    ///
    /// Gesture detection features can be optionally enabled, allowing the use of [`Self::last_gesture_direction()`] and [`Self::last_gesture_direction()`].
    pub fn new(port: u8, gesture_detection_enabled: bool) -> Result<Self, OpticalError> {
        let mut sensor = Self {
            port,
            gesture_detection_enabled,
        };

        if gesture_detection_enabled {
            sensor.enable_gesture_detection()?;
        } else {
            sensor.disable_gesture_detection()?;
        }

        Ok(sensor)
    }

    /// Get the pwm value of the White LED. PWM value ranges from 0 to 100.
    pub fn led_pwm(&self) -> Result<i32, OpticalError> {
        unsafe { Ok(bail_on!(PROS_ERR, pros_sys::optical_get_led_pwm(self.port))) }
    }

    /// Sets the pwm value of the White LED. Valid values are in the range `0` `100`.
    pub fn set_led_pwm(&self, value: u8) -> Result<(), OpticalError> {
        if value > MAX_LED_PWM {
            return Err(OpticalError::InvalidLedPwm);
        }
        unsafe {
            bail_on!(PROS_ERR, pros_sys::optical_set_led_pwm(self.port, value));
        }
        Ok(())
    }

    /// Get integration time (update rate) of the optical sensor in milliseconds, with
    /// minimum time being 3ms and the maximum time being 712ms.
    pub fn integration_time(&self) -> Result<Duration, OpticalError> {
        unsafe {
            Ok(Duration::from_millis(bail_on!(
                PROS_ERR_F,
                pros_sys::optical_get_integration_time(self.port)
            ) as u64))
        }
    }

    /// Set integration time (update rate) of the optical sensor.
    ///
    /// Lower integration time results in faster update rates with lower accuracy
    /// due to less available light being read by the sensor.
    ///
    /// Time value must be a [`Duration`] between 3 and 712 milliseconds. See
    /// https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9 for
    /// more information.
    pub fn set_integration_time(&self, time: Duration) -> Result<(), OpticalError> {
        if time < MIN_INTEGRATION_TIME || time > MAX_INTEGRATION_TIME {
            return Err(OpticalError::InvalidIntegrationTime);
        }

        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::optical_set_integration_time(self.port, time.as_millis() as f64)
            );
        }

        Ok(())
    }

    /// Get the detected color hue.
    ///
    /// Hue has a range of `0` to `359.999`.
    pub fn hue(&self) -> Result<f64, OpticalError> {
        unsafe { Ok(bail_on!(PROS_ERR_F, pros_sys::optical_get_hue(self.port))) }
    }

    /// Gets the detected color saturation.
    ///
    /// Saturation has a range `0` to `1.0`.
    pub fn saturation(&self) -> Result<f64, OpticalError> {
        unsafe {
            Ok(bail_on!(
                PROS_ERR_F,
                pros_sys::optical_get_saturation(self.port)
            ))
        }
    }

    /// Get the detected color brightness.
    ///
    /// Brightness values range from `0` to `1.0`.
    pub fn brightness(&self) -> Result<f64, OpticalError> {
        unsafe {
            Ok(bail_on!(
                PROS_ERR_F,
                pros_sys::optical_get_brightness(self.port)
            ))
        }
    }

    /// Get the detected proximity value
    ///
    /// Proximity has a range of `0` to `255`.
    pub fn proximity(&self) -> Result<i32, OpticalError> {
        unsafe {
            Ok(bail_on!(
                PROS_ERR,
                pros_sys::optical_get_proximity(self.port)
            ))
        }
    }

    /// Get the processed RGBC data from the sensor
    pub fn rgbc(&self) -> Result<Rgbc, OpticalError> {
        unsafe { pros_sys::optical_get_rgb(self.port).try_into() }
    }

    /// Get the raw, unprocessed RGBC data from the sensor
    pub fn rgbc_raw(&self) -> Result<RgbcRaw, OpticalError> {
        unsafe { pros_sys::optical_get_raw(self.port).try_into() }
    }

    /// Enables gesture detection features on the sensor.
    ///
    /// This allows [`Self::last_gesture_direction()`] and [`Self::last_gesture_direction()`] to be called without error, if
    /// gesture detection wasn't already enabled.
    pub fn enable_gesture_detection(&mut self) -> Result<(), OpticalError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::optical_enable_gesture(self.port)
        });

        self.gesture_detection_enabled = true;
        Ok(())
    }

    /// Disables gesture detection features on the sensor.
    pub fn disable_gesture_detection(&mut self) -> Result<(), OpticalError> {
        bail_on!(PROS_ERR, unsafe {
            pros_sys::optical_disable_gesture(self.port)
        });

        self.gesture_detection_enabled = false;
        Ok(())
    }

    /// Determine if gesture detection is enabled or not on the sensor.
    pub fn gesture_detection_enabled(&self) -> bool {
        self.gesture_detection_enabled
    }

    /// Get the most recent gesture data from the sensor. Gestures will be cleared after 500mS.
    ///
    /// Will return [`OpticalError::GestureDetectionNotEnabled`] if the sensor is not
    /// confgured to detect gestures.
    pub fn last_gesture_direction(&self) -> Result<GestureDirection, OpticalError> {
        if !self.gesture_detection_enabled {
            return Err(OpticalError::GestureDetectionDisabled);
        }

        unsafe { pros_sys::optical_get_gesture(self.port).try_into() }
    }

    /// Get the most recent raw gesture data from the sensor.
    ///
    /// Will return [`OpticalError::GestureDetectionNotEnabled`] if the sensor is not
    /// confgured to detect gestures.
    pub fn last_gesture_raw(&self) -> Result<GestureRaw, OpticalError> {
        if !self.gesture_detection_enabled {
            return Err(OpticalError::GestureDetectionDisabled);
        }

        unsafe { pros_sys::optical_get_gesture_raw(self.port).try_into() }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum GestureDirection {
    Up,
    Down,
    Left,
    Right,
    Error,
    #[default]
    NoGesture,
}

impl TryFrom<pros_sys::optical_direction_e_t> for GestureDirection {
    type Error = OpticalError;

    fn try_from(value: pros_sys::optical_direction_e_t) -> Result<GestureDirection, OpticalError> {
        bail_on!(pros_sys::E_OPTICAL_DIRECTION_ERROR, value);

        Ok(match value {
            pros_sys::E_OPTICAL_DIRECTION_UP => Self::Up,
            pros_sys::E_OPTICAL_DIRECTION_DOWN => Self::Down,
            pros_sys::E_OPTICAL_DIRECTION_LEFT => Self::Left,
            pros_sys::E_OPTICAL_DIRECTION_RIGHT => Self::Right,
            pros_sys::E_OPTICAL_DIRECTION_NO_GESTURE => Self::NoGesture,
            _ => unreachable!("Encountered unknown gesture direction code."),
        })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct GestureRaw {
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8,
    pub gesture_type: u8,
    pub count: u16,
    pub time: u32,
}

impl TryFrom<pros_sys::optical_gesture_s_t> for GestureRaw {
    type Error = OpticalError;

    fn try_from(value: pros_sys::optical_gesture_s_t) -> Result<GestureRaw, OpticalError> {
        Ok(Self {
            up: bail_on!(OPT_GESTURE_ERR as u8, value.udata),
            down: value.ddata,
            left: value.ldata,
            right: value.rdata,
            gesture_type: value.r#type,
            count: value.count,
            time: value.time,
        })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Rgbc {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub brightness: f64,
}

impl TryFrom<pros_sys::optical_rgb_s_t> for Rgbc {
    type Error = OpticalError;

    fn try_from(value: pros_sys::optical_rgb_s_t) -> Result<Rgbc, OpticalError> {
        Ok(Self {
            red: bail_on!(PROS_ERR_F, value.red), // Docs incorrectly claim this is PROS_ERR
            green: value.green,
            blue: value.blue,
            brightness: value.brightness,
        })
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct RgbcRaw {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub clear: u32,
}

impl TryFrom<pros_sys::optical_raw_s_t> for RgbcRaw {
    type Error = OpticalError;

    fn try_from(value: pros_sys::optical_raw_s_t) -> Result<RgbcRaw, OpticalError> {
        Ok(Self {
            clear: bail_on!(PROS_ERR_F as u32, value.clear),
            red: value.red,
            green: value.green,
            blue: value.blue,
        })
    }
}

#[derive(Debug, Snafu)]
pub enum OpticalError {
    #[snafu(display("LED PWM value must be between 0 and 100."))]
    InvalidLedPwm,

    #[snafu(display("Integration time must be between 3 and 712 milliseconds. See https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9 for more information."))]
    InvalidIntegrationTime,

    #[snafu(display("Gesture detection is not enabled for this sensor."))]
    GestureDetectionDisabled,

    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    OpticalError {} inherit PortError;
}
