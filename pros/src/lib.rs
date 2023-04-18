#![no_std]

pub mod motor;
pub mod multitasking;

#[cfg(not(feature = "lvgl"))]
#[macro_use]
pub mod lcd;

#[cfg(feature = "lvgl")]
#[macro_use]
pub mod lvgl;

pub(crate) mod errno;
