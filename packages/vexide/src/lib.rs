//! # vexide
//!
//! Open-source Rust runtime for VEX V5 robots. vexide provides a `no_std` Rust runtime,
//! async executor, device API, and more for the VEX V5 Brain!
//!
//! ## Getting Started
//!
//! If you're just getting started, we recommend going through our [docs](https://vexide.dev/docs/), which provide step-by-step instructions for setting up a development environment
//! and using vexide's common features.
//!
//! # Usage
//!
//! In order to get a program running, use the `#[vexide::main]` attribute on your main function.
//! ```rust
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main() {
//!     println!("Hello, world!");
//! }
//!```
//!
//! Check out our [examples](https://github.com/vexide/vexide/tree/main/examples/) for more examples of different features.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(html_logo_url = "https://vexide.dev/images/logo.svg")]

/// Async runtime & executor.
#[cfg(feature = "async")]
pub mod runtime {
    #[doc(inline)]
    pub use vexide_async::block_on;
}

#[doc(inline)]
#[cfg(feature = "async")]
pub use vexide_async::task;

/// Utilities for tracking time.
///
/// This module provides types for measuring time and executing code after a set periods
/// of time.
///
/// - [`Instant`] can measure execution time with high precision.
///
/// - [`Sleep`] is a future that does no work and completes at a specific [`Instant`]
///   in time.
///
/// - [`sleep`] and [`sleep_until`] provide ways to yield control away from a future
///   for or until a specific instant in time.
///
/// [`sleep`]: vexide_async::time::sleep
/// [`sleep_until`]: vexide_async::time::sleep_until
/// [`Instant`]: vexide_core::time::Instant
#[cfg(any(feature = "core", feature = "async"))]
pub mod time {
    #[doc(inline)]
    #[cfg(feature = "async")]
    pub use vexide_async::time::*;
    #[doc(inline)]
    #[cfg(feature = "core")]
    pub use vexide_core::time::*;
}

#[doc(inline)]
#[cfg(feature = "allocator")]
pub use vexide_core::allocator;
#[doc(inline)]
#[cfg(feature = "core")]
pub use vexide_core::{backtrace, competition, float, fs, io, os, path, program, sync};
#[doc(inline)]
#[cfg(feature = "devices")]
pub use vexide_devices as devices;
#[doc(inline)]
#[cfg(feature = "macro")]
pub use vexide_macro::main;
#[doc(inline)]
#[cfg(feature = "panic")]
pub use vexide_panic as panic;
#[doc(inline)]
#[cfg(feature = "startup")]
pub use vexide_startup as startup;

/// Commonly used features of vexide.
///
/// This module is meant to be glob imported.
pub mod prelude {
    #[cfg(feature = "devices")]
    pub use crate::devices::{
        adi::{
            accelerometer::{AdiAccelerometer, Sensitivity},
            addrled::AdiAddrLed,
            analog::AdiAnalogIn,
            digital::{AdiDigitalIn, AdiDigitalOut},
            encoder::AdiEncoder,
            gyroscope::AdiGyroscope,
            light_sensor::AdiLightSensor,
            line_tracker::AdiLineTracker,
            motor::AdiMotor,
            potentiometer::{AdiPotentiometer, PotentiometerType},
            pwm::AdiPwmOut,
            range_finder::AdiRangeFinder,
            servo::AdiServo,
            AdiDevice, AdiPort,
        },
        battery,
        controller::Controller,
        display::Display,
        peripherals::{DynamicPeripherals, Peripherals},
        position::Position,
        rgb::Rgb,
        smart::{
            ai_vision::{AiVisionColor, AiVisionColorCode, AiVisionObject, AiVisionSensor},
            distance::DistanceSensor,
            expander::AdiExpander,
            imu::InertialSensor,
            link::{LinkType, RadioLink},
            motor::{BrakeMode, Direction, Gearset, Motor, MotorControl},
            optical::OpticalSensor,
            rotation::RotationSensor,
            serial::SerialPort,
            vision::{
                LedMode, VisionCode, VisionMode, VisionObject, VisionSensor, VisionSignature,
                WhiteBalance,
            },
            SmartDevice, SmartPort,
        },
    };
    #[cfg(feature = "core")]
    pub use crate::{
        competition::{Compete, CompeteExt, CompetitionRuntime},
        float::Float,
        io::{dbg, print, println, BufRead, Read, Seek, Write},
    };
    #[cfg(feature = "async")]
    pub use crate::{
        runtime::block_on,
        task::{spawn, Task},
        time::{sleep, sleep_until},
    };
}
