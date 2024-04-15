//! # vexide
//! Work in progress high level bindings for the V5 Brain VEX SDK.
//! Unlike other libraries for the V5 Brain, vexide doesn't use an RTOS.
//! Instead, vexide leverages Rust's powerful async/await (cooperative multitasking) to allow faster and more user friendly 3dfcode.
//!
//! Advantages over similar libraries like PROS include:
//! - vexide is a real crate on crates.io instead of a template, or similar. This allows for dependency management with cargo.
//! - vexide has an [`Async executor`](async_runtime) which allows for easy and performant asynchronous code.
//! - Active development. vexide is actively developed and maintained.
//! - vexide is a real crate on crates.io instead of a template, or similar. This allows for dependency management with cargo.
//! - vexide has no external dependencies. It's 100% Rust and every line of code is yours to see.
//! - vexide produces tiny and fast binaries.
//!
//! # Usage
//!
//! In order to get a program running, use the `#[vexide::main]` attribute on your main function.
//! ```rust
//! // Async
//! use vexide::prelude::*;
//! #[vexide::main]
//! async fn main() {
//!     println!("Hello, world!");
//! }
//!```
#![no_std]

#[cfg(feature = "async")]
pub use vexide_async as async_runtime;
#[cfg(feature = "core")]
pub use vexide_core as core;
#[cfg(feature = "devices")]
pub use vexide_devices as devices;
#[cfg(feature = "macro")]
pub use vexide_macro as r#macro;
#[cfg(feature = "macro")]
pub use vexide_macro::main;
#[cfg(feature = "math")]
pub use vexide_math as math;
#[cfg(feature = "panic")]
pub use vexide_panic as panic;
#[cfg(feature = "startup")]
pub use vexide_startup as startup;

/// Commonly used features of vexide.
/// This module is meant to be glob imported.
pub mod prelude {
    #[cfg(feature = "async")]
    pub use vexide_async::{block_on, sleep, spawn};
    #[cfg(feature = "core")]
    pub use vexide_core::{
        competition::{Competition, CompetitionRobot, CompetitionRobotExt},
        dbg,
        io::{BufRead, Read, Seek, Write},
        print, println,
    };
    #[cfg(feature = "devices")]
    pub use vexide_devices::{
        adi::{
            analog::AdiAnalogIn,
            digital::{AdiDigitalIn, AdiDigitalOut},
            pwm::AdiPwmOut,
            AdiDevice, AdiPort,
        },
        color::Rgb,
        controller::Controller,
        peripherals::{DynamicPeripherals, Peripherals},
        position::Position,
        screen::{Circle, Line, Rect, Screen, Text, TextFormat, TextPosition, TouchState},
        smart::{
            distance::DistanceSensor,
            expander::AdiExpander,
            imu::InertialSensor,
            link::RadioLink,
            motor::{BrakeMode, Direction, Gearset, Motor, MotorControl},
            optical::OpticalSensor,
            rotation::RotationSensor,
            vision::VisionSensor,
            SmartDevice, SmartPort,
        },
    };
    #[cfg(feature = "math")]
    pub use vexide_math::{feedforward::MotorFeedforwardController, pid::PidController};
}
