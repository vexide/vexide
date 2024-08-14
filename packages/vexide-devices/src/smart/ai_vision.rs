//! AI Vision sensor device.

use alloc::vec::Vec;
use core::mem;

use bitflags::bitflags;
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
        match value {
            0 => ObjectType::Unknown,
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
                        x_pos: data.xoffset,
                        y_pos: data.yoffset,
                        width: data.width,
                        height: data.height,
                        angle: data.angle as f64 / 10.0,
                    }
                }
                ObjectType::Model => {
                    let data = object.model;
                    AiVisionObjectData::Model {
                        x_pos: data.xoffset,
                        y_pos: data.yoffset,
                        width: data.width,
                        height: data.height,
                        confidence: data.score,
                    }
                }
                ObjectType::AprilTag => {
                    let data = object.tag;
                    AiVisionObjectData::AprilTag {
                        point1: mint::Point2 {
                            x: data.x0,
                            y: data.y0,
                        },
                        point2: mint::Point2 {
                            x: data.x1,
                            y: data.y1,
                        },
                        point3: mint::Point2 {
                            x: data.x2,
                            y: data.y2,
                        },
                        point4: mint::Point2 {
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
        /// The x position of the object.
        x_pos: u16,
        /// The y position of the object.
        y_pos: u16,

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
        point1: mint::Point2<i16>,
        point2: mint::Point2<i16>,
        point3: mint::Point2<i16>,
        point4: mint::Point2<i16>,
    },
    /// An object detected by an onboard model.
    Model {
        /// The x position of the object.
        x_pos: u16,
        /// The y position of the object.
        y_pos: u16,
        /// The width of the object.
        width: u16,
        /// The height of the object.
        height: u16,
        /// The confidence reported by the model.
        confidence: u16,
    },
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
/// Possible april tag families to be detected by the sensor.
pub enum AiVisionAprilTagFamily {
    /// Circle21h7 family
    Circle21h7 = 0,
    /// 16h5 family
    Tag16h5 = 1,
    /// 25h9 family
    Tag25h9 = 2,
    /// 36h11 family
    Tag36h11 = 3,
}

bitflags! {
    /// The mode of the AI Vision sensor.
    #[derive(Debug, Copy, Clone)]
    pub struct AiVisionDetectionMode: u32 {
        /// Objects will be detected by an onboard model
        const MODEL = 0b1;
        /// Color blobs will be detected
        const COLOR = 0b10;
        /// Color codes will be detected
        /// This only functions while color detection is enabled.
        const COLOR_CODE = 0b10000;
        /// April tags will be detected
        const APRIL_TAG = 0b100;
    }
}

const MODE_MAGIC_BIT: u32 = 0x20000000;

#[derive(Debug, Copy, Clone)]
/// A color signature used by an AI Vision Sensor to detect color blobs.
pub struct AiVisionColor {
    /// The red value of the color.
    pub red: u8,
    /// The green value of the color.
    pub green: u8,
    /// The blue value of the color.
    pub blue: u8,
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
    pub const fn new<const N: usize>(code: [Option<u8>; 7]) -> Self {
        Self(code)
    }

    /// Returns the color signature ids in the color code.
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

    /// Create a new AI Vision sensor from a smart port.
    pub fn new(port: SmartPort, brightness: f64, contrast: f64) -> Self {
        let device = unsafe { port.device_handle() };
        // Configure the AI Vision sensor with the given brightness and contrast.
        // SAFETY: The device handle is valid because it was created from a valid port.
        unsafe { vexDeviceAiVisionSensorSet(device, brightness, contrast) }
        Self {
            device,
            port,
            brightness,
            contrast,
        }
    }

    /// Returns the current temperature of the AI Vision sensor.
    pub fn temperature(&self) -> Result<f64> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceAiVisionTemperatureGet(self.device) })
    }

    /// Returns the contrast of the AI Vision sensor.
    /// # Note
    /// This method does not query the device for the current contrast.
    /// If the sensor is not connected, this function will not error.
    pub const fn contrast(&self) -> f64 {
        self.contrast
    }
    /// Sets the contrast of the AI Vision sensor.
    pub fn set_contrast(&mut self, contrast: f64) -> Result<()> {
        self.validate_port()?;
        self.contrast = contrast;
        unsafe { vexDeviceAiVisionSensorSet(self.device, self.brightness, contrast) }
        Ok(())
    }

    /// Returns the brightness of the AI Vision sensor.
    /// # Note
    /// This method does not query the device for the current brightness.
    /// If the sensor is not connected, this function will not error.
    pub const fn brightness(&self) -> f64 {
        self.brightness
    }
    /// Sets the brightness of the AI Vision sensor.
    pub fn set_brightness(&mut self, brightness: f64) -> Result<()> {
        self.validate_port()?;
        self.brightness = brightness;
        unsafe { vexDeviceAiVisionSensorSet(self.device, brightness, self.contrast) }
        Ok(())
    }

    /// Sets a color code used to detect groups of colors.
    /// # Note
    /// This function will return an error if the given ID is not in the range [1, 8].
    pub fn set_color_code(&mut self, id: u8, code: AiVisionColorCode) -> Result<()> {
        if !(1..=8).contains(&id) {
            return Err(AiVisionError::InvalidId);
        }
        self.validate_port()?;

        // Copy the color code into the V5_DeviceAiVisionCode struct
        let mut ids = [0u8; 7];
        for (i, id) in code.0.iter().flatten().enumerate() {
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
            c1: ids[0] as i16,
            c2: ids[1] as i16,
            c3: ids[2] as i16,
            c4: ids[3] as i16,
            c5: ids[4] as i16,
            c6: ids[5] as i16,
            c7: ids[6] as i16,
        };
        unsafe {
            vexDeviceAiVisionCodeSet(self.device, &mut code as *mut _);
        }

        Ok(())
    }

    /// Returns the color code set on the AI Vision sensor with the given ID if it exists.
    pub fn color_code(&self, id: u8) -> Result<Option<AiVisionColorCode>> {
        if !(1..=8).contains(&id) {
            return Err(AiVisionError::InvalidId);
        }
        self.validate_port()?;

        // Get the color code from the sensor
        let mut code: V5_DeviceAiVisionCode = unsafe { mem::zeroed() };
        let read = unsafe { vexDeviceAiVisionCodeGet(self.device, id as _, &mut code as *mut _) };
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
    /// # Note
    /// This function will return an error if the given ID is not in the range [1, 7].
    pub fn set_color(&mut self, id: u8, color: AiVisionColor) -> Result<()> {
        if !(1..=7).contains(&id) {
            return Err(AiVisionError::InvalidId);
        }
        self.validate_port()?;

        let mut color = V5_DeviceAiVisionColor {
            id,
            red: color.red,
            grn: color.green,
            blu: color.blue,
            hangle: color.hue,
            hdsat: color.saturation,
            reserved: 0,
        };

        //TODO: Make sure that the color is not modified by this function
        unsafe { vexDeviceAiVisionColorSet(self.device, &mut color as *mut _) }

        Ok(())
    }

    /// Returns the color signature set on the AI Vision sensor with the given ID if it exists.
    pub fn color(&self, id: u8) -> Result<Option<AiVisionColor>> {
        if !(1..=7).contains(&id) {
            return Err(AiVisionError::InvalidId);
        }
        self.validate_port()?;

        let mut color: V5_DeviceAiVisionColor = unsafe { mem::zeroed() };

        let read =
            unsafe { vexDeviceAiVisionColorGet(self.device, id as u32, &mut color as *mut _) };
        if !read {
            return Ok(None);
        }

        Ok(Some(AiVisionColor {
            red: color.red,
            green: color.grn,
            blue: color.blu,
            hue: color.hangle,
            saturation: color.hdsat,
        }))
    }

    /// Returns all color signatures set on the AI Vision sensor.
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
    /// # Note
    /// This function currently cannot detect if the sensor is in color code detection mode.
    pub fn detection_mode(&self) -> Result<AiVisionDetectionMode> {
        let status = self.status()?;
        let mode = status & 0b111;
        Ok(AiVisionDetectionMode::from_bits_truncate(mode))
    }
    /// Set the type of objects that will be detected
    pub fn set_detection_mode(&mut self, mode: AiVisionDetectionMode) -> Result<()> {
        // Mask out the current detection mode
        let mode_mask = 0b11111000;
        let current_mode = self.status()? & mode_mask as u32;

        let new_mode = current_mode | mode.bits();

        unsafe { vexDeviceAiVisionModeSet(self.device, new_mode | MODE_MAGIC_BIT) }

        Ok(())
    }

    /// Sets the family of apriltag that will be detected
    pub fn set_apriltag_family(&mut self, family: AiVisionAprilTagFamily) -> Result<()> {
        self.validate_port()?;

        let new_mode = (family as u32) << 16;

        //TODO: This should overwrite all of the other settings?
        //TODO: Testing required. This is what vexcode does...
        unsafe { vexDeviceAiVisionModeSet(self.device, new_mode | MODE_MAGIC_BIT) }
        Ok(())
    }

    /// Returns all objects detected by the AI Vision sensor.
    pub fn objects(&self) -> Result<Vec<AiVisionObject>> {
        let num_objects = self.num_objects()?;

        let mut objects = Vec::new();
        for i in 0..num_objects {
            unsafe {
                let mut object: V5_DeviceAiVisionObject = mem::zeroed();
                vexDeviceAiVisionObjectGet(self.device, i, &mut object as *mut _);
                let object = object.try_into()?;
                objects.push(object);
            }
        }

        Ok(objects)
    }

    /// Returns the number of objects currently detected by the AI Vision sensor.
    pub fn num_objects(&self) -> Result<u32> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceAiVisionObjectCountGet(self.device) as _ })
    }
}

impl SmartDevice for AiVisionSensor {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::AiVision
    }
}

#[derive(Debug, Snafu)]
/// Errors that can occur when using a vision sensor.
pub enum AiVisionError {
    /// An object created by VEXos failed to be converted.
    InvalidObject,
    /// The given signature ID or argument is out of range.
    InvalidId,
    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
