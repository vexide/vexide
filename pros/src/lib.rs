#![no_std]

pub mod controller;
pub mod motor;
pub mod multitasking;
pub mod sensors;

#[cfg(not(feature = "lvgl"))]
#[macro_use]
pub mod lcd;

#[cfg(feature = "lvgl")]
#[macro_use]
pub mod lvgl;

pub(crate) mod errno;
