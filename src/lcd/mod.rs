#[macro_use]
pub mod macros;
pub mod buttons;

pub(crate) mod writer;

lazy_static::lazy_static! {
    pub(crate) static ref WRITER: spin::Mutex<writer::Writer> = spin::Mutex::new(writer::Writer::new());
}
