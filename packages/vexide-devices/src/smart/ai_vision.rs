//! AI Vision sensor device.

use alloc::vec::Vec;
use core::mem;

use bitflags::bitflags;
use mint::Point2;
use rgb::Rgb;
use snafu::Snafu;
use vex_sdk::{
    vexDeviceAiVisionCodeGet, vexDeviceAiVisionCodeSet, vexDeviceAiVisionColorGet,
    vexDeviceAiVisionColorSet, vexDeviceAiVisionModeSet, vexDeviceAiVisionObjectCountGet,
    vexDeviceAiVisionObjectGet, vexDeviceAiVisionSensorSet, vexDeviceAiVisionStatusGet,
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

/// An object detected by the AI Vision sensor.
pub struct AiVisionObject {
    /// The ID of the object.
    pub id: u8,
    /// The data associated with the object.
    pub data: AiVisionObjectData,
}

impl TryFrom<V5_DeviceAiVisionObject> for AiVisionObject {
    type Error = AiVisionError;

    fn try_from(value: V5_DeviceAiVisionObject) -> Result<Self, Self::Error> {
        let object = value.object;
        let id = value.id;
        let data = unsafe {
            match id.into() {
                ObjectType::Color => {
                    let data = object.color;
                    AiVisionObjectData::Color {
                        position: Point2 {
                            x: data.xoffset,
                            y: data.yoffset,
                        },
                        width: data.width,
                        height: data.height,
                        angle: f64::from(data.angle) / 10.0,
                    }
                }
                ObjectType::Model => {
                    let data = object.model;
                    AiVisionObjectData::Model {
                        position: Point2 {
                            x: data.xoffset,
                            y: data.yoffset,
                        },
                        width: data.width,
                        height: data.height,
                        confidence: data.score,
                    }
                }
                ObjectType::AprilTag => {
                    let data = object.tag;
                    AiVisionObjectData::AprilTag {
                        corner_1: mint::Point2 {
                            x: data.x0,
                            y: data.y0,
                        },
                        corner_2: mint::Point2 {
                            x: data.x1,
                            y: data.y1,
                        },
                        corner_3: mint::Point2 {
                            x: data.x2,
                            y: data.y2,
                        },
                        corner_4: mint::Point2 {
                            x: data.x3,
                            y: data.y3,
                        },
                    }
                }
                _ => return Err(AiVisionError::InvalidObject),
            }
        };

        Ok(Self { id, data })
    }
}

/// The data associated with an AI Vision object.
/// The data is different depending on the type of object detected.
pub enum AiVisionObjectData {
    /// An object detected by color blob detection.
    Color {
        /// The position of the object.
        position: Point2<u16>,
        /// The width of the object.
        width: u16,
        /// The height of the object.
        height: u16,
        /// The angle of the object.
        angle: f64,
    },
    /// An object detected by apriltag detection.
    AprilTag {
        //TODO: figure out what corners these points represent
        /// Tag Corner 1
        corner_1: mint::Point2<i16>,
        /// Tag Corner 2
        corner_2: mint::Point2<i16>,
        /// Tag Corner 3
        corner_3: mint::Point2<i16>,
        /// Tag Corner 4
        corner_4: mint::Point2<i16>,
    },
    /// An object detected by an onboard model.
    Model {
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
#[derive(Default, Debug, Copy, Clone)]
pub enum AprilTagFamily {
    /// Circle21h7 family
    #[default]
    Circle21h7,
    /// 16h5 family
    Tag16h5,
    /// 25h9 family
    Tag25h9,
    /// 36h11 family
    Tag36h11,
    /// Sensor is in test mode
    TestMode(u8),
}

bitflags! {
    /// Flags relating to the sensor's detection mode.
    #[derive(Default, Debug, Copy, Clone)]
    pub struct AiVisionMode: u8 {
        /// Disable model detection
        const DISABLE_MODEL = 1 << 2;

        /// Disable color detection
        const DISABLE_COLOR = 1 << 1;

        /// Disable apriltag detection
        const DISABLE_APRILTAG = 1 << 0;

        /// Merge color blobs?
        const COLOR_MERGE = 1 << 4;

        /// Disable USB overlay
        const DISABLE_USB_OVERLAY = 1 << 7;
    }
}

#[derive(Debug, Copy, Clone)]
/// Represents the state of the AI Vision sensor's USB overlay.
pub enum AiVisionUsbOverlay {
    /// The USB overlay is enabled.
    Enabled,
    /// The USB overlay is disabled.
    Disabled,
}

#[derive(Debug, Copy, Clone)]
/// A color signature used by an AI Vision Sensor to detect color blobs.
pub struct AiVisionColor {
    /// The RGB color value.
    pub rgb: Rgb<u8>,
    /// The accepted hue range of the color. VEXcode limits this value to [0, 20]
    pub hue: f32,
    /// The accepted saturation range of the color.
    pub saturation: f32,
}

/// A color code used by an AI Vision Sensor to detect groups of color blobs.
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
    brightness: f64,
    contrast: f64,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for AiVisionSensor {}
unsafe impl Sync for AiVisionSensor {}

impl AiVisionSensor {
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
    const UPDATE_FLAG: u32 = (1 << 25);
    const TEST_MODE_FLAG: u32 = (1 << 26);

    /// Create a new AI Vision sensor from a smart port.
    #[must_use]
    pub fn new(port: SmartPort, brightness: f64, contrast: f64) -> Self {
        let device = unsafe { port.device_handle() };
        // Configure the AI Vision sensor with the given brightness and contrast.
        // SAFETY: The device handle is valid because it was created from a valid port.
        unsafe { vexDeviceAiVisionSensorSet(device, brightness, contrast) }
        let mut this = Self {
            port,
            device,
            brightness,
            contrast,
        };
        unsafe {
            vexDeviceAiVisionModeSet(device, Self::RESET_FLAG);
        }
        let _ = this.set_mode(AiVisionMode::DISABLE_USB_OVERLAY);
        this
    }

    /// Returns the current temperature of the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn temperature(&self) -> Result<f64> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceAiVisionTemperatureGet(self.device) })
    }

    /// Returns the contrast of the AI Vision sensor.
    ///
    /// # Note
    ///
    /// This method does not query the device for the current contrast.
    /// If the sensor is not connected, this function will not error.
    #[must_use]
    pub const fn contrast(&self) -> f64 {
        self.contrast
    }
    /// Sets the contrast of the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn set_contrast(&mut self, contrast: f64) -> Result<()> {
        self.validate_port()?;
        self.contrast = contrast;
        unsafe { vexDeviceAiVisionSensorSet(self.device, self.brightness, contrast) }
        Ok(())
    }

    /// Returns the brightness of the AI Vision sensor.
    ///
    /// # Note
    ///
    /// This method does not query the device for the current brightness.
    /// If the sensor is not connected, this function will not error.
    #[must_use]
    pub const fn brightness(&self) -> f64 {
        self.brightness
    }
    /// Sets the brightness of the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn set_brightness(&mut self, brightness: f64) -> Result<()> {
        self.validate_port()?;
        self.brightness = brightness;
        unsafe { vexDeviceAiVisionSensorSet(self.device, brightness, self.contrast) }
        Ok(())
    }

    /// Sets a color code used to detect groups of colors.
    ///
    /// # Note
    ///
    /// This function will return an error if the given ID is not in the range [1, 8].
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    /// - A [`AiVisionError::InvalidId`] is returned if the given ID is not in the range [1, 8].
    /// - A [`AiVisionError::InvalidIdInCode`] is returned if the given color code contains an ID that is not in the range [1, 7].
    pub fn set_color_code(&mut self, id: u8, code: &AiVisionColorCode) -> Result<()> {
        if !(1..=8).contains(&id) {
            return InvalidIdSnafu { id, range: 1..=8 }.fail();
        }
        self.validate_port()?;

        // Copy the color code into the V5_DeviceAiVisionCode struct
        let mut ids = [0u8; 7];
        for (i, id) in code.0.iter().flatten().enumerate() {
            if !(1..=7).contains(id) {
                return InvalidIdInCodeSnafu { id: *id }.fail();
            }
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
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    /// - A [`AiVisionError::InvalidId`] is returned if the given ID is not in the range [1, 8].
    pub fn color_code(&self, id: u8) -> Result<Option<AiVisionColorCode>> {
        if !(1..=8).contains(&id) {
            return InvalidIdSnafu { id, range: 1..=8 }.fail();
        }
        self.validate_port()?;

        // Get the color code from the sensor
        let mut code: V5_DeviceAiVisionCode = unsafe { mem::zeroed() };
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
    /// # Note
    ///
    /// This function will return an error if the given ID is not in the range [1, 7].
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    /// - A [`AiVisionError::InvalidId`] is returned if the given ID is not in the range [1, 7].
    pub fn set_color(&mut self, id: u8, color: AiVisionColor) -> Result<()> {
        if !(1..=7).contains(&id) {
            return InvalidIdSnafu { id, range: 1..=7 }.fail();
        }
        self.validate_port()?;

        let mut color = V5_DeviceAiVisionColor {
            id,
            red: color.rgb.r,
            grn: color.rgb.g,
            blu: color.rgb.b,
            hangle: color.hue,
            hdsat: color.saturation,
            reserved: 0,
        };

        //TODO: Make sure that the color is not modified by this function
        unsafe { vexDeviceAiVisionColorSet(self.device, core::ptr::from_mut(&mut color)) }

        Ok(())
    }

    /// Returns the color signature set on the AI Vision sensor with the given ID if it exists.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    /// - A [`AiVisionError::InvalidId`] is returned if the given ID is not in the range [1, 7].
    pub fn color(&self, id: u8) -> Result<Option<AiVisionColor>> {
        if !(1..=7).contains(&id) {
            return InvalidIdSnafu { id, range: 1..=7 }.fail();
        }
        self.validate_port()?;

        let mut color: V5_DeviceAiVisionColor = unsafe { mem::zeroed() };

        let read = unsafe {
            vexDeviceAiVisionColorGet(self.device, u32::from(id), core::ptr::from_mut(&mut color))
        };
        if !read {
            return Ok(None);
        }

        Ok(Some(AiVisionColor {
            rgb: Rgb::new(color.red, color.grn, color.blu),
            hue: color.hangle,
            saturation: color.hdsat,
        }))
    }

    /// Returns all color signatures set on the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
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

    fn status(&self) -> Result<u32> {
        self.validate_port()?;
        let status = unsafe { vexDeviceAiVisionStatusGet(self.device) };
        Ok(status)
    }

    /// Returns the current detection mode of the AI Vision sensor.
    ///
    /// # Note
    ///
    /// This function currently cannot detect if the sensor is in color code detection mode.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn mode(&self) -> Result<AiVisionMode> {
        // Only care about the first byte of status.
        // See https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c988c99e1f9b3a6d3c3fd91591b6dac1
        Ok(AiVisionMode::from_bits_retain(
            (self.status()? & 0xff) as u8,
        ))
    }

    /// Set the type of objects that will be detected
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn set_mode(&mut self, mode: AiVisionMode) -> Result<()> {
        // Status is shifted to the right from mode. Least-significant byte is missing.
        // See https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c988c99e1f9b3a6d3c3fd91591b6dac1
        let mut new_mode = self.status()? << 8;

        new_mode &= !(0xff << 8); // Clear the mode bits.
        new_mode |= (u32::from(mode.bits()) << 8) | Self::UPDATE_FLAG; // Set the mode bits and set the UPDATE flag in StateFlags.

        // Update mode
        unsafe { vexDeviceAiVisionModeSet(self.device, new_mode) }

        Ok(())
    }

    /// Sets the family of apriltag that will be detected
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn set_apriltag_family(&mut self, family: AprilTagFamily) -> Result<()> {
        // Status is shifted to the right from mode. Least-significant byte is missing.
        // See https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c988c99e1f9b3a6d3c3fd91591b6dac1
        let mut new_mode = self.status()? << 8;

        new_mode &= !(0xff << 16); // Clear the existing apriltag family bits.
        new_mode |= u32::from(match family {
            AprilTagFamily::Circle21h7 => 0,
            AprilTagFamily::Tag16h5 => 1,
            AprilTagFamily::Tag25h9 => 2,
            AprilTagFamily::Tag36h11 => 3,
            AprilTagFamily::TestMode(value) => value,
        }) << 16; // Set family bits

        new_mode |= if matches!(family, AprilTagFamily::TestMode(_)) {
            // Set TEST_MODE flag
            Self::TEST_MODE_FLAG
        } else {
            // Set UPDATE flag
            Self::UPDATE_FLAG
        };

        // Update mode
        unsafe { vexDeviceAiVisionModeSet(self.device, new_mode) }

        Ok(())
    }

    /// Returns the family of apriltag that will be detected
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn apriltag_family(&mut self) -> Result<AprilTagFamily> {
        let status = self.status()?;
        let is_test_mode = ((status << 8) & Self::TEST_MODE_FLAG) == Self::TEST_MODE_FLAG;
        let family_byte = (status >> 8 & 0xff) as u8;

        Ok(if is_test_mode {
            AprilTagFamily::TestMode(family_byte)
        } else {
            match family_byte {
                0 => AprilTagFamily::Circle21h7,
                1 => AprilTagFamily::Tag16h5,
                2 => AprilTagFamily::Tag25h9,
                3 => AprilTagFamily::Tag36h11,
                // Probably unreachable
                other => AprilTagFamily::TestMode(other),
            }
        })
    }

    /// Returns all objects detected by the AI Vision sensor.
    ///
    /// # Errors
    ///
    /// - A [`PortError`] is returned if an AI Vision is not connected to the Smart Port.
    pub fn objects(&self) -> Result<Vec<AiVisionObject>> {
        let num_objects = self.object_count()?;

        let mut objects = Vec::new();
        for i in 0..num_objects {
            unsafe {
                let mut object: V5_DeviceAiVisionObject = mem::zeroed();
                vexDeviceAiVisionObjectGet(self.device, i, core::ptr::from_mut(&mut object));
                let object = object.try_into()?;
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
    /// The given signature ID or argument is out of range.
    #[snafu(display("The given ID ({id}) is out of the range {range:?}."))]
    InvalidId {
        /// The ID that was out of range.
        id: u8,
        /// The range of possible values for the ID.
        range: core::ops::RangeInclusive<u8>,
    },
    /// A color signature ID in a given color code is out of range.
    #[snafu(display("The given color code contains an ID ({id}) that is out of range."))]
    InvalidIdInCode {
        /// The ID that was out of range.
        id: u8,
    },
    /// Generic port related error.
    #[snafu(transparent)]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
