//! Optical Sensor
//!
//! This module provides an interface to interact with the V5 Optical Sensor, which combines ambient
//! light sensing, color detection, proximity measurement, and gesture recognition capabilities.
//!
//! # Hardware Overview
//!
//! The optical sensor provides multi-modal optical sensing with an integrated white LED for
//! low-light operation.
//!
//! ## Color Detection
//!
//! Color data reported as RGB, HSV, and grayscale data, with optimal performance at distances under
//! 100mm. The proximity sensing uses reflected light intensity, making readings dependent on both
//! ambient lighting and target reflectivity.
//!
//! ## Gesture Detection
//!
//! The optical sensor can detect four distinct motions (up, down, left, right) of objects passing
//! over the sensor.

use core::time::Duration;

use vex_sdk::{
    V5_DeviceOpticalGesture, V5_DeviceOpticalRaw, V5_DeviceOpticalRgb, V5_DeviceT,
    vexDeviceOpticalBrightnessGet, vexDeviceOpticalGestureEnable, vexDeviceOpticalGestureGet,
    vexDeviceOpticalHueGet, vexDeviceOpticalIntegrationTimeGet, vexDeviceOpticalIntegrationTimeSet,
    vexDeviceOpticalLedPwmGet, vexDeviceOpticalLedPwmSet, vexDeviceOpticalProximityGet,
    vexDeviceOpticalRawGet, vexDeviceOpticalRgbGet, vexDeviceOpticalSatGet,
    vexDeviceOpticalStatusGet,
};
use vexide_core::time::LowResolutionTime;

use super::{PortError, SmartDevice, SmartDeviceType, SmartPort};

/// An optical sensor plugged into a Smart Port.
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

    /// The interval that gesture detection through [`OpticalSensor::last_gesture`] provides new
    /// data at.
    pub const GESTURE_UPDATE_INTERVAL: Duration = Duration::from_millis(50);

    /// Creates a new optical sensor from a [`SmartPort`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = OpticalSensor::new(peripherals.port_1);
    /// }
    /// ```
    #[must_use]
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
        }
    }

    /// Returns the detected color's hue.
    ///
    /// Hue has a range of `0` to `359.999`.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     if let Ok(hue) = sensor.hue() {
    ///         println!("Detected color hue: {:.1}°", hue);
    ///
    ///         // Classify the color based on hue angle
    ///         let color = match hue as u32 {
    ///             0..=30 => "Red",
    ///             31..=90 => "Yellow",
    ///             91..=150 => "Green",
    ///             151..=210 => "Cyan",
    ///             211..=270 => "Blue",
    ///             271..=330 => "Magenta",
    ///             _ => "Red", // 331-359 wraps back to red
    ///         };
    ///
    ///         println!("Color: {}", color);
    ///     }
    /// }
    /// ```
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
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     if let Ok(saturation) = sensor.saturation() {
    ///         println!("Color saturation: {}%", saturation * 100.0);
    ///
    ///         // Check if color is muted or vibrant
    ///         if saturation < 0.5 {
    ///             println!("Muted color detected");
    ///         } else {
    ///             println!("Vibrant color detected");
    ///         }
    ///     }
    /// }
    /// ```
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
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     if let Ok(brightness) = sensor.brightness() {
    ///         println!("Color brightness: {}%", brightness * 100.0);
    ///
    ///         // Check if color is dark or bright
    ///         if brightness < 0.3 {
    ///             println!("Dark color detected");
    ///         } else if brightness > 0.7 {
    ///             println!("Bright color detected");
    ///         } else {
    ///             println!("Medium brightness color detected");
    ///         }
    ///     }
    /// }
    /// ```
    pub fn brightness(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalBrightnessGet(self.device) })
    }

    /// Returns an analog proximity value from `0` to `1.0`.
    ///
    /// A reading of 1.0 indicates that the object is close to the sensor, while 0.0 indicates that
    /// no object is detected in range of the sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     // Monitor proximity with thresholds
    ///     if let Ok(prox) = sensor.proximity() {
    ///         match prox {
    ///             x if x > 0.8 => println!("Object very close!"),
    ///             x if x > 0.5 => println!("Object nearby"),
    ///             x if x > 0.2 => println!("Object detected"),
    ///             _ => println!("No object in range"),
    ///         }
    ///     }
    /// }
    /// ```
    pub fn proximity(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        Ok(f64::from(unsafe { vexDeviceOpticalProximityGet(self.device) }) / 255.0)
    }

    /// Returns the processed RGB color data from the sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     // Color detection with RGB values
    ///     if let Ok(rgb) = sensor.color() {
    ///         println!(
    ///             "Color reading: R={}, G={}, B={}",
    ///             rgb.red, rgb.green, rgb.blue
    ///         );
    ///
    ///         // Example: Check if object is primarily red
    ///         // Note that you should probably use `OpticalSensor::hue` instead for this.
    ///         if rgb.red > rgb.green && rgb.red > rgb.blue {
    ///             println!("Object is primarily red!");
    ///         }
    ///     }
    /// }
    /// ```
    pub fn color(&self) -> Result<OpticalRgb, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceOpticalRgb::default();
        unsafe { vexDeviceOpticalRgbGet(self.device, &raw mut data) };

        Ok(data.into())
    }

    /// Returns the raw, unprocessed RGBC color data from the sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    pub fn raw_color(&self) -> Result<OpticalRaw, PortError> {
        self.validate_port()?;

        let mut data = V5_DeviceOpticalRaw::default();
        unsafe { vexDeviceOpticalRawGet(self.device, &raw mut data) };

        Ok(data.into())
    }

    /// Returns the most recent gesture data from the sensor, or `None` if no gesture was detected.
    ///
    /// Gesture data updates every 500 milliseconds.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     // Print the details of the last detected gesture.
    ///     loop {
    ///         if let Ok(Some(gesture)) = sensor.last_gesture() {
    ///             println!("Direction: {:?}", gesture.direction);
    ///         }
    ///
    ///         sleep(Duration::from_millis(25)).await;
    ///     }
    /// }
    /// ```
    pub fn last_gesture(&self) -> Result<Option<Gesture>, PortError> {
        self.validate_port()?;

        // Enable gesture detection if not already enabled.
        //
        // For some reason, PROS docs claim that this function makes color reading
        // unavailable, but from hardware testing this is false.
        unsafe { vexDeviceOpticalGestureEnable(self.device) };

        let mut gesture = V5_DeviceOpticalGesture::default();
        let direction = match unsafe { vexDeviceOpticalGestureGet(self.device, &raw mut gesture) } {
            // see: https://github.com/purduesigbots/pros/blob/master/include/pros/optical.h#L37
            1 => GestureDirection::Up,
            2 => GestureDirection::Down,
            3 => GestureDirection::Left,
            4 => GestureDirection::Right,

            // This is just a zero return usually if no gesture was detected.
            _ => return Ok(None),
        };

        Ok(Some(Gesture {
            direction,
            up: gesture.udata,
            down: gesture.ddata,
            left: gesture.ldata,
            right: gesture.rdata,
            gesture_type: gesture.gesture_type,
            count: gesture.count,
            time: LowResolutionTime::from_millis_since_epoch(gesture.time),
        }))
    }

    /// Returns the intensity/brightness of the sensor's LED indicator as a number from [0.0-1.0].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     if let Ok(brightness) = sensor.led_brightness() {
    ///         println!("LED brightness: {:.1}%", brightness * 100.0);
    ///     }
    /// }
    /// ```
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
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     // Blink LED 3 times
    ///     for _ in 0..3 {
    ///         // Turn LED on
    ///         if let Err(e) = sensor.set_led_brightness(1.0) {
    ///             println!("Failed to turn LED on: {:?}", e);
    ///         }
    ///
    ///         sleep(Duration::from_millis(250)).await;
    ///
    ///         // Turn LED off
    ///         if let Err(e) = sensor.set_led_brightness(0.0) {
    ///             println!("Failed to turn LED off: {:?}", e);
    ///         }
    ///
    ///         sleep(Duration::from_millis(250)).await;
    ///     }
    /// }
    /// ```
    pub fn set_led_brightness(&mut self, brightness: f64) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe { vexDeviceOpticalLedPwmSet(self.device, (brightness * 100.0) as i32) }

        Ok(())
    }

    /// Returns integration time of the optical sensor in milliseconds, with minimum time being 3ms
    /// and the maximum time being 712ms.
    ///
    /// The default integration time for the sensor is 103mS, unless otherwise set with
    /// [`OpticalSensor::set_integration_time`].
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     // Set integration time to 50 milliseconds.
    ///     _ = sensor.set_integration_time(Duration::from_millis(50));
    ///
    ///     // Log out the new integration time.
    ///     if let Ok(time) = sensor.integration_time() {
    ///         println!("Integration time: {:?}", time);
    ///     }
    /// }
    /// ```
    pub fn integration_time(&self) -> Result<Duration, PortError> {
        self.validate_port()?;

        Ok(Duration::from_millis(
            unsafe { vexDeviceOpticalIntegrationTimeGet(self.device) } as u64,
        ))
    }

    /// Set the integration time of the optical sensor.
    ///
    /// Lower integration time results in faster update rates with lower accuracy due to less
    /// available light being read by the sensor.
    ///
    /// The `time` value must be a [`Duration`] between 3 and 712 milliseconds. If the integration
    /// time is out of this range, it will be clamped to fit inside it. See
    /// <https://www.vexforum.com/t/v5-optical-sensor-refresh-rate/109632/9> for more information.
    ///
    /// The default integration time for the sensor is 103mS, unless otherwise set.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     // Set integration time to 50 milliseconds.
    ///     _ = sensor.set_integration_time(Duration::from_millis(50));
    /// }
    /// ```
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

    /// Returns the internal status code of the optical sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError::Disconnected`] error is returned if no device was connected to the port.
    /// - A [`PortError::IncorrectDevice`] error is returned if the wrong type of device was
    ///   connected to the port.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = OpticalSensor::new(peripherals.port_1);
    ///
    ///     if let Ok(status) = sensor.status() {
    ///         println!("Status: {:b}", status);
    ///     }
    /// }
    /// ```
    pub fn status(&self) -> Result<u32, PortError> {
        self.validate_port()?;

        Ok(unsafe { vexDeviceOpticalStatusGet(self.device) })
    }
}

impl SmartDevice for OpticalSensor {
    const UPDATE_INTERVAL: Duration = Duration::from_millis(20);

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
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GestureDirection {
    /// Up gesture.
    Up = 1,
    /// Down gesture.
    Down = 2,
    /// Left gesture.
    Left = 3,
    /// Right gesture.
    Right = 4,
}

/// Gesture data from an [`OpticalSensor`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    pub time: LowResolutionTime,
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
