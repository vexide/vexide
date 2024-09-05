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

/// Represents a smart port configured as a V5 optical sensor
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

    /// Creates a new optical sensor from a smart port index.
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
        }
    }

    /// Get the PWM percentage (intensity/brightness) of the sensor's LED indicator.
    pub fn led_brightness(&self) -> Result<i32, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalLedPwmGet(self.device) })
    }

    /// Set the PWM percentage (intensity/brightness) of the sensor's LED indicator.
    pub fn set_led_brightness(&mut self, brightness: f64) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe { vexDeviceOpticalLedPwmSet(self.device, (brightness * 100.0) as i32) }

        Ok(())
    }

    /// Get integration time (update rate) of the optical sensor in milliseconds, with
    /// minimum time being 3ms and the maximum time being 712ms.
    pub fn integration_time(&self) -> Result<Duration, PortError> {
        self.validate_port()?;

        Ok(Duration::from_millis(
            unsafe { vexDeviceOpticalIntegrationTimeGet(self.device) } as u64,
        ))
    }

    /// Set integration time (update rate) of the optical sensor.
    ///
    /// Lower integration time results in faster update rates with lower accuracy
    /// due to less available light being read by the sensor.
    ///
    /// Time value must be a [`Duration`] between 3 and 712 milliseconds. See
    /// <https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9> for
    /// more information.
    pub fn set_integration_time(&mut self, time: Duration) -> Result<(), PortError> {
        self.validate_port()?;

        let time_ms = time.as_millis().clamp(
            Self::MIN_INTEGRATION_TIME.as_millis(),
            Self::MAX_INTEGRATION_TIME.as_millis(),
        ) as f64;

        unsafe { vexDeviceOpticalIntegrationTimeSet(self.device, time_ms) }

        Ok(())
    }

    /// Get the detected color hue.
    ///
    /// Hue has a range of `0` to `359.999`.
    pub fn hue(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalHueGet(self.device) })
    }

    /// Gets the detected color saturation.
    ///
    /// Saturation has a range `0` to `1.0`.
    pub fn saturation(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalSatGet(self.device) })
    }

    /// Get the detected color brightness.
    ///
    /// Brightness values range from `0` to `1.0`.
    pub fn brightness(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalBrightnessGet(self.device) })
    }

    /// Get the analog proximity value from `0` to `1.0`.
    ///
    /// A reading of 1.0 indicates that the object is close to the sensor, while 0.0
    /// indicates that no object is detected in range of the sensor.
    pub fn proximity(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalProximityGet(self.device) } as f64 / 255.0)
    }

    /// Get the processed RGB data from the sensor
    pub fn rgb(&self) -> Result<OpticalRgb, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceOpticalRgb::default();
        unsafe { vexDeviceOpticalRgbGet(self.device, &mut data) };

        Ok(data.into())
    }

    /// Get the raw, unprocessed RGBC data from the sensor
    pub fn raw(&self) -> Result<OpticalRaw, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceOpticalRaw::default();
        unsafe { vexDeviceOpticalRawGet(self.device, &mut data) };

        Ok(data.into())
    }

    /// Get the most recent gesture data from the sensor. Gestures will be cleared after 500mS.
    pub fn last_gesture(&self) -> Result<Gesture, PortError> {
        self.validate_port()?;

        // Enable gesture detection if not already enabled.
        //
        // For some reason, PROS docs claim that this function makes color reading
        // unavilable, but from hardware testing this is false.
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

    /// Gets the status code of the distance sensor
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
            // all values even if they're unreacahable.
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
