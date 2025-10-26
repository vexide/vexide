//! Vision Sensor
//!
//! This module provides an interface for interacting with the VEX Vision Sensor.
//!
//! # Hardware Overview
//!
//! The VEX Vision Sensor is a device powered by an ARM Cortex M4 and Cortex M0 coprocessor with a
//! color camera for the purpose of performing object recognition. The sensor can be trained to
//! locate objects by color. The camera module itself is very similar internally to the Pixy2
//! camera, and performs its own onboard image processing. Manually processing raw image data from
//! the sensor is not currently possible.
//!
//! Every 20 milliseconds, the camera provides a list of the objects found matching up to seven
//! unique [`VisionSignature`]s. The object’s height, width, and location is provided. Multi-colored
//! objects may also be programmed through the use of [`VisionCode`]s.
//!
//! The Vision Sensor has USB for a direct connection to a computer, where it can be configured
//! using VEX's proprietary vision utility tool to generate color signatures. The Vision Sensor also
//! has Wi-Fi Direct and can act as web server, allowing a live video feed of the camera from any
//! computer equipped with a browser and Wi-Fi.

extern crate alloc;

use alloc::vec::Vec;
use core::time::Duration;

use snafu::{Snafu, ensure};
use vex_sdk::{
    V5_DeviceT, V5_DeviceVisionObject, V5_DeviceVisionRgb, V5_DeviceVisionSignature,
    V5VisionBlockType, V5VisionLedMode, V5VisionMode, V5VisionWBMode, V5VisionWifiMode,
    vexDeviceVisionBrightnessGet, vexDeviceVisionBrightnessSet, vexDeviceVisionLedColorGet,
    vexDeviceVisionLedColorSet, vexDeviceVisionLedModeGet, vexDeviceVisionLedModeSet,
    vexDeviceVisionModeGet, vexDeviceVisionModeSet, vexDeviceVisionObjectCountGet,
    vexDeviceVisionObjectGet, vexDeviceVisionSignatureGet, vexDeviceVisionSignatureSet,
    vexDeviceVisionWhiteBalanceGet, vexDeviceVisionWhiteBalanceModeGet,
    vexDeviceVisionWhiteBalanceModeSet, vexDeviceVisionWhiteBalanceSet, vexDeviceVisionWifiModeGet,
    vexDeviceVisionWifiModeSet,
};

use super::{PortError, SmartDevice, SmartDeviceType, SmartPort};
use crate::{
    color::Color,
    math::{Angle, Point2},
};

/// VEX Vision Sensor
///
/// This struct represents a vision sensor plugged into a Smart Port.
#[derive(Debug, Eq, PartialEq)]
pub struct VisionSensor {
    port: SmartPort,
    codes: Vec<VisionCode>,
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for VisionSensor {}
unsafe impl Sync for VisionSensor {}

impl VisionSensor {
    /// The horizontal resolution of the vision sensor.
    ///
    /// This value is based on the `VISION_FOV_WIDTH` macro constant in PROS.
    pub const HORIZONTAL_RESOLUTION: u16 = 316;

    /// The vertical resolution of the vision sensor.
    ///
    /// This value is based on the `VISION_FOV_HEIGHT` macro constant in PROS.
    pub const VERTICAL_RESOLUTION: u16 = 212;

    /// The horizontal FOV of the vision sensor in degrees.
    pub const HORIZONTAL_FOV: f32 = 64.6;

    /// The vertical FOV of the vision sensor in degrees.
    pub const VERTICAL_FOV: f32 = 46.0;

    /// The diagonal FOV of the vision sensor in degrees.
    pub const DIAGONAL_FOV: f32 = 78.0;

    /// The update rate of the vision sensor.
    pub const UPDATE_INTERVAL: Duration = Duration::from_millis(50);

    /// Creates a new vision sensor from a [`SmartPort`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let sensor = VisionSensor::new(peripherals.port_1);
    /// }
    /// ```
    #[must_use]
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
            codes: Vec::new(),
        }
    }

    /// Adds a detection signature to the sensor's onboard memory.
    ///
    /// This signature will be used to identify objects when using [`VisionSensor::objects`].
    ///
    /// The sensor can store up to 7 unique signatures, with each signature slot denoted by the `id`
    /// parameter. If a signature with an ID matching an existing signature on the sensor is added,
    /// then the existing signature will be overwritten with the new one.
    ///
    /// # Volatile Memory
    ///
    /// The memory on the Vision Sensor is *volatile* and will therefore be wiped when the sensor
    /// loses power. As a result, this function should be called every time the sensor is used on
    /// program start.
    ///
    /// # Panics
    ///
    /// - Panics if the given signature ID is not in the interval [1, 7].
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
    /// use vexide::{prelude::*, smart::vision::VisionSignature};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // These signatures can be generated using VEX's vision utility.
    ///     let example_signature = VisionSignature::new((10049, 11513, 10781), (-425, 1, -212), 4.1);
    ///
    ///     // Set signature 1 one the sensor.
    ///     _ = sensor.set_signature(1, example_signature);
    /// }
    /// ```
    pub fn set_signature(&mut self, id: u8, signature: VisionSignature) -> Result<(), PortError> {
        assert!(
            (1..=7).contains(&id),
            "The given signature ID `{id}` is not in the expected interval [1, 7]."
        );
        self.validate_port()?;

        let mut signature = V5_DeviceVisionSignature {
            id,
            uMin: signature.u_threshold.0,
            uMean: signature.u_threshold.1,
            uMax: signature.u_threshold.2,
            vMin: signature.v_threshold.0,
            vMean: signature.v_threshold.1,
            vMax: signature.v_threshold.2,
            range: signature.range,
            mType: u32::from(
                if self.codes.iter().any(|code| code.contains_signature(id)) {
                    V5VisionBlockType::kVisionTypeColorCode
                } else {
                    V5VisionBlockType::kVisionTypeNormal
                }
                .0,
            ),
            ..Default::default()
        };

        unsafe { vexDeviceVisionSignatureSet(self.device, &raw mut signature) }

        Ok(())
    }

    /// Reads a signature off the sensor's onboard memory, returning `Some(sig)` if the slot is
    /// filled or `None` if no signature is stored with the given ID.
    fn read_raw_signature(
        &self,
        id: u8,
    ) -> Result<Option<V5_DeviceVisionSignature>, VisionSignatureError> {
        assert!(
            (1..=7).contains(&id),
            "The given signature ID `{id}` is not in the expected interval [1, 7]."
        );

        let mut raw_signature = V5_DeviceVisionSignature::default();
        let read_operation = unsafe {
            vexDeviceVisionSignatureGet(self.device, u32::from(id), &raw mut raw_signature)
        };

        if !read_operation {
            return Ok(None);
        }

        // pad[0] is actually an undocumented flags field on `V5_DeviceVisionSignature`. If the
        // sensor returns no flags, then it has failed to send data back.
        //
        // TODO: Make sure this is correct and not the PROS docs being wrong here.
        //
        // We also check that the read operation succeeded from the return of
        // `vexDeviceVisionSignatureGet`.
        ensure!(raw_signature.pad[0] != 0, ReadingFailedSnafu);

        Ok(Some(raw_signature))
    }

    /// Adjusts the type of a signature stored on the sensor.
    ///
    /// This is used when assigning certain stored signatures as belonging to color codes.
    fn write_signature_type(&mut self, id: u8, sig_type: u32) -> Result<(), VisionSignatureError> {
        if let Some(mut sig) = self.read_raw_signature(id)? {
            sig.mType = sig_type;
            unsafe { vexDeviceVisionSignatureSet(self.device, &raw mut sig) }
        } else {
            return ReadingFailedSnafu.fail();
        }

        Ok(())
    }

    /// Returns a signature from the sensor's onboard volatile memory.
    ///
    /// # Panics
    ///
    /// - Panics if the given signature ID is not in the interval [1, 7].
    ///
    /// # Errors
    ///
    /// - A [`VisionSignatureError::Port`] error is returned if there was not a sensor connected to
    ///   the port.
    /// - A [`VisionSignatureError::ReadingFailed`] error is returned if a read operation failed or
    ///   there was no signature previously set in the slot(s) specified in the [`VisionCode`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::{prelude::*, smart::vision::VisionSignature};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Set an example signature in the sensor's first slot.
    ///     _ = sensor.set_signature(
    ///         1,
    ///         VisionSignature::new((10049, 11513, 10781), (-425, 1, -212), 4.1),
    ///     );
    ///
    ///     // Read signature 1 off the sensor.
    ///     // This should be the same as the one we just set.
    ///     if let Ok(Some(sig)) = sensor.signature(1) {
    ///         println!("{:?}", sig);
    ///     }
    /// }
    /// ```
    pub fn signature(&self, id: u8) -> Result<Option<VisionSignature>, VisionSignatureError> {
        self.validate_port()?;

        Ok(self.read_raw_signature(id)?.map(Into::into))
    }

    /// Returns all signatures currently stored on the sensor's onboard volatile memory.
    ///
    /// # Errors
    ///
    /// - A [`VisionSignatureError::Port`] error is returned if there was not a sensor connected to
    ///   the port.
    /// - A [`VisionSignatureError::ReadingFailed`] error is returned if a read operation failed or
    ///   there was no signature previously set in the slot(s) specified in the [`VisionCode`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::{prelude::*, smart::vision::VisionSignature};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // A bunch of random color signatures.
    ///     let sig_1 = VisionSignature::new((10049, 11513, 10781), (-425, 1, -212), 4.1);
    ///     let sig_2 = VisionSignature::new((8973, 11143, 10058), (-2119, -1053, -1586), 5.4);
    ///     let sig_3 = VisionSignature::new((-3665, -2917, -3292), (4135, 10193, 7164), 2.0);
    ///     let sig_4 = VisionSignature::new((-5845, -4809, -5328), (-5495, -4151, -4822), 3.1);
    ///
    ///     // Set signatures 1-4.
    ///     _ = sensor.set_signature(1, sig_1);
    ///     _ = sensor.set_signature(2, sig_2);
    ///     _ = sensor.set_signature(3, sig_3);
    ///     _ = sensor.set_signature(4, sig_4);
    ///
    ///     // Read back the signatures from the sensor's memory.
    ///     // These should be the signatures that we just set.
    ///     if let Ok(signatures) = sensor.signatures() {
    ///         for sig in signatures.into_iter().flatten() {
    ///             println!("Found sig saved on sensor: {:?}", sig);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn signatures(&self) -> Result<[Option<VisionSignature>; 7], VisionSignatureError> {
        Ok([
            self.signature(1)?,
            self.signature(2)?,
            self.signature(3)?,
            self.signature(4)?,
            self.signature(5)?,
            self.signature(6)?,
            self.signature(7)?,
        ])
    }

    /// Registers a color code to the sensor's onboard memory. This code will be used to identify
    /// objects when using [`VisionSensor::objects`].
    ///
    /// Color codes are effectively "signature groups" that the sensor will use to identify objects
    /// containing the color of their signatures next to each other.
    ///
    /// # Volatile Memory
    ///
    /// The onboard memory of the Vision Sensor is *volatile* and will therefore be wiped when the
    /// sensor loses its power source. As a result, this function should be called every time the
    /// sensor is used on program start.
    ///
    /// # Panics
    ///
    /// - Panics if one or more of the given signature IDs are not in the interval [1, 7].
    ///
    /// # Errors
    ///
    /// - A [`VisionSignatureError::Port`] error is returned if a vision sensor is not currently
    ///   connected to the Smart Port.
    /// - A [`VisionSignatureError::ReadingFailed`] error is returned if a read operation failed or
    ///   there was no signature previously set in the slot(s) specified in the [`VisionCode`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::prelude::*;
    /// use vexide::smart::vision::{VisionSignature, VisionCode, DetectionSource};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Two color signatures.
    ///     let sig_1 = VisionSignature::new((10049, 11513, 10781), (-425, 1, -212), 4.1);
    ///     let sig_2 = VisionSignature::new((8973, 11143, 10058), (-2119, -1053, -1586), 5.4);
    ///
    ///     // Store the signatures on the sensor.
    ///     _ = sensor.set_signature(1, sig_1);
    ///     _ = sensor.set_signature(2, sig_2);
    ///
    ///     // Create a code associating signatures 1 and 2 together.
    ///     let code = VisionCode::from((1, 2));
    ///
    ///     // Register our code on the sensor. When we call [`VisionSensor::objects`], the associated
    ///     // signatures will be returned as a single object if their colors are detected next to each other.
    ///     _ = sensor.add_code(code);
    ///
    ///     // Scan for objects. Filter only objects matching the code we just set.
    ///     if let Ok(objects) = sensor.objects() {
    ///         for object in objects.iter().filter(|obj| obj.source == DetectionSource::Code(code)) {
    ///             println!("{:?}", object);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn add_code(&mut self, code: impl Into<VisionCode>) -> Result<(), VisionSignatureError> {
        self.validate_port()?;

        let code = code.into();

        self.write_signature_type(code.0, u32::from(V5VisionBlockType::kVisionTypeColorCode.0))?;
        self.write_signature_type(code.1, u32::from(V5VisionBlockType::kVisionTypeColorCode.0))?;
        if let Some(sig_3) = code.2 {
            self.write_signature_type(sig_3, u32::from(V5VisionBlockType::kVisionTypeColorCode.0))?;
        }
        if let Some(sig_4) = code.3 {
            self.write_signature_type(sig_4, u32::from(V5VisionBlockType::kVisionTypeColorCode.0))?;
        }
        if let Some(sig_5) = code.4 {
            self.write_signature_type(sig_5, u32::from(V5VisionBlockType::kVisionTypeColorCode.0))?;
        }

        self.codes.push(code);

        Ok(())
    }

    /// Returns the user-set behavior of the LED indicator on the sensor.
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
    /// use vexide::{color::Rgb, prelude::*, smart::vision::LedMode};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Set the LED to red at 100% brightness.
    ///     _ = sensor.set_led_mode(LedMode::Manual(Rgb { r: 255, g: 0, b: 0 }, 1.0));
    ///
    ///     // Give the sensor time to update.
    ///     sleep(VisionSensor::UPDATE_INTERVAL).await;
    ///
    ///     // Check the sensor's reported LED mode. Should be the same as what we just set
    ///     if let Ok(led_mode) = sensor.led_mode() {
    ///         assert_eq!(led_mode, LedMode::Manual(Rgb { r: 255, g: 0, b: 0 }, 1.0));
    ///     }
    /// }
    /// ```
    pub fn led_mode(&self) -> Result<LedMode, PortError> {
        self.validate_port()?;

        Ok(match unsafe { vexDeviceVisionLedModeGet(self.device) } {
            V5VisionLedMode::kVisionLedModeAuto => LedMode::Auto,
            V5VisionLedMode::kVisionLedModeManual => {
                let led_color = unsafe { vexDeviceVisionLedColorGet(self.device) };

                LedMode::Manual(
                    Color::new(led_color.red, led_color.green, led_color.blue),
                    f64::from(led_color.brightness) / 100.0,
                )
            }
            _ => unreachable!(),
        })
    }

    /// Returns a [`Vec`] of objects detected by the sensor.
    ///
    /// # Errors
    ///
    /// - A [`VisionObjectError::Port`] error is returned if there was not a sensor connected to the
    ///   port.
    /// - A [`VisionObjectError::WifiMode`] error is returned if the sensor is in Wi-Fi mode.
    /// - A [`VisionObjectError::InvalidObject`] error if the sensor failed to read an object.
    ///
    /// # Examples
    ///
    /// With one signature:
    ///
    /// ```no_run
    /// use vexide::{prelude::*, smart::vision::VisionSignature};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Set a color signature on the sensor's first slot.
    ///     _ = sensor.set_signature(
    ///         1,
    ///         VisionSignature::new((10049, 11513, 10781), (-425, 1, -212), 4.1),
    ///     );
    ///
    ///     // Scan for detected objects.
    ///     if let Ok(objects) = sensor.objects() {
    ///         for object in objects {
    ///             println!("{:?}", object);
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// With multiple signatures:
    ///
    /// ```no_run
    /// use vexide::{
    ///     prelude::*,
    ///     smart::vision::{DetectionSource, VisionSignature},
    /// };
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Two color signatures.
    ///     let sig_1 = VisionSignature::new((10049, 11513, 10781), (-425, 1, -212), 4.1);
    ///     let sig_2 = VisionSignature::new((8973, 11143, 10058), (-2119, -1053, -1586), 5.4);
    ///
    ///     // Store the signatures on the sensor.
    ///     _ = sensor.set_signature(1, sig_1);
    ///     _ = sensor.set_signature(2, sig_2);
    ///
    ///     // Scan for objects.
    ///     if let Ok(objects) = sensor.objects() {
    ///         for object in objects {
    ///             // Identify which signature the detected object matches.
    ///             match object.source {
    ///                 DetectionSource::Signature(1) => {
    ///                     println!("Detected object matching sig_1: {:?}", object)
    ///                 }
    ///                 DetectionSource::Signature(2) => {
    ///                     println!("Detected object matching sig_2: {:?}", object)
    ///                 }
    ///                 _ => {}
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn objects(&self) -> Result<Vec<VisionObject>, VisionObjectError> {
        ensure!(self.mode()? != VisionMode::Wifi, WifiModeSnafu);

        let object_count = unsafe { vexDeviceVisionObjectCountGet(self.device) } as usize;
        let mut objects = Vec::with_capacity(object_count);

        for i in 0..object_count {
            let mut object = V5_DeviceVisionObject::default();

            if unsafe { vexDeviceVisionObjectGet(self.device, i as u32, &raw mut object) } == 0 {
                return InvalidObjectSnafu.fail();
            }

            let object: VisionObject = object.into();

            match object.source {
                DetectionSource::Signature(_) | DetectionSource::Line => {
                    objects.push(object);
                }
                DetectionSource::Code(code) => {
                    if self.codes.contains(&code) {
                        objects.push(object);
                    }
                }
            }
        }

        Ok(objects)
    }

    /// Returns the number of objects detected by the sensor.
    ///
    /// # Errors
    ///
    /// - A [`VisionObjectError::Port`] error is returned if there is not a sensor connected to the
    ///   port.
    /// - A [`VisionObjectError::WifiMode`] error is returned if the sensor is in Wi-Fi mode.
    /// - A [`VisionObjectError::InvalidObject`] error if the sensor failed to read an object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vexide::{prelude::*, smart::vision::VisionSignature};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Set a color signature on the sensor's first slot.
    ///     _ = sensor.set_signature(
    ///         1,
    ///         VisionSignature::new((10049, 11513, 10781), (-425, 1, -212), 4.1),
    ///     );
    ///
    ///     loop {
    ///         if let Ok(n) = sensor.object_count() {
    ///             println!("Sensor is currently detecting {n} objects.");
    ///         }
    ///
    ///         sleep(VisionSensor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn object_count(&self) -> Result<usize, VisionObjectError> {
        // NOTE: We actually can't rely on [`vexDeviceVisionObjectCountGet`], due to the way that
        // vision codes are registered.
        //
        // When a code is registered, all this really does is set a bunch of normal signatures with
        // an additional flag set (see: [`Self::set_code_signature`]). This means that if the user
        // has multiple vision codes, we can't distinguish between which objects were detected by
        // a certain code until AFTER we get the full objects list (where we can then distinguish)
        // by [`VisionObject::source`].
        Ok(self.objects()?.len())
    }

    /// Returns the current brightness setting of the vision sensor as a percentage.
    ///
    /// The returned result should be from `0.0` (0%) to `1.0` (100%).
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
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Set brightness to 50%
    ///     _ = sensor.set_brightness(0.5);
    ///
    ///     // Give the sensor time to update.
    ///     sleep(VisionSensor::UPDATE_INTERVAL).await;
    ///
    ///     // Read brightness. Should be 50%, since we just set it.
    ///     if let Ok(brightness) = sensor.brightness() {
    ///         assert_eq!(brightness, 0.5);
    ///     }
    /// }
    /// ```
    pub fn brightness(&self) -> Result<f64, PortError> {
        self.validate_port()?;

        // SDK function gives us brightness percentage 0-100.
        Ok(f64::from(unsafe { vexDeviceVisionBrightnessGet(self.device) }) / 100.0)
    }

    /// Returns the current white balance of the vision sensor as an RGB color.
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
    /// use vexide::{color::Color, prelude::*, smart::vision::WhiteBalance};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Set white balance to manual.
    ///     _ = sensor.set_white_balance(WhiteBalance::Manual(Color::WHITE));
    ///
    ///     // Give the sensor time to update.
    ///     sleep(VisionSensor::UPDATE_INTERVAL).await;
    ///
    ///     // Read brightness. Should be 50%, since we just set it.
    ///     if let Ok(white_balance) = sensor.white_balance() {
    ///         assert_eq!(white_balance, WhiteBalance::Manual(Color::WHITE));
    ///     }
    /// }
    /// ```
    pub fn white_balance(&self) -> Result<WhiteBalance, PortError> {
        self.validate_port()?;

        Ok(
            match unsafe { vexDeviceVisionWhiteBalanceModeGet(self.device) } {
                V5VisionWBMode::kVisionWBNormal => WhiteBalance::Auto,
                V5VisionWBMode::kVisionWBStart => WhiteBalance::StartupAuto,
                V5VisionWBMode::kVisionWBManual => WhiteBalance::Manual({
                    let raw = unsafe { vexDeviceVisionWhiteBalanceGet(self.device) };

                    Color::new(raw.red, raw.green, raw.blue)
                }),
                _ => unreachable!(),
            },
        )
    }

    /// Sets the brightness percentage of the vision sensor. Should be between 0.0 and 1.0.
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
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Set brightness to 50%
    ///     _ = sensor.set_brightness(0.5);
    /// }
    /// ```
    pub fn set_brightness(&mut self, brightness: f64) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe { vexDeviceVisionBrightnessSet(self.device, (brightness * 100.0) as u8) }

        Ok(())
    }

    /// Sets the white balance of the vision sensor.
    ///
    /// White balance can be either automatically set or manually set through an RGB color.
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
    /// use vexide::{color::Color, prelude::*, smart::vision::WhiteBalance};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Set white balance to manual.
    ///     _ = sensor.set_white_balance(WhiteBalance::Manual(Color::WHITE));
    /// }
    /// ```
    pub fn set_white_balance(&mut self, white_balance: WhiteBalance) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe { vexDeviceVisionWhiteBalanceModeSet(self.device, white_balance.into()) }

        if let WhiteBalance::Manual(rgb) = white_balance {
            unsafe {
                vexDeviceVisionWhiteBalanceSet(
                    self.device,
                    V5_DeviceVisionRgb {
                        red: rgb.r,
                        green: rgb.g,
                        blue: rgb.b,

                        // Pretty sure this field does nothing, but PROS sets it to this.
                        //
                        // TODO: Run some hardware tests to see if this value actually influences
                        // white balance. Based on the Pixy2 API, I doubt it and bet this is just
                        // here for the LED setter, which uses the same type.
                        brightness: 255,
                    },
                );
            }
        }

        Ok(())
    }

    /// Configure the behavior of the LED indicator on the sensor.
    ///
    /// The default behavior is represented by [`LedMode::Auto`], which will display the color of
    /// the most prominent detected object's signature color. Alternatively, the LED can be
    /// configured to display a single RGB color.
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
    /// use vexide::{color::Color, prelude::*, smart::vision::LedMode};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Set the LED to red at 100% brightness.
    ///     _ = sensor.set_led_mode(LedMode::Manual(Color::RED, 1.0));
    /// }
    /// ```
    pub fn set_led_mode(&mut self, mode: LedMode) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe { vexDeviceVisionLedModeSet(self.device, mode.into()) }

        if let LedMode::Manual(rgb, brightness) = mode {
            unsafe {
                vexDeviceVisionLedColorSet(
                    self.device,
                    V5_DeviceVisionRgb {
                        red: rgb.r,
                        green: rgb.g,
                        blue: rgb.b,
                        brightness: (brightness * 100.0) as u8,
                    },
                );
            }
        }

        Ok(())
    }

    /// Sets the vision sensor's detection mode. See [`VisionMode`] for more information on what
    /// each mode does.
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
    /// use vexide::{prelude::*, smart::vision::VisionMode};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Place the sensor into "Wi-Fi mode", allowing you to connect to it via a hotspot
    ///     // and receive a video stream of its camera from another device.
    ///     _ = sensor.set_mode(VisionMode::Wifi);
    /// }
    /// ```
    pub fn set_mode(&mut self, mode: VisionMode) -> Result<(), PortError> {
        self.validate_port()?;

        unsafe {
            vexDeviceVisionWifiModeSet(
                self.device,
                match mode {
                    VisionMode::Wifi => V5VisionWifiMode::kVisionWifiModeOn,
                    _ => V5VisionWifiMode::kVisionWifiModeOff,
                },
            );

            vexDeviceVisionModeSet(
                self.device,
                match mode {
                    VisionMode::ColorDetection => V5VisionMode::kVisionModeNormal,
                    VisionMode::LineDetection => V5VisionMode::kVisionModeLineDetect,
                    VisionMode::MixedDetection => V5VisionMode::kVisionModeMixed,
                    // If the user requested Wi-Fi mode, then we already set
                    // it around 14 lines ago, so there's nothing to do here.
                    VisionMode::Wifi => return Ok(()),
                    VisionMode::Test => V5VisionMode::kVisionTypeTest,
                },
            );
        }

        Ok(())
    }

    /// Returns the current detection mode that the sensor is using.
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
    /// use vexide::{prelude::*, smart::vision::VisionMode};
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut sensor = VisionSensor::new(peripherals.port_1);
    ///
    ///     // Place the sensor into "Wi-Fi mode", allowing you to connect to it via a hotspot
    ///     // and receive a video stream of its camera from another device.
    ///     _ = sensor.set_mode(VisionMode::Wifi);
    ///
    ///     sleep(VisionSensor::UPDATE_INTERVAL).await;
    ///
    ///     // Since we just set the mode, we can get the mode off the sensor to verify that it's
    ///     // now in Wi-Fi mode.
    ///     if let Ok(mode) = sensor.mode() {
    ///         assert_eq!(mode, VisionMode::Wifi);
    ///     }
    /// }
    /// ```
    pub fn mode(&self) -> Result<VisionMode, PortError> {
        self.validate_port()?;

        if unsafe { vexDeviceVisionWifiModeGet(self.device) } == V5VisionWifiMode::kVisionWifiModeOn
        {
            return Ok(VisionMode::Wifi);
        }

        Ok(unsafe { vexDeviceVisionModeGet(self.device) }.into())
    }
}

impl SmartDevice for VisionSensor {
    /// The frametime of the Vision Sensor.
    const UPDATE_INTERVAL: Duration = Duration::from_millis(20);

    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Vision
    }
}
impl From<VisionSensor> for SmartPort {
    fn from(device: VisionSensor) -> Self {
        device.port
    }
}

/// A vision detection color signature.
///
/// Vision signatures contain information used by the vision sensor to detect objects of a certain
/// color. These signatures are typically generated through VEX's vision utility tool rather than
/// written by hand. For creating signatures using the utility, see [`from_utility`].
///
/// [`from_utility`]: VisionSignature::from_utility
///
/// # Format & Detection Overview
///
/// Vision signatures operate in a version of the Y'UV color space, specifically using the "U" and
/// "V" chroma components for edge detection purposes. This can be seen in the `u_threshold` and
/// `v_threshold` fields of this struct. These fields place three "threshold" (min, max, mean)
/// values on the u and v chroma values detected by the sensor. The values are then transformed to a
/// 3D lookup table to detect actual colors.
///
/// There is additionally a `range` field, which works as a scale factor or threshold for how
/// lenient edge detection should be.
///
/// Signatures can additionally be grouped together into [`VisionCode`]s, which narrow the filter
/// for object detection by requiring two colors.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VisionSignature {
    /// The (min, max, mean) values on the "U" axis.
    ///
    /// This defines a threshold of values for the sensor to match against a certain chroma in the
    /// Y'UV color space - specifically on the U component.
    pub u_threshold: (i32, i32, i32),

    /// The (min, max, mean) values on the V axis.
    ///
    /// This defines a threshold of values for the sensor to match against a certain chroma in the
    /// Y'UV color space - specifically on the "V" component.
    pub v_threshold: (i32, i32, i32),

    /// The signature range scale factor.
    ///
    /// This value effectively serves as a threshold for how lenient the sensor should be when
    /// detecting the edges of colors. This value ranges from 0-11 in Vision Utility.
    ///
    /// Higher values of `range` will increase the range of brightness that the sensor will
    /// consider to be part of the signature. Lighter/Darker shades of the signature's color will
    /// be detected more often.
    pub range: f32,

    /// The signature's flags.
    pub flags: u8,
}

impl VisionSignature {
    /// Create a [`VisionSignature`].
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::smart::vision::VisionSignature;
    ///
    /// let my_signature = VisionSignature::new((10049, 11513, 10781), (-425, 1, -212), 4.1);
    /// ```
    #[must_use]
    pub const fn new(
        u_threshold: (i32, i32, i32),
        v_threshold: (i32, i32, i32),
        range: f32,
    ) -> Self {
        Self {
            flags: 0,
            u_threshold,
            v_threshold,
            range,
        }
    }

    /// Create a [`VisionSignature`] using the same format as VEX's Vision Utility tool.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::smart::vision::VisionSignature;
    ///
    /// // Register a signature for detecting red objects.
    /// // This numbers in this signature was generated using VEX's vision utility app.
    /// let my_signature = VisionSignature::from_utility(1, 10049, 11513, 10781, -425, 1, -212, 4.1, 0);
    /// ```
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn from_utility(
        _id: u8, // We don't store IDs in our vision signatures.
        u_min: i32,
        u_max: i32,
        u_mean: i32,
        v_min: i32,
        v_max: i32,
        v_mean: i32,
        range: f32,
        _signature_type: u32, // This is handled automatically by [`VisionSensor::add_code`].
    ) -> Self {
        Self {
            u_threshold: (u_min, u_max, u_mean),
            v_threshold: (v_min, v_max, v_mean),
            range,
            flags: 0,
        }
    }
}

impl From<V5_DeviceVisionSignature> for VisionSignature {
    fn from(value: V5_DeviceVisionSignature) -> Self {
        Self {
            u_threshold: (value.uMin, value.uMax, value.uMean),
            v_threshold: (value.vMin, value.vMax, value.vMean),
            range: value.range,
            flags: value.flags,
        }
    }
}

/// A vision detection code.
///
/// Codes are a special type of detection signature that group multiple [`VisionSignature`]s
/// together. A [`VisionCode`] can associate 2-5 color signatures together, detecting the resulting
/// object when its color signatures are present close to each other.
///
/// These codes work very similarly to [Pixy2 Color Codes](https://docs.pixycam.com/wiki/doku.php?id=wiki:v2:using_color_codes).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VisionCode(
    pub u8,
    pub u8,
    pub Option<u8>,
    pub Option<u8>,
    pub Option<u8>,
);

impl VisionCode {
    /// Creates a new vision code.
    ///
    /// Two signatures are required to create a vision code, with an additional three optional
    /// signatures.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::smart::vision::VisionCode;
    ///
    /// // Create a vision code associated with signatures 1, 2, and 3.
    /// let code = VisionCode::new(1, 2, Some(3), None, None);
    /// ```
    #[must_use]
    pub const fn new(
        sig_1: u8,
        sig_2: u8,
        sig_3: Option<u8>,
        sig_4: Option<u8>,
        sig_5: Option<u8>,
    ) -> Self {
        Self(sig_1, sig_2, sig_3, sig_4, sig_5)
    }

    /// Creates a [`VisionCode`] from a bit representation of its signature IDs.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::smart::vision::VisionCode;
    ///
    /// let sig_1_id: u8 = 1;
    /// let sig_2_id: u8 = 2;
    ///
    /// let mut code_id: u16 = 0;
    ///
    /// // Store the bits of IDs 1 and 2 in the code ID.
    /// code_id = (code_id << 3) | u16::from(sig_1_id);
    /// code_id = (code_id << 3) | u16::from(sig_2_id);
    ///
    /// // Create a [`VisionCode`] from signatures 1 and 2.
    /// let code = VisionCode::from_id(code_id);
    /// ```
    #[must_use]
    pub const fn from_id(id: u16) -> Self {
        const MASK: u16 = (1 << 3) - 1;

        Self(
            ((id >> 12) & MASK) as u8,
            ((id >> 9) & MASK) as u8,
            match ((id >> 6) & MASK) as u8 {
                0 => None,
                sig => Some(sig),
            },
            match ((id >> 3) & MASK) as u8 {
                0 => None,
                sig => Some(sig),
            },
            match (id & MASK) as u8 {
                0 => None,
                sig => Some(sig),
            },
        )
    }

    /// Returns `true` if a given signature ID is stored in this code.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::smart::vision::VisionCode;
    ///
    /// // Create a vision code associated with signatures 1, 2, and 3.
    /// let code = VisionCode::new(1, 2, Some(3), None, None);
    ///
    /// assert!(code.contains_signature(1));
    /// ```
    #[must_use]
    pub const fn contains_signature(&self, id: u8) -> bool {
        if self.0 == id || self.1 == id {
            return true;
        }

        //TODO: Update this to use [`Option::is_some_and`] once that's stabilized as const.
        if let Some(sig_3) = self.2
            && sig_3 == id
        {
            return true;
        }
        if let Some(sig_4) = self.3
            && sig_4 == id
        {
            return true;
        }
        if let Some(sig_5) = self.4
            && sig_5 == id
        {
            return true;
        }

        false
    }

    /// Returns the internal ID used by the sensor to determine which signatures belong to which
    /// code.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::smart::vision::VisionCode;
    ///
    /// let sig_1_id: u8 = 1;
    /// let sig_2_id: u8 = 2;
    ///
    /// let mut code_id: u16 = 0;
    ///
    /// // Store the bits of IDs 1 and 2 in the code ID.
    /// code_id = (code_id << 3) | u16::from(sig_1_id);
    /// code_id = (code_id << 3) | u16::from(sig_2_id);
    ///
    /// // Create a [`VisionCode`] from signatures 1 and 2.
    /// let code = VisionCode::from_id(code_id);
    ///
    /// // The ID of the code we just created should be identical to the bit representation
    /// // containing each signature ID we created it from.
    /// assert_eq!(code.id(), code_id);
    /// ```
    #[must_use]
    pub fn id(&self) -> u16 {
        let mut id: u16 = 0;

        id = (id << 3) | u16::from(self.0);
        id = (id << 3) | u16::from(self.1);
        id = (id << 3) | u16::from(self.2.unwrap_or_default());
        id = (id << 3) | u16::from(self.3.unwrap_or_default());
        id = (id << 3) | u16::from(self.4.unwrap_or_default());

        id
    }
}

impl From<(u8, u8)> for VisionCode {
    /// Convert a tuple of two [`VisionSignature`]s into a [`VisionCode`].
    fn from(signatures: (u8, u8)) -> Self {
        Self(signatures.0, signatures.1, None, None, None)
    }
}

impl From<(u8, u8, u8)> for VisionCode {
    /// Convert a tuple of three [`VisionSignature`]s into a [`VisionCode`].
    fn from(signatures: (u8, u8, u8)) -> Self {
        Self(signatures.0, signatures.1, Some(signatures.2), None, None)
    }
}

impl From<(u8, u8, u8, u8)> for VisionCode {
    /// Convert a tuple of four [`VisionSignature`]s into a [`VisionCode`].
    fn from(signatures: (u8, u8, u8, u8)) -> Self {
        Self(
            signatures.0,
            signatures.1,
            Some(signatures.2),
            Some(signatures.3),
            None,
        )
    }
}

impl From<(u8, u8, u8, u8, u8)> for VisionCode {
    /// Convert a tuple of five [`VisionSignature`]s into a [`VisionCode`].
    fn from(signatures: (u8, u8, u8, u8, u8)) -> Self {
        Self(
            signatures.0,
            signatures.1,
            Some(signatures.2),
            Some(signatures.3),
            Some(signatures.4),
        )
    }
}

/// A possible "detection mode" for the vision sensor.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionMode {
    /// Uses color signatures and codes to identify objects in blocks.
    #[default]
    ColorDetection,

    /// Uses line tracking to identify lines.
    LineDetection,

    /// Both color signatures and lines will be detected as objects.
    MixedDetection,

    /// Sets the sensor into "Wi-Fi mode", which disables all forms of object detection and enables
    /// the sensor's onboard Wi-Fi hotspot for streaming camera data over a web server.
    ///
    /// Once enabled, the sensor will create a wireless network with an SSID in the format of
    /// VISION_XXXX. The sensor's camera feed is available at `192.168.1.1`.
    ///
    /// This mode will be automatically disabled when connected to field control.
    Wifi,

    /// Unknown use.
    Test,
}

impl From<V5VisionMode> for VisionMode {
    fn from(value: V5VisionMode) -> Self {
        match value {
            V5VisionMode::kVisionModeNormal => Self::ColorDetection,
            V5VisionMode::kVisionModeLineDetect => Self::LineDetection,
            V5VisionMode::kVisionModeMixed => Self::MixedDetection,
            V5VisionMode::kVisionTypeTest => Self::Test,
            _ => unreachable!(),
        }
    }
}

/// Defines a source for what method was used to detect a [`VisionObject`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionSource {
    /// A normal vision signature not associated with a color code was used to detect this object.
    Signature(u8),

    /// Multiple signatures joined in a color code were used to detect this object.
    Code(VisionCode),

    /// Line detection was used to find this object.
    Line,
}

/// A detected vision object.
///
/// This struct contains metadata about objects detected by the vision sensor. Objects are detected
/// by calling [`VisionSensor::objects`] after adding signatures and color codes to the sensor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisionObject {
    /// The ID of the signature or color code used to detect this object.
    pub source: DetectionSource,

    /// The width of the detected object's bounding box in pixels.
    pub width: u16,

    /// The height of the detected object's bounding box in pixels.
    pub height: u16,

    /// The top-left coordinate of the detected object relative to the top-left of the camera's
    /// field of view.
    pub offset: Point2<u16>,

    /// The center coordinate of the detected object relative to the top-left of the camera's field
    /// of view.
    pub center: Point2<u16>,

    /// The approximate degrees of rotation of the detected object's bounding box.
    pub angle: Angle,
}

impl From<V5_DeviceVisionObject> for VisionObject {
    fn from(value: V5_DeviceVisionObject) -> Self {
        Self {
            source: match value.r#type {
                V5VisionBlockType::kVisionTypeColorCode => {
                    DetectionSource::Code(VisionCode::from_id(value.signature))
                }
                V5VisionBlockType::kVisionTypeNormal => {
                    DetectionSource::Signature(value.signature as u8)
                }
                V5VisionBlockType::kVisionTypeLineDetect => DetectionSource::Line,
                x => panic!("Unknown vision block type: {x:?}"),
            },
            width: value.width,
            height: value.height,
            offset: Point2 {
                x: value.xoffset,
                y: value.yoffset,
            },
            center: Point2 {
                x: value.xoffset + (value.width / 2),
                y: value.yoffset + (value.height / 2),
            },
            angle: Angle::from_degrees(f64::from(value.angle) / 10.0),
        }
    }
}

/// Vision Sensor white balance mode.
///
/// Represents a white balance configuration for the vision sensor's camera.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteBalance {
    /// Automatic Mode
    ///
    /// The sensor will automatically adjust the camera's white balance, using the brightest part
    /// of the image as a white point.
    #[default]
    Auto,

    /// "Startup" Automatic Mode
    ///
    /// The sensor will automatically adjust the camera's white balance, but will only perform this
    /// adjustment once on power-on.
    StartupAuto,

    /// Manual Mode
    ///
    /// Allows for manual control over white balance using an RGB color.
    Manual(Color),
}

impl From<WhiteBalance> for V5VisionWBMode {
    fn from(value: WhiteBalance) -> Self {
        match value {
            WhiteBalance::Auto => Self::kVisionWBNormal,
            WhiteBalance::StartupAuto => Self::kVisionWBStart,
            WhiteBalance::Manual(_) => Self::kVisionWBManual,
        }
    }
}

/// Vision Sensor LED mode.
///
/// Represents the states that the integrated LED indicator on a vision sensor can be in.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum LedMode {
    /// Automatic Mode
    ///
    /// When in automatic mode, the integrated LED will display the color of the most prominent
    /// detected object's signature color.
    #[default]
    Auto,

    /// Manual Mode
    ///
    /// When in manual mode, the integrated LED will display a user-set RGB color code and
    /// brightness percentage from 0.0-1.0.
    Manual(Color, f64),
}

impl From<LedMode> for V5VisionLedMode {
    fn from(value: LedMode) -> Self {
        match value {
            LedMode::Auto => Self::kVisionLedModeAuto,
            LedMode::Manual(_, _) => Self::kVisionLedModeManual,
        }
    }
}

/// Error returned by [`VisionSensor::objects`] and [`VisionSensor::object_count`].
#[derive(Debug, Clone, Copy, Eq, PartialEq, Snafu)]
pub enum VisionObjectError {
    /// Objects cannot be detected while Wi-Fi mode is enabled.
    WifiMode,

    /// Failed to read vision object.
    InvalidObject,

    /// Generic port related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
}

/// Error returned by [`VisionSensor`] methods that get/set color signatures.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Snafu)]
pub enum VisionSignatureError {
    /// The camera could not be read.
    ReadingFailed,

    /// Generic port related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
