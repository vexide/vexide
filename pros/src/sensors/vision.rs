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

    pub fn nth_largest_object(&self, nth: u32) -> Result<VisionObject> {
        unsafe {
            let object = pros_sys::vision_get_by_size(self.port, nth);

            VisionError::from_last_errno()?;

            object
        }
    }
}

//TODO: figure out how coordinates are done.
pub struct VisionObject {
    pub top: u16,
    pub left: u16,
    pub middle_x: u16,
    pub middle_y: u16,


    pub width: u16,
    pub height: u16,

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
            _ => Ok(())
        }
    }
}