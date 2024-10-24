//! Optical sensor device

use core::time::Duration;

use vex_sdk::{
    vexDeviceOpticalBrightnessGet, vexDeviceOpticalGestureEnable, vexDeviceOpticalGestureGet,
    vexDeviceOpticalHueGet, vexDeviceOpticalIntegrationTimeGet, vexDeviceOpticalIntegrationTimeSet,
    vexDeviceOpticalLedPwmGet, vexDeviceOpticalLedPwmSet, vexDeviceOpticalProximityGet,
    vexDeviceOpticalRawGet, vexDeviceOpticalRgbGet, vexDeviceOpticalSatGet,
    vexDeviceOpticalStatusGet, V5_DeviceOpticalGesture, V5_DeviceOpticalRaw, V5_DeviceOpticalRgb,
    V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

/// An optical sensor plugged into a smart port.
#[derive(Debug, Eq, PartialEq)]
pub struct OpticalSensor {
    port: SmartPort,
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for OpticalSensor {}
unsafe impl Sync for OpticalSensor {}

impl OpticalSensor {
    /// The smallest integration time you can set on an optical sensor.
    ///
    /// Source: <https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9>
    pub const MIN_INTEGRATION_TIME: Duration = Duration::from_millis(3);

    /// The largest integration time you can set on an optical sensor.
    ///
    /// Source: <https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9>
    pub const MAX_INTEGRATION_TIME: Duration = Duration::from_millis(712);

    /// Creates a new optical sensor from a [`SmartPort`].
    #[must_use]
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
        }
    }

    /// Returns the intensity/brightness of the sensor's LED indicator as a number from [0.0-1.0].
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn led_brightness(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(f64::from(unsafe { vexDeviceOpticalLedPwmGet(self.device) }) / 100.0)
    }

    /// Set the intensity of (intensity/brightness) of the sensor's LED indicator.
    ///
    /// Intensity is expressed as a number from [0.0, 1.0].
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn set_led_brightness(&mut self, brightness: f64) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe { vexDeviceOpticalLedPwmSet(self.device, (brightness * 100.0) as i32) }

        Ok(())
    }

    /// Returns integration time of the optical sensor in milliseconds, with
    /// minimum time being 3ms and the maximum time being 712ms.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn integration_time(&self) -> Result<Duration, PortError> {
        self.validate_port()?;

        Ok(Duration::from_millis(
            unsafe { vexDeviceOpticalIntegrationTimeGet(self.device) } as u64,
        ))
    }

    /// Set the integration time of the optical sensor.
    ///
    /// Lower integration time results in faster update rates with lower accuracy
    /// due to less available light being read by the sensor.
    ///
    /// The `time` value must be a [`Duration`] between 3 and 712 milliseconds. If
    /// the integration time is out of this range, it will be clamped to fit inside it. See
    /// <https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9> for
    /// more information.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn set_integration_time(&mut self, time: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        // `time_ms` is clamped to a range that will not cause precision loss.
        #[allow(clippy::cast_precision_loss)]
        let time_ms = time.as_millis().clamp(
            Self::MIN_INTEGRATION_TIME.as_millis(),
            Self::MAX_INTEGRATION_TIME.as_millis(),
        ) as f64;

        unsafe { vexDeviceOpticalIntegrationTimeSet(self.device, time_ms) }

        Ok(())
    }

    /// Returns the detected color's hue.
    ///
    /// Hue has a range of `0` to `359.999`.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn hue(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalHueGet(self.device) })
    }

    /// Returns the detected color's saturation.
    ///
    /// Saturation has a range `0` to `1.0`.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn saturation(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalSatGet(self.device) })
    }

    /// Returns the detected color's brightness.
    ///
    /// Brightness values range from `0` to `1.0`.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn brightness(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalBrightnessGet(self.device) })
    }

    /// Returns an analog proximity value from `0` to `1.0`.
    ///
    /// A reading of 1.0 indicates that the object is close to the sensor, while 0.0
    /// indicates that no object is detected in range of the sensor.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn proximity(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(f64::from(unsafe { vexDeviceOpticalProximityGet(self.device) }) / 255.0)
    }

    /// Returns the processed RGB color data from the sensor.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn color(&self) -> Result<OpticalRgb, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceOpticalRgb::default();
        unsafe { vexDeviceOpticalRgbGet(self.device, &mut data) };

        Ok(data.into())
    }

    /// Returns the raw, unprocessed RGBC color data from the sensor.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn raw_color(&self) -> Result<OpticalRaw, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceOpticalRaw::default();
        unsafe { vexDeviceOpticalRawGet(self.device, &mut data) };

        Ok(data.into())
    }

    /// Returns the most recent gesture data from the sensor.
    ///
    /// Gestures will be cleared after 500 milliseconds.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn last_gesture(&self) -> Result<Gesture, PortError> {
        self.validate_port()?;

        // Enable gesture detection if not already enabled.
        //
        // For some reason, PROS docs claim that this function makes color reading
        // unavailable, but from hardware testing this is false.
        unsafe { vexDeviceOpticalGestureEnable(self.device) };

        let mut gesture = V5_DeviceOpticalGesture::default();
        let direction: GestureDirection =
            unsafe { vexDeviceOpticalGestureGet(self.device, &mut gesture) }.into();

        Ok(Gesture {
            direction,
            up: gesture.udata,
            down: gesture.ddata,
            left: gesture.ldata,
            right: gesture.rdata,
            gesture_type: gesture.gesture_type,
            count: gesture.count,
            time: gesture.time,
        })
    }

    /// Returns the internal status code of the optical sensor.
    ///
    /// # Errors
    ///
    /// An error is returned if an optical sensor is not currently connected to the smart port.
    pub fn status(&self) -> Result<u32, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalStatusGet(self.device) })
    }
}

impl SmartDevice for OpticalSensor {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Optical
    }
}
impl From<OpticalSensor> for SmartPort {
    fn from(device: OpticalSensor) -> Self {
        device.port
    }
}

/// Represents a gesture and its direction.
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum GestureDirection {
    /// No gesture detected.
    #[default]
    None = 0,
    /// Up gesture.
    Up = 1,
    /// Down gesture.
    Down = 2,
    /// Left gesture.
    Left = 3,
    /// Right gesture.
    Right = 4,
}

impl From<u32> for GestureDirection {
    fn from(code: u32) -> Self {
        // https://github.com/purduesigbots/pros/blob/master/include/pros/optical.h#L37
        match code {
            //
            1 => Self::Up,
            2 => Self::Down,
            3 => Self::Left,
            4 => Self::Right,
            // Normally this is just 0, but this is `From` so we have to handle
            // all values even if they're unreachable.
            _ => Self::None,
        }
    }
}

/// Gesture data from an [`OpticalSensor`].
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Gesture {
    /// Gesture Direction
    pub direction: GestureDirection,
    /// Up value.
    pub up: u8,
    /// Down value.
    pub down: u8,
    /// Left value.
    pub left: u8,
    /// Right value.
    pub right: u8,
    /// Gesture type.
    pub gesture_type: u8,
    /// The count of the gesture.
    pub count: u16,
    /// The time of the gesture.
    pub time: u32,
}

/// RGB data from a [`OpticalSensor`].
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct OpticalRgb {
    /// The red value from the sensor.
    pub red: f64,
    /// The green value from the sensor.
    pub green: f64,
    /// The blue value from the sensor.
    pub blue: f64,
    /// The brightness value from the sensor.
    pub brightness: f64,
}

impl From<V5_DeviceOpticalRgb> for OpticalRgb {
    fn from(value: V5_DeviceOpticalRgb) -> Self {
        Self {
            red: value.red,
            green: value.green,
            blue: value.blue,
            brightness: value.brightness,
        }
    }
}

/// Represents the raw RGBC data from the sensor.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpticalRaw {
    /// The red value from the sensor.
    pub red: u16,
    /// The green value from the sensor.
    pub green: u16,
    /// The blue value from the sensor.
    pub blue: u16,
    /// The clear value from the sensor.
    pub clear: u16,
}

impl From<V5_DeviceOpticalRaw> for OpticalRaw {
    fn from(value: V5_DeviceOpticalRaw) -> Self {
        Self {
            red: value.red,
            green: value.green,
            blue: value.blue,
            clear: value.clear,
        }
    }
}
