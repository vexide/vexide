use crate::multitasking::mutex::Mutex;

#[macro_use]
pub mod macros;
pub mod buttons;

pub(crate) mod writer;

lazy_static::lazy_static! {
    pub(crate) static ref WRITER: Mutex<writer::Writer> = {
        unsafe { pros_sys::lcd_initialize() };
        Mutex::new(writer::Writer::new())
    };
}
