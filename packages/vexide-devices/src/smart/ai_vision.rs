//! AI Vision sensor device.
//!
//! This module provides an API for interacting with the AI Vision sensor.
//! The AI Vision sensor is meant to be a direct upgrade from the [Vision Sensor](super::vision)
//! with a wider camera range and AI model capabilities.
//!
//! # Hardware overview
//!
//! The AI Vision sensor has three detection modes that can all be enabled at the same time:
//!     - [Color detection](AiVisionDetectionMode::COLOR)
//!     - [Custom model detection](AiVisionDetectionMode::MODEL)
//!     - [AprilTag detection](AiVisionDetectionMode::APRILTAG) (requires color detection to be enabled)
//! Currently there is no known way to upload custom models to the sensor and fields do not have AprilTags.
//! However, there are built-in models that can be used for detection.
//! See [VEX's documentation](https://kb.vex.com/hc/en-us/articles/30326315023892-Using-AI-Classifications-with-the-AI-Vision-Sensor) for more information.
//!
//! The resolution of the AI Vision sensor is 320x240 pixels.
//! It has a horizontal FOV of 74 degrees and a vertical FOV of 63 degrees.
//! Both of these values are a slight upgrade from the Vision Sensor.
//!
//! Unlike the Vision Sensor, the AI Vision sensor uses more readable color signatures
//! that may be created without the AI Vision utility.
//! It still has a USB port that can be used to create these signatures with VEX's utility.

use alloc::{
    ffi::{CString, IntoStringError},
    string::String,
    vec::Vec,
};

use bitflags::bitflags;
use mint::Point2;
use rgb::Rgb;
use snafu::Snafu;
use vex_sdk::{
    vexDeviceAiVisionClassNameGet, vexDeviceAiVisionCodeGet, vexDeviceAiVisionCodeSet,
    vexDeviceAiVisionColorGet, vexDeviceAiVisionColorSet, vexDeviceAiVisionModeSet,
    vexDeviceAiVisionObjectCountGet, vexDeviceAiVisionObjectGet, vexDeviceAiVisionStatusGet,
    vexDeviceAiVisionTemperatureGet, V5_DeviceAiVisionCode, V5_DeviceAiVisionColor,
    V5_DeviceAiVisionObject, V5_DeviceT,
};

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::PortError;

type Result<T, E = AiVisionError> = core::result::Result<T, E>;

#[repr(u8)]
enum ObjectType {
    Unknown = 0,
    Color = (1 << 0),
    Code = (1 << 1),
    Model = (1 << 2),
    AprilTag = (1 << 3),
    All = 0x3F,
}
impl From<u8> for ObjectType {
    fn from(value: u8) -> Self {
        #[allow(clippy::match_same_arms)]
        match value {
            1 => ObjectType::Color,
            2 => ObjectType::Code,
            4 => ObjectType::Model,
            8 => ObjectType::AprilTag,
            63 => ObjectType::All,
            _ => ObjectType::Unknown,
        }
    }
}

/// The data associated with an AI Vision object.
/// The data is different depending on the type of object detected.
#[derive(Debug, Clone, PartialEq)]
pub enum AiVisionObject {
    /// An object detected by color blob detection.
    Color {
        /// ID of the signature used to detect this object.
        id: u8,
        /// The top-left corner of the object.
        position: Point2<u16>,
        /// The width of the object.
        width: u16,
        /// The height of the object.
        height: u16,
    },

    /// An object detected by color code detection.
    Code {
        /// ID of the code used to detect this object.
        id: u8,
        /// The position of the object.
        position: Point2<u16>,
        /// The width of the object.
        width: u16,
        /// The height of the object.
        height: u16,
        /// The angle of the object's associated colors. Not always reliably available.
        angle: f64,
    },

    /// An object detected by apriltag detection.
    AprilTag {
        /// The detected AprilTag(s) ID number
        id: u8,
        /// Position of the top-left corner of the tag
        top_left: mint::Point2<i16>,
        /// Position of the top-right corner of the tag
        top_right: mint::Point2<i16>,
        /// Position of the top-right corner of the tag
        bottom_right: mint::Point2<i16>,
        /// Position of the bottom-left corner of the tag
        bottom_left: mint::Point2<i16>,
    },

    /// An object detected by an onboard model.
    Model {
        /// ID of the detected object.
        id: u8,
        /// A string describing the specific onboard model used to detect this object.
        classification: String,
        /// The position of the object.
        position: Point2<u16>,
        /// The width of the object.
        width: u16,
        /// The height of the object.
        height: u16,
        /// The confidence reported by the model.
        confidence: u16,
    },
}

/// Possible april tag families to be detected by the sensor.
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum AprilTagFamily {
    /// Circle21h7 family
    #[default]
    Circle21h7 = 0,
    /// 16h5 family
    Tag16h5 = 1,
    /// 25h9 family
    Tag25h9 = 2,
    /// 36h11 family
    Tag36h11 = 3,
}

bitflags! {
    /// Represents the mode of the AI Vision sensor.
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct AiVisionFlags: u8 {
        /// Disable apriltag detection
        const DISABLE_APRILTAG = 1 << 0;
        /// Disable color detection
        const DISABLE_COLOR = 1 << 1;
        /// Disable model detection
        const DISABLE_MODEL = 1 << 2;
        /// Merge color blobs?
        const COLOR_MERGE = 1 << 4;
        /// Disable status overlay
        const DISABLE_STATUS_OVERLAY = 1 << 5;
        /// Disable USB overlay
        const DISABLE_USB_OVERLAY = 1 << 7;
    }

    /// Flags relating to the sensor's detection mode.
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct AiVisionDetectionMode: u8 {
        /// Enable apriltag detection
        const APRILTAG = 1 << 0;
        /// Enable color detection
        const COLOR = 1 << 1;
        /// Enable model detection
        const MODEL = 1 << 2;
        /// Merge color blobs?
        const COLOR_MERGE = 1 << 4;
    }
}

impl Default for AiVisionFlags {
    fn default() -> Self {
        Self::DISABLE_USB_OVERLAY
    }
}

impl From<AiVisionDetectionMode> for AiVisionFlags {
    fn from(value: AiVisionDetectionMode) -> Self {
        !Self::from_bits((value ^ AiVisionDetectionMode::COLOR_MERGE).bits()).unwrap_or_default()
            & !(Self::DISABLE_STATUS_OVERLAY | Self::DISABLE_USB_OVERLAY)
    }
}

/// A color signature used by an AI Vision Sensor to detect color blobs.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AiVisionColor {
    /// The RGB color value.
    pub rgb: Rgb<u8>,
    /// The accepted hue range of the color. VEXcode limits this value to [0, 20]
    pub hue_range: f32,
    /// The accepted saturation range of the color.
    pub saturation_range: f32,
}

/// A color code used by an AI Vision Sensor to detect groups of color blobs.
///
/// The color code can have up to 7 color signatures.
/// When the colors in a color code are detected next to eachother, the sensor will detect the color code.
pub struct AiVisionColorCode([Option<u8>; 7]);
impl AiVisionColorCode {
    /// Creates a new color code with the given color signature ids.
    #[must_use]
    pub const fn new<const N: usize>(code: [Option<u8>; 7]) -> Self {
        Self(code)
    }

    /// Returns the color signature ids in the color code.
    #[must_use]
    pub fn colors(&self) -> Vec<u8> {
        self.0.iter().flatten().copied().collect()
    }
}
impl From<(u8,)> for AiVisionColorCode {
    fn from(value: (u8,)) -> Self {
        Self([Some(value.0), None, None, None, None, None, None])
    }
}
impl From<(u8, u8)> for AiVisionColorCode {
    fn from(value: (u8, u8)) -> Self {
        Self([Some(value.0), Some(value.1), None, None, None, None, None])
    }
}
impl From<(u8, u8, u8)> for AiVisionColorCode {
    fn from(value: (u8, u8, u8)) -> Self {
        Self([
            Some(value.0),
            Some(value.1),
            Some(value.2),
            None,
            None,
            None,
            None,
        ])
    }
}
impl From<(u8, u8, u8, u8)> for AiVisionColorCode {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self([
            Some(value.0),
            Some(value.1),
            Some(value.2),
            Some(value.3),
            None,
            None,
            None,
        ])
    }
}
impl From<(u8, u8, u8, u8, u8)> for AiVisionColorCode {
    fn from(value: (u8, u8, u8, u8, u8)) -> Self {
        Self([
            Some(value.0),
            Some(value.1),
            Some(value.2),
            Some(value.3),
            Some(value.4),
            None,
            None,
        ])
    }
}
impl From<(u8, u8, u8, u8, u8, u8)> for AiVisionColorCode {
    fn from(value: (u8, u8, u8, u8, u8, u8)) -> Self {
        Self([
            Some(value.0),
            Some(value.1),
            Some(value.2),
            Some(value.3),
            Some(value.4),
            Some(value.5),
            None,
        ])
    }
}
impl From<(u8, u8, u8, u8, u8, u8, u8)> for AiVisionColorCode {
    fn from(value: (u8, u8, u8, u8, u8, u8, u8)) -> Self {
        Self([
            Some(value.0),
            Some(value.1),
            Some(value.2),
            Some(value.3),
            Some(value.4),
            Some(value.5),
            Some(value.6),
        ])
    }
}
macro_rules! impl_code_from_array {
    ($($size:literal),*) => {
        $(
            impl From<[Option<u8>; $size]> for AiVisionColorCode {
                fn from(value: [Option<u8>; $size]) -> Self {
                    let mut code = [None; 7];
                    code[..$size].copy_from_slice(&value[..]);
                    Self(code)
                }
            }
            impl From<[u8; $size]> for AiVisionColorCode {
                fn from(value: [u8; $size]) -> Self {
                    let mut code = [None; 7];
                    for (i, id) in value.iter().enumerate() {
                        code[i] = Some(*id);
                    }
                    Self(code)
                }
            }
        )*
    };
}
impl_code_from_array!(1, 2, 3, 4, 5, 6, 7);

/// An AI Vision sensor.
pub struct AiVisionSensor {
    port: SmartPort,
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for AiVisionSensor {}
unsafe impl Sync for AiVisionSensor {}

impl AiVisionSensor {
    /// Maximum number of objects that can be detected at once.
    pub const MAX_OBJECTS: usize = 24;

    /// The horizontal resolution of the AI Vision sensor.
    pub const HORIZONTAL_RESOLUTION: u16 = 320;

    /// The vertical resolution of the AI Vision sensor.
    pub const VERTICAL_RESOLUTION: u16 = 240;

    /// The horizontal FOV of the vision sensor in degrees.
    pub const HORIZONTAL_FOV: f32 = 74.0;

    /// The vertical FOV of the vision sensor in degrees.
    pub const VERTICAL_FOV: f32 = 63.0;

    /// The diagonal FOV of the vision sensor in degrees.
    pub const DIAGONAL_FOV: f32 = 87.0;

    const RESET_FLAG: u32 = (1 << 30);
    const TAG_SET_FLAG: u32 = (1 << 29);
    const MODE_SET_FLAG: u32 = (1 << 25);
    const TEST_MODE_FLAG: u32 = (1 << 26);
    const AWB_START_FLAG: u32 = (1 << 27);

    // const AWB_START_VALUE: u32 = 4;

    /// Create a new AI Vision sensor from a smart port with the given brightness and contrast.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     // Do something with the AI Vision sensor
    /// }
    /// ```
    #[must_use]
    pub fn new(port: SmartPort) -> Self {
        let device = unsafe { port.device_handle() };

        unsafe {
            vexDeviceAiVisionModeSet(device, Self::RESET_FLAG);
        }

        Self { port, device }
    }

    /// Returns the current temperature of the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     loop {
    ///         println!("{:?}", ai_vision.temperature());
    ///         sleep(AiVisionSensor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn temperature(&self) -> Result<f64> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceAiVisionTemperatureGet(self.device) })
    }

    /// Sets a color code used to detect groups of colors.
    ///
    /// # Panics
    ///
    /// - Panics if the given color code contains an ID that is not in the interval [1, 7].
    /// - Panics if the given ID is not in the interval [1, 8].
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     let color = AiVisionColor {
    ///         rgb: Rgb::new(255, 0, 0),
    ///         hue: 10.0,
    ///         saturation: 1.0,
    ///     };
    ///     _ = ai_vision.set_color(1, color);
    ///     let code = AiVisionColorCode::from([1]);
    ///     _ = ai_vision.set_color_code(1, &code);
    /// }
    /// ```
    pub fn set_color_code(&mut self, id: u8, code: &AiVisionColorCode) -> Result<()> {
        assert!(
            !(1..=8).contains(&id),
            "The given ID ({id}) is out of the interval [1, 8]."
        );
        self.validate_port()?;

        // Copy the color code into the V5_DeviceAiVisionCode struct
        let mut ids = [0u8; 7];
        for (i, id) in code.0.iter().flatten().enumerate() {
            assert!(
                !(1..=7).contains(id),
                "The given color code contains an ID ({id}) that is out of the interval [1, 7]."
            );
            ids[i] = *id;
        }

        // Calculate the length of the color code color ids
        let mut len = 0;
        for id in &ids {
            if *id != 0 {
                len += 1;
            } else {
                break;
            }
        }

        let mut code = V5_DeviceAiVisionCode {
            id,
            len,
            c1: i16::from(ids[0]),
            c2: i16::from(ids[1]),
            c3: i16::from(ids[2]),
            c4: i16::from(ids[3]),
            c5: i16::from(ids[4]),
            c6: i16::from(ids[5]),
            c7: i16::from(ids[6]),
        };
        unsafe {
            vexDeviceAiVisionCodeSet(self.device, core::ptr::from_mut(&mut code));
        }

        Ok(())
    }

    /// Returns the color code set on the AI Vision sensor with the given ID if it exists.
    ///
    /// # Panics
    ///
    /// - Panics if the given ID is not in the interval [1, 8].
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     let code = AiVisionColorCode::from([1]);
    ///     _ = ai_vision.set_color_code(1, &code);
    ///     if let Ok(Some(code)) = ai_vision.color_code(1) {
    ///          println!("{:?}", code);
    ///     } else {
    ///         println!("Something went wrong!");
    ///     }
    /// }
    /// ```
    pub fn color_code(&self, id: u8) -> Result<Option<AiVisionColorCode>> {
        assert!(
            !(1..=8).contains(&id),
            "The given ID ({id}) is out of the interval [1, 8]."
        );
        self.validate_port()?;

        // Get the color code from the sensor
        let mut code: V5_DeviceAiVisionCode = unsafe { core::mem::zeroed() };
        let read = unsafe {
            vexDeviceAiVisionCodeGet(self.device, id.into(), core::ptr::from_mut(&mut code))
        };
        if !read {
            return Ok(None);
        }

        // Get the valid (hopefully) color ids from the color code
        let ids = [
            code.c1, code.c2, code.c3, code.c4, code.c5, code.c6, code.c7,
        ];
        let mut color_ids = [None; 7];
        for i in 0..code.len as usize {
            color_ids[i] = Some(ids[i] as u8);
        }

        let signature = AiVisionColorCode::from(color_ids);

        Ok(Some(signature))
    }

    /// Returns all color codes set on the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     _ = ai_vision.set_color_code(1, &AiVisionColorCode::from([1]));
    ///     _ = ai_vision.set_color_code(2, &AiVisionColorCode::from([1, 2]));
    ///     println!("{:?}", ai_vision.color_codes());
    /// }
    /// ```
    pub fn color_codes(&self) -> Result<[Option<AiVisionColorCode>; 8]> {
        Ok([
            self.color_code(1)?,
            self.color_code(2)?,
            self.color_code(3)?,
            self.color_code(4)?,
            self.color_code(5)?,
            self.color_code(6)?,
            self.color_code(7)?,
            self.color_code(8)?,
        ])
    }

    /// Sets a color signature for the AI Vision sensor.
    ///
    /// # Panics
    ///
    /// - Panics if the given ID is not in the range [1, 7].
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     let color = AiVisionColor {
    ///         rgb: Rgb::new(255, 0, 0),
    ///         hue: 10.0,
    ///         saturation: 1.0,
    ///     };
    ///     _ = ai_vision.set_color(1, color);
    ///     _ = ai_vision.set_color(2, color);
    /// }
    /// ```
    pub fn set_color(&mut self, id: u8, color: AiVisionColor) -> Result<()> {
        assert!(
            !(1..=7).contains(&id),
            "The given ID ({id}) is out of the interval [1, 7]."
        );
        self.validate_port()?;

        let mut color = V5_DeviceAiVisionColor {
            id,
            red: color.rgb.r,
            grn: color.rgb.g,
            blu: color.rgb.b,
            hangle: color.hue_range,
            hdsat: color.saturation_range,
            reserved: 0,
        };

        //TODO: Make sure that the color is not modified by this function
        unsafe { vexDeviceAiVisionColorSet(self.device, core::ptr::from_mut(&mut color)) }

        Ok(())
    }

    /// Returns the color signature set on the AI Vision sensor with the given ID if it exists.
    ///
    /// # Panics
    ///
    /// - Panics if the given ID is not in the interval [1, 7].
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     let color = AiVisionColor {
    ///         rgb: Rgb::new(255, 0, 0),
    ///         hue: 10.0,
    ///         saturation: 1.0,
    ///     };
    ///     _ = ai_vision.set_color(1, color);
    ///     if let Ok(Some(color)) = ai_vision.color(1) {
    ///         println!("{:?}", color);
    ///     } else {
    ///         println!("Something went wrong!");
    ///     }
    /// }
    /// ```
    pub fn color(&self, id: u8) -> Result<Option<AiVisionColor>> {
        assert!(
            !(1..=7).contains(&id),
            "The given ID ({id}) is out of the interval [1, 7]."
        );
        self.validate_port()?;

        let mut color: V5_DeviceAiVisionColor = unsafe { core::mem::zeroed() };

        let read = unsafe {
            vexDeviceAiVisionColorGet(self.device, u32::from(id), core::ptr::from_mut(&mut color))
        };
        if !read {
            return Ok(None);
        }

        Ok(Some(AiVisionColor {
            rgb: Rgb::new(color.red, color.grn, color.blu),
            hue_range: color.hangle,
            saturation_range: color.hdsat,
        }))
    }

    /// Returns all color signatures set on the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     let color = AiVisionColor {
    ///         rgb: Rgb::new(255, 0, 0),
    ///         hue: 10.0,
    ///         saturation: 1.0,
    ///     };
    ///     _ = ai_vision.set_color(1, color);
    ///     let colors = ai_vision.colors().unwrap();
    ///     println!("{:?}", colors);
    /// }
    /// ```
    pub fn colors(&self) -> Result<[Option<AiVisionColor>; 7]> {
        Ok([
            self.color(1)?,
            self.color(2)?,
            self.color(3)?,
            self.color(4)?,
            self.color(5)?,
            self.color(6)?,
            self.color(7)?,
        ])
    }

    /// Sets the detection mode of the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     _ = ai_vision.set_detection_mode(AiVisionDetectionMode::COLOR | AiVisionDetectionMode::COLOR_MERGE);
    /// }
    /// ```
    pub fn set_detection_mode(&mut self, mode: AiVisionDetectionMode) -> Result<()> {
        let flags = (self.flags()?
            & (AiVisionFlags::DISABLE_USB_OVERLAY | AiVisionFlags::DISABLE_STATUS_OVERLAY))
            | AiVisionFlags::from(mode);
        self.set_flags(flags)
    }

    fn raw_status(&self) -> Result<u32> {
        self.validate_port()?;
        let status = unsafe { vexDeviceAiVisionStatusGet(self.device) };
        Ok(status)
    }

    /// Returns the current flags of the AI Vision sensor including the detection mode
    /// flags set by [`Self::set_detection_mode`].
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     println!("{:?}", ai_vision.flags());
    /// }
    /// ```
    pub fn flags(&self) -> Result<AiVisionFlags> {
        // Only care about the first byte of status.
        // See https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c988c99e1f9b3a6d3c3fd91591b6dac1
        Ok(AiVisionFlags::from_bits_retain(
            (self.raw_status()? & 0xff) as u8,
        ))
    }

    /// Set the full flags of the AI Vision sensor, including the detection mode.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     // Enable all detection modes except for custom model and disable USB overlay
    ///     let flags = AiVisionFlags::DISABLE_USB_OVERLAY | AiVisionFlags::DISABLE_MODEL;
    ///     _ = ai_vision.set_flags(flags);
    /// }
    /// ```
    pub fn set_flags(&mut self, mode: AiVisionFlags) -> Result<()> {
        // Status is shifted to the right from mode. Least-significant byte is missing.
        // See https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c988c99e1f9b3a6d3c3fd91591b6dac1
        let mut new_mode = self.raw_status()? << 8;

        new_mode &= !(0xff << 8); // Clear the mode bits.
                                  // Set the mode bits and set the update flag in byte 4.
        new_mode |= (u32::from(mode.bits()) << 8) | Self::MODE_SET_FLAG;

        // Update mode
        unsafe { vexDeviceAiVisionModeSet(self.device, new_mode) }

        Ok(())
    }

    /// Restarts the automatic white balance process.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn start_awb(&mut self) -> Result<()> {
        // Status is shifted to the right from mode. Least-significant byte is missing.
        // See https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c988c99e1f9b3a6d3c3fd91591b6dac1
        let mut new_mode = self.raw_status()? << 8;

        new_mode &= !(0xff << 16); // Clear byte 3
        new_mode |= (1 << 18) | Self::AWB_START_FLAG;

        // Update mode
        unsafe { vexDeviceAiVisionModeSet(self.device, new_mode) }

        Ok(())
    }

    /// Enables and begins the automatic white balance process.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn enable_test(&mut self, test: u8) -> Result<()> {
        // Status is shifted to the right from mode. Least-significant byte is missing.
        // See https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c988c99e1f9b3a6d3c3fd91591b6dac1
        let mut new_mode = self.raw_status()? << 8;

        new_mode &= !(0xff << 16); // Clear byte 3
        new_mode |= (u32::from(test) << 16) | Self::TEST_MODE_FLAG;

        // Update mode
        unsafe { vexDeviceAiVisionModeSet(self.device, new_mode) }

        Ok(())
    }

    /// Sets the family of apriltag that will be detected
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     _ = ai_vision.set_apriltag_family(AprilTagFamily::Tag16h5);
    /// }
    /// ```
    pub fn set_apriltag_family(&mut self, family: AprilTagFamily) -> Result<()> {
        // Status is shifted to the right from mode. Least-significant byte is missing.
        // See https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c988c99e1f9b3a6d3c3fd91591b6dac1
        let mut new_mode = self.raw_status()? << 8;

        new_mode &= !(0xff << 16); // Clear the existing apriltag family bits.
        new_mode |= u32::from(family as u8) << 16 | Self::TAG_SET_FLAG; // Set family bits

        // Update mode
        unsafe { vexDeviceAiVisionModeSet(self.device, new_mode) }

        Ok(())
    }

    /// Returns all objects detected by the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    ///
    /// # Examples
    ///
    /// Loop through all objects of a specific type
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     loop {
    ///         let objects = ai_vision.objects().unwrap();
    ///         for object in objects {
    ///             if let AiVisionObjectData::Color { position, .. } = object.data {
    ///                 println!("{:?}", position);
    ///             }
    ///         }
    ///         sleep(AiVisionSensor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn objects(&self) -> Result<Vec<AiVisionObject>> {
        let num_objects = self.object_count()?;

        let mut objects = Vec::new();
        for i in 0..num_objects {
            unsafe {
                let mut raw: V5_DeviceAiVisionObject = core::mem::zeroed();
                vexDeviceAiVisionObjectGet(self.device, i, core::ptr::from_mut(&mut raw));

                let object = match raw.r#type.into() {
                    ObjectType::Color => AiVisionObject::Color {
                        id: raw.id,
                        position: Point2 {
                            x: raw.object.color.xoffset,
                            y: raw.object.color.yoffset,
                        },
                        width: raw.object.color.width,
                        height: raw.object.color.height,
                    },
                    ObjectType::Code => AiVisionObject::Code {
                        id: raw.id,
                        position: Point2 {
                            x: raw.object.color.xoffset,
                            y: raw.object.color.yoffset,
                        },
                        width: raw.object.color.width,
                        height: raw.object.color.height,
                        angle: f64::from(raw.object.color.angle) / 10.0,
                    },
                    ObjectType::Model => AiVisionObject::Model {
                        id: raw.id,
                        classification: {
                            let ptr = CString::default().into_raw();

                            vexDeviceAiVisionClassNameGet(
                                self.device,
                                i32::from(raw.id),
                                ptr.cast(),
                            );

                            CString::from_raw(ptr).into_string()?
                        },
                        position: Point2 {
                            x: raw.object.model.xoffset,
                            y: raw.object.model.yoffset,
                        },
                        width: raw.object.model.width,
                        height: raw.object.model.height,
                        confidence: raw.object.model.score,
                    },
                    ObjectType::AprilTag => AiVisionObject::AprilTag {
                        id: raw.id,
                        top_left: mint::Point2 {
                            x: raw.object.tag.x0,
                            y: raw.object.tag.y0,
                        },
                        top_right: mint::Point2 {
                            x: raw.object.tag.x1,
                            y: raw.object.tag.y1,
                        },
                        bottom_right: mint::Point2 {
                            x: raw.object.tag.x2,
                            y: raw.object.tag.y2,
                        },
                        bottom_left: mint::Point2 {
                            x: raw.object.tag.x3,
                            y: raw.object.tag.y3,
                        },
                    },
                    _ => return Err(AiVisionError::InvalidObject),
                };

                objects.push(object);
            }
        }

        Ok(objects)
    }

    /// Returns the number of objects currently detected by the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected toMODE_MAGIC_BIT the Smart Port.
    ///
    /// # Examples
    ///
    /// ```
    /// use vexide::prelude::*;
    ///
    /// #[vexide::main]
    /// async fn main(peripherals: Peripherals) {
    ///     let mut ai_vision = AiVisionSensor::new(peripherals.port_1);
    ///     loop {
    ///         println!("AI Vision sensor currently detects {:?} objects", ai_vision.object_count());
    ///         sleep(AiVisionSensor::UPDATE_INTERVAL).await;
    ///     }
    /// }
    /// ```
    pub fn object_count(&self) -> Result<u32> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceAiVisionObjectCountGet(self.device) as _ })
    }
}

impl SmartDevice for AiVisionSensor {
    fn port_number(&self) -> u8 {
        self.port.number()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::AiVision
    }
}
impl From<AiVisionSensor> for SmartPort {
    fn from(val: AiVisionSensor) -> Self {
        val.port
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when using a vision sensor.
pub enum AiVisionError {
    /// An object created by VEXos failed to be converted.
    InvalidObject,
    /// Failed to fetch the class name of a model-detected object due it having a invalid
    /// string representation.
    #[snafu(transparent)]
    InvalidClassName {
        /// The source of the error.
        source: IntoStringError,
    },
    /// Generic port related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
