use crate::error::FromErrno;

pub struct VisionSensor {
    port: u8,
}

impl VisionSensor {
    pub fn new(port: u8) -> Result<Self, crate::error::PortError> {
        unsafe {
            pros_sys::vision_get_by_size(port, 0);

            crate::error::PortError::from_last_errno()?;
        }

        Ok(Self { port })
    }

    pub fn nth_largest_object(&self, nth: u32) -> Result<VisionObject, VisionError> {
        unsafe {
            let object = pros_sys::vision_get_by_size(self.port, nth).into();

            VisionError::from_last_errno()?;

            Ok(object)
        }
    }
}

//TODO: figure out how coordinates are done.
pub struct VisionObject {
    pub top: i16,
    pub left: i16,
    pub middle_x: i16,
    pub middle_y: i16,

    pub width: i16,
    pub height: i16,
}

impl From<pros_sys::vision_object> for VisionObject {
    fn from(value: pros_sys::vision_object) -> Self {
        Self {
            top: value.top_coord,
            left: value.left_coord,
            middle_x: value.x_middle_coord,
            middle_y: value.y_middle_coord,
            width: value.width,
            height: value.height,
        }
    }
}

pub enum VisionError {
    ReadingFailed,
    IndexTooHigh,
}

impl crate::error::FromErrno for VisionError {
    fn from_last_errno() -> Result<(), Self> {
        match unsafe { crate::errno::ERRNO.get() } {
            pros_sys::EHOSTDOWN => Err(Self::ReadingFailed),
            pros_sys::EDOM => Err(Self::IndexTooHigh),
            _ => Ok(()),
        }
    }
}
