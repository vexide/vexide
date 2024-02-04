//! Vision sensor device.
//!
//! Vision sensors take in a zero point at creation.

extern crate alloc;
use alloc::vec::Vec;

use pros_core::{bail_errno, bail_on, map_errno, error::PortError};
use pros_sys::{PROS_ERR, VISION_OBJECT_ERR_SIG};
use snafu::Snafu;

use super::{SmartDevice, SmartDeviceType, SmartPort};
use crate::color::Rgb;

/// Represents a vision sensor plugged into the vex.
#[derive(Debug, Eq, PartialEq)]
pub struct VisionSensor {
    port: SmartPort,
}

impl VisionSensor {
    /// Creates a new vision sensor.
    pub fn new(port: SmartPort, zero: VisionZeroPoint) -> Result<Self, VisionError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::vision_set_zero_point(port.index(), zero as _)
            );
        }

        Ok(Self { port })
    }

    /// Returns the nth largest object seen by the camera.
    pub fn nth_largest_object(&self, n: u32) -> Result<VisionObject, VisionError> {
        unsafe { pros_sys::vision_get_by_size(self.port.index(), n).try_into() }
    }

    /// Returns a list of all objects in order of size (largest to smallest).
    pub fn objects(&self) -> Result<Vec<VisionObject>, VisionError> {
        let obj_count = self.num_objects()?;
        let mut objects_buf = Vec::with_capacity(obj_count);

        unsafe {
            pros_sys::vision_read_by_size(
                self.port.index(),
                0,
                obj_count as _,
                objects_buf.as_mut_ptr(),
            );
        }

        bail_errno!();

        Ok(objects_buf
            .into_iter()
            .filter_map(|object| object.try_into().ok())
            .collect())
    }

    /// Returns the number of objects seen by the camera.
    pub fn num_objects(&self) -> Result<usize, PortError> {
        unsafe {
            Ok(bail_on!(
                PROS_ERR,
                pros_sys::vision_get_object_count(self.port.index())
            )
            .try_into()
            .unwrap())
        }
    }

    /// Get the current exposure percentage of the vision sensor. The returned result should be within 0.0 to 1.5.
    pub fn exposure(&self) -> f32 {
        unsafe { (pros_sys::vision_get_exposure(self.port.index()) as f32) * 1.5 / 150.0 }
    }

    /// Get the current white balance of the vision sensor.
    pub fn current_white_balance(&self) -> Rgb {
        unsafe { (pros_sys::vision_get_white_balance(self.port.index()) as u32).into() }
    }

    /// Sets the exposure percentage of the vision sensor. Should be between 0.0 and 1.5.
    pub fn set_exposure(&mut self, exposure: f32) {
        unsafe {
            pros_sys::vision_set_exposure(self.port.index(), (exposure * 150.0 / 1.5) as u8);
        }
    }

    /// Sets the white balance of the vision sensor.
    pub fn set_white_balance(&mut self, white_balance: WhiteBalance) {
        unsafe {
            match white_balance {
                WhiteBalance::Auto => pros_sys::vision_set_auto_white_balance(self.port.index(), 1),
                WhiteBalance::Rgb(rgb) => {
                    // Turn off automatic white balance
                    pros_sys::vision_set_auto_white_balance(self.port.index(), 0);
                    pros_sys::vision_set_white_balance(
                        self.port.index(),
                        <Rgb as Into<u32>>::into(rgb) as i32,
                    )
                }
            };
        }
    }

    /// Sets the point that object positions are relative to, in other words where (0, 0) is or the zero point.
    pub fn set_zero_point(&mut self, zero: VisionZeroPoint) {
        unsafe {
            pros_sys::vision_set_zero_point(self.port.index(), zero as _);
        }
    }

    /// Sets the color of the led.
    pub fn set_led(&mut self, mode: LedMode) {
        unsafe {
            match mode {
                LedMode::Off => pros_sys::vision_clear_led(self.port.index()),
                LedMode::On(rgb) => pros_sys::vision_set_led(
                    self.port.index(),
                    <Rgb as Into<u32>>::into(rgb) as i32,
                ),
            };
        }
    }
}

impl SmartDevice for VisionSensor {
    fn port_index(&self) -> u8 {
        self.port.index()
    }

    fn device_type(&self) -> SmartDeviceType {
        SmartDeviceType::Vision
    }
}

//TODO: figure out how coordinates are done.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
/// An object detected by the vision sensor
pub struct VisionObject {
    /// The offset from the top of the object to the vision center.
    pub top: i16,
    /// The offset from the left of the object to the vision center.
    pub left: i16,
    /// The x-coordinate of the middle of the object relative to the vision center.
    pub middle_x: i16,
    /// The y-coordinate of the middle of the object relative to the vision center.
    pub middle_y: i16,

    /// The width of the object.
    pub width: i16,
    /// The height of the object.
    pub height: i16,
}

impl TryFrom<pros_sys::vision_object_s_t> for VisionObject {
    type Error = VisionError;
    fn try_from(value: pros_sys::vision_object_s_t) -> Result<VisionObject, VisionError> {
        if value.signature == VISION_OBJECT_ERR_SIG {
            bail_errno!();
            unreachable!("Errno should be non-zero")
        }

        Ok(Self {
            top: value.top_coord,
            left: value.left_coord,
            middle_x: value.x_middle_coord,
            middle_y: value.y_middle_coord,
            width: value.width,
            height: value.height,
        })
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The zero point of the vision sensor.
/// Vision object coordinates are relative to this point.
pub enum VisionZeroPoint {
    /// The zero point will be the top left corner of the vision sensor.
    TopLeft,
    /// The zero point will be the top right corner of the vision sensor.
    Center,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The white balance of the vision sensor.
pub enum WhiteBalance {
    /// Provide a specific color to balance the white balance.
    Rgb(Rgb),
    /// Automatically balance the white balance.
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The mode of the vision sensor led.
pub enum LedMode {
    /// Turn on the led with a certain color.
    On(Rgb),
    /// Turn off the led.
    Off,
}

#[derive(Debug, Snafu)]
/// Errors that can occur when using a vision sensor.
pub enum VisionError {
    /// The camera could not be read.
    ReadingFailed,
    /// The index specified was higher than the total number of objects seen by the camera.
    IndexTooHigh,
    /// Port already taken.
    PortTaken,
    #[snafu(display("{source}"), context(false))]
    /// Generic port related error.
    Port {
        /// The source of the error.
        source: PortError,
    },
}

map_errno! {
    VisionError {
        EHOSTDOWN => Self::ReadingFailed,
        EDOM => Self::IndexTooHigh,
        EACCES => Self::PortTaken,
    }
    inherit PortError;
}
