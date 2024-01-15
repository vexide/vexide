//! Vision sensor device.
//!
//! Vision sensors take in a zero point at creation.

extern crate alloc;
use alloc::vec::Vec;

use pros_sys::{PROS_ERR, VISION_OBJECT_ERR_SIG};
use snafu::Snafu;

use crate::{
    error::{bail_errno, bail_on, map_errno, PortError},
    lvgl::colors::LcdColor,
};

/// Represents a vision sensor plugged into the vex.
pub struct VisionSensor {
    port: u8,
}

impl VisionSensor {
    /// Creates a new vision sensor.
    pub fn new(port: u8, zero: VisionZeroPoint) -> Result<Self, crate::error::PortError> {
        unsafe {
            bail_on!(PROS_ERR, pros_sys::vision_set_zero_point(port, zero as _));
        }

        Ok(Self { port })
    }

    /// Returns the nth largest object seen by the camera.
    pub fn nth_largest_object(&self, n: u32) -> Result<VisionObject, VisionError> {
        unsafe { pros_sys::vision_get_by_size(self.port, n).try_into() }
    }

    /// Returns a list of all objects in order of size (largest to smallest).
    pub fn objects(&self) -> Result<Vec<VisionObject>, VisionError> {
        let obj_count = self.num_objects()?;
        let mut objects_buf = Vec::with_capacity(obj_count);

        unsafe {
            pros_sys::vision_read_by_size(self.port, 0, obj_count as _, objects_buf.as_mut_ptr());
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
            Ok(
                bail_on!(PROS_ERR, pros_sys::vision_get_object_count(self.port))
                    .try_into()
                    .unwrap(),
            )
        }
    }

    /// Get the current exposure percentage of the vision sensor. The returned result should be within 0.0 to 1.5.
    pub fn exposure(&self) -> f32 {
        unsafe { (pros_sys::vision_get_exposure(self.port) as f32) * 1.5 / 150.0 }
    }

    /// Get the current white balance of the vision sensor.
    pub fn current_white_balance(&self) -> Rgb {
        unsafe { (pros_sys::vision_get_white_balance(self.port) as u32).into() }
    }

    /// Sets the exposure percentage of the vision sensor. Should be between 0.0 and 1.5.
    pub fn set_exposure(&mut self, exposure: f32) {
        unsafe {
            pros_sys::vision_set_exposure(self.port, (exposure * 150.0 / 1.5) as u8);
        }
    }

    /// Sets the white balance of the vision sensor.
    pub fn set_white_balance(&mut self, white_balance: WhiteBalance) {
        unsafe {
            match white_balance {
                WhiteBalance::Auto => pros_sys::vision_set_auto_white_balance(self.port, 1),
                WhiteBalance::Rgb(rgb) => {
                    // Turn off automatic white balance
                    pros_sys::vision_set_auto_white_balance(self.port, 0);
                    pros_sys::vision_set_white_balance(
                        self.port,
                        <Rgb as Into<u32>>::into(rgb) as i32,
                    )
                }
            };
        }
    }

    /// Sets the point that object positions are relative to, in other words where (0, 0) is or the zero point.
    pub fn set_zero_point(&mut self, zero: VisionZeroPoint) {
        unsafe {
            pros_sys::vision_set_zero_point(self.port, zero as _);
        }
    }

    /// Sets the color of the led.
    pub fn set_led(&mut self, mode: LedMode) {
        unsafe {
            match mode {
                LedMode::Off => pros_sys::vision_clear_led(self.port),
                LedMode::On(rgb) => {
                    pros_sys::vision_set_led(self.port, <Rgb as Into<u32>>::into(rgb) as i32)
                }
            };
        }
    }
}

//TODO: figure out how coordinates are done.
#[derive(Debug)]
pub struct VisionObject {
    pub top: i16,
    pub left: i16,
    pub middle_x: i16,
    pub middle_y: i16,

    pub width: i16,
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

pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for u32 {
    fn from(other: Rgb) -> u32 {
        ((other.r as u32) << 16) + ((other.g as u32) << 8) + other.b as u32
    }
}

const BITMASK: u32 = 0b11111111;

impl From<u32> for Rgb {
    fn from(value: u32) -> Self {
        Self {
            r: ((value >> 16) & BITMASK) as _,
            g: ((value >> 8) & BITMASK) as _,
            b: (value & BITMASK) as _,
        }
    }
}

impl From<Rgb> for LcdColor {
    fn from(other: Rgb) -> Self {
        Self(pros_sys::lv_color_t {
            red: other.r,
            green: other.g,
            blue: other.b,
            alpha: 0xFF,
        })
    }
}

impl From<LcdColor> for Rgb {
    fn from(other: LcdColor) -> Self {
        Self {
            r: other.red,
            g: other.green,
            b: other.blue,
        }
    }
}

#[repr(u32)]
pub enum VisionZeroPoint {
    TopLeft,
    Center,
}

pub enum WhiteBalance {
    Rgb(Rgb),
    Auto,
}

pub enum LedMode {
    On(Rgb),
    Off,
}

#[derive(Debug, Snafu)]
pub enum VisionError {
    #[snafu(display(
        "The index specified was higher than the total number of objects seen by the camera."
    ))]
    ReadingFailed,
    #[snafu(display("The camera could not be read."))]
    IndexTooHigh,
    #[snafu(display("Port already taken."))]
    PortTaken,
    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}

map_errno! {
    VisionError {
        EHOSTDOWN => Self::ReadingFailed,
        EDOM => Self::IndexTooHigh,
        EACCES => Self::PortTaken,
    }
    inherit PortError;
}
