//! # vexide
//!
//! Open-source Rust runtime for VEX V5 robots. vexide provides a `no_std` Rust runtime,
//! async executor, device API, and more for the VEX V5 Brain!
//!
//! ## Getting Started
//!
//! If you're just getting started, we recommend going through our [docs](https://vexide.dev/docs/),
//! which provide step-by-step instructions for setting up a development environment
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
//! Check out our [examples](https://github.com/vexide/vexide/tree/main/examples/) for more examples
//! of different features.

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(html_logo_url = "https://vexide.dev/images/logo.svg")]

/// Async runtime & executor.
#[cfg(feature = "rt")]
pub mod runtime {
    #[doc(inline)]
    pub use vexide_runtime::block_on;
}

#[doc(inline)]
#[cfg(feature = "rt")]
pub use vexide_runtime::{task, time, backtrace, competition, os, sync, banner};
#[doc(inline)]
#[cfg(feature = "devices")]
pub use vexide_devices as devices;
#[doc(inline)]
#[cfg(feature = "macro")]
pub use vexide_macro::main;
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
            encoder::{AdiEncoder, AdiOpticalEncoder},
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
    #[cfg(feature = "rt")]
    pub use crate::{
        competition::{Compete, CompeteExt, CompetitionRuntime},
        runtime::block_on,
        task::{spawn, Task},
        time::{sleep, sleep_until},
    };
}
