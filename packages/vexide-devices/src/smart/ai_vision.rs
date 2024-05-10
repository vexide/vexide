use alloc::vec::Vec;
use core::mem;

use snafu::Snafu;
use vex_sdk::{
    vexDeviceAiVisionObjectCountGet, vexDeviceAiVisionObjectGet, vexDeviceAiVisionTemperatureGet,
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

pub enum AiVisionObject {
    Color {
        id: u8,
        x_pos: u16,
        y_pos: u16,

        width: u16,
        height: u16,
        angle: f64,
    },
    AprilTag {
        id: u8,
        point1: mint::Point2<i16>,
        point2: mint::Point2<i16>,
        point3: mint::Point2<i16>,
        point4: mint::Point2<i16>,
    },
    Model {
        id: u8,
        x_pos: u16,
        y_pos: u16,
        width: u16,
        height: u16,
        confidence: u16,
    },
}
impl TryFrom<V5_DeviceAiVisionObject> for AiVisionObject {
    type Error = AiVisionError;

    fn try_from(value: V5_DeviceAiVisionObject) -> Result<Self, Self::Error> {
        let data = value.object;
        unsafe {
            match value.r#type.into() {
                ObjectType::Color => Ok(AiVisionObject::Color {
                    id: value.id,
                    x_pos: data.color.xoffset,
                    y_pos: data.color.yoffset,
                    width: data.color.width,
                    height: data.color.height,
                    angle: data.color.angle as f64 / 10.0,
                }),
                ObjectType::Model => Ok(AiVisionObject::Model {
                    id: value.id,
                    x_pos: data.model.xoffset,
                    y_pos: data.model.yoffset,
                    width: data.model.width,
                    height: data.model.height,
                    confidence: data.model.score,
                }),
                ObjectType::AprilTag => Ok(AiVisionObject::AprilTag {
                    id: value.id,
                    point1: mint::Point2 {
                        x: data.tag.x0,
                        y: data.tag.y0,
                    },
                    point2: mint::Point2 {
                        x: data.tag.x1,
                        y: data.tag.y1,
                    },
                    point3: mint::Point2 {
                        x: data.tag.x2,
                        y: data.tag.y2,
                    },
                    point4: mint::Point2 {
                        x: data.tag.x3,
                        y: data.tag.y3,
                    },
                }),
                _ => Err(AiVisionError::InvalidObject),
            }
        }
    }
}

pub struct AiVisionSensor {
    port: SmartPort,
    device: V5_DeviceT,
}

// SAFETY: Required because we store a raw pointer to the device handle to avoid it getting from the
// SDK each device function. Simply sharing a raw pointer across threads is not inherently unsafe.
unsafe impl Send for AiVisionSensor {}
unsafe impl Sync for AiVisionSensor {}

impl AiVisionSensor {
    pub fn new(port: SmartPort) -> Self {
        Self {
            device: unsafe { port.device_handle() },
            port,
        }
    }

    pub fn temperature(&self) -> Result<f64> {
        self.validate_port()?;
        Ok(unsafe { vexDeviceAiVisionTemperatureGet(self.device) })
    }

    pub fn set_mode(&mut self) -> Result<()> {
        self.validate_port()?;
        todo!()
    }

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
    /// Generic port related error.
    #[snafu(display("{source}"), context(false))]
    Port {
        /// The source of the error.
        source: PortError,
    },
}
