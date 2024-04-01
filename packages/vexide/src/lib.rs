//! # vexide
//! Work in progress high level bindings for the V5 Brain VEX SDK.
//! Unlike other libraries for the V5 Brain, vexide doesn't use an RTOS.
//! Instead, vexide leverages Rust's powerful async/await (cooperative multitasking) to allow faster and more user friendly 3dfcode.
//!
//! Advantages over similar libraries or PROS itself:
//! - vexideindings for the PROS library and kernel.
//! Not everything in this library is one to one with the PROS API. has an [`Async executor`](async_runtime) which allows for easy and performant asynchronous code.
//! - Simulation support with [`pros-simulator`](https://crates.io/crates/pros-simulator) and any interface with it (e.g. [`pros-simulator-gui`](https://github.com/vexide/pros-simulator-gui))
//! - Active development. vexide is actively developed and maintained.
//! - vexide is a real crate on crates.io instead of a template, or similar. This allows for dependency management with cargo.
//!
//! # Usage
//!
//! When using pros, you have a few options for how you want to get started.
//! You have two options: `async` and `sync`.
//! When using async, an async executor is started and you can use it to run code asynchronously without any FreeRTOS tasks.
//! When using sync, if you want to run code asynchronously you must create a FreeRTOS task.
//!
//! Here are some examples of both:
//!
//! ```rust
//! // Async
//! use pros::prelude::*;
//!
//! #[derive(Default)]
//! struct Robot;
//! impl AsyncRobot for Robot {
//!    async fn opcontrol(&mut self) -> Result {
//!       loop {
//!         // Do something
//!        sleep(Duration::from_millis(20)).await;
//!       }
//!    }
//! }
//! async_robot!(Robot);
//! ```
//!
//!```rust
//! // Sync
//! use pros::prelude::*;
//!
//! #[derive(Default)]
//! struct Robot;
//! impl SyncRobot for Robot {
//!   fn opcontrol(&mut self) -> Result {
//!      loop {
//!       // Do something
//!      delay(Duration::from_millis(20));
//!      }
//!    }
//! }
//! sync_robot!(Robot);
//! ```
//!
//! You may have noticed the `#[derive(Default)]` attribute on these Robot structs.
//! If you want to learn why, look at the docs for [`vexide_async::async_robot`] or [`pros_sync::sync_robot`].
#![no_std]

#[cfg(feature = "async")]
pub use vexide_async as async_runtime;
#[cfg(feature = "core")]
pub use vexide_core as core;
#[cfg(feature = "math")]
pub use vexide_math as math;
#[cfg(feature = "panic")]
pub use vexide_panic as panic;
pub use pros_sys as sys;
#[cfg(feature = "devices")]
pub use vexide_devices as devices;

/// Commonly used features of vexide.
/// This module is meant to be glob imported.
pub mod prelude {
    #[cfg(feature = "async")]
    pub use vexide_async::{async_robot, block_on, sleep, spawn, AsyncRobot};
    #[cfg(feature = "core")]
    pub use vexide_core::{
        dbg, eprint, eprintln,
        error::{PortError, Result},
        io::{BufRead, Read, Seek, Write},
        print, println,
        task::delay,
    };
    #[cfg(feature = "math")]
    pub use vexide_math::{feedforward::MotorFeedforwardController, pid::PidController};
    #[cfg(feature = "devices")]
    pub use vexide_devices::{
        adi::{
            analog::AdiAnalogIn,
            digital::{AdiDigitalIn, AdiDigitalOut},
            encoder::AdiEncoder,
            gyro::AdiGyro,
            motor::AdiMotor,
            potentiometer::{AdiPotentiometer, AdiPotentiometerType},
            pwm::AdiPwmOut,
            solenoid::AdiSolenoid,
            ultrasonic::AdiUltrasonic,
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
            link::{Link, RxLink, TxLink},
            motor::{BrakeMode, Direction, Gearset, Motor, MotorControl},
            optical::OpticalSensor,
            rotation::RotationSensor,
            vision::VisionSensor,
            SmartDevice, SmartPort,
        },
    };
}
