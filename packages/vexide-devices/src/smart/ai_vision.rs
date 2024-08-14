//! AI Vision sensor device.

use alloc::vec::Vec;
use core::mem;

use snafu::Snafu;
use vex_sdk::{
    vexDeviceAiVisionCodeSet, vexDeviceAiVisionColorSet, vexDeviceAiVisionModeSet, vexDeviceAiVisionObjectCountGet, vexDeviceAiVisionObjectGet, vexDeviceAiVisionSensorSet, vexDeviceAiVisionTemperatureGet, V5_DeviceAiVisionColor, V5_DeviceAiVisionObject, V5_DeviceT
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
    Color {
        x_pos: u16,
        y_pos: u16,

        width: u16,
        height: u16,
        angle: f64,
    },
    AprilTag {
        point1: mint::Point2<i16>,
        point2: mint::Point2<i16>,
        point3: mint::Point2<i16>,
        point4: mint::Point2<i16>,
    },
    Model {
        x_pos: u16,
        y_pos: u16,
        width: u16,
        height: u16,
        confidence: u16,
    },
}

/// The mode of the AI Vision sensor.
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum AiVisionMode {
    Code = 0,
    Model = 1,
    AprilTag = 2,
}

#[derive(Debug, Copy, Clone)]
pub struct AiVisionColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub hue: f32,
    pub saturation: f32,
}

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

    pub fn set_color(&mut self, id: u8, color: AiVisionColor) -> Result<()> {
        if !(1..7).contains(&id) {
            return Err(AiVisionError::InvalidId);
        }

        let mut color = V5_DeviceAiVisionColor  {
            id,
            red: color.red,
            grn: color.green,
            blu: color.blue,
            hangle: color.hue,
            hdsat: color.saturation,
            reserved: 0,
        };

        //TODO: Make sure that the color is not modified by this function
        unsafe {
            vexDeviceAiVisionColorSet(self.device, &mut color as *mut _)
        }

        self.validate_port()?;

        Ok(())
    }

    pub fn set_mode(&mut self, mode: AiVisionMode) -> Result<()> {
        self.validate_port()?;
        unsafe { vexDeviceAiVisionModeSet(self.device, mode as u32) }

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
