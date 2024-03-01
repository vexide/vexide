//! # Pros
//! Opinionated bindings for the PROS library and kernel.
//! Not everything in this library is one to one with the PROS API.
//!
//! Advantages over similar libraries or PROS itself:
//! - Pros-rs has an [`Async executor`](async_runtime) which allows for easy and performant asynchronous code.
//! - Simulation support with [`pros-simulator`](https://crates.io/crates/pros-simulator) and any interface with it (e.g. [`pros-simulator-gui`](https://github.com/pros-rs/pros-simulator-gui))
//! - Active development. Pros-rs is actively developed and maintained.
//! - Pros-rs is a real crate on crates.io instead of a template, or similar. This allows for dependency management with cargo.
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
//! If you want to learn why, look at the docs for [`pros_async::async_robot`] or [`pros_sync::sync_robot`].
#![no_std]

#[cfg(feature = "async")]
pub use pros_async as async_runtime;
#[cfg(feature = "core")]
pub use pros_core as core;
#[cfg(feature = "devices")]
pub use pros_devices as devices;
#[cfg(feature = "math")]
pub use pros_math as math;
#[cfg(feature = "panic")]
pub use pros_panic as panic;
#[cfg(feature = "sync")]
pub use pros_sync as sync;
pub use pros_sys as sys;

/// Commonly used features of pros-rs.
/// This module is meant to be glob imported.
pub mod prelude {
    #[cfg(feature = "async")]
    pub use pros_async::{async_robot, block_on, sleep, spawn, AsyncRobot};
    #[cfg(feature = "core")]
    pub use pros_core::{
        dbg, eprint, eprintln,
        error::{PortError, Result},
        io::{BufRead, Read, Seek, Write},
        print, println,
        task::delay,
    };
    #[cfg(feature = "devices")]
    pub use pros_devices::{
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
        peripherals::{DynamicPeripherals, Peripherals},
        position::Position,
        screen::{Circle, Line, Rect, Screen, Text, TextFormat, TextPosition, TouchState},
        smart::{
            distance::DistanceSensor,
            expander::AdiExpander,
            gps::GpsSensor,
            imu::InertialSensor,
            link::{Link, RxLink, TxLink},
            motor::{BrakeMode, Gearset, Motor},
            optical::OpticalSensor,
            rotation::RotationSensor,
            vision::VisionSensor,
            SmartDevice, SmartPort,
        },
    };
    #[cfg(feature = "math")]
    pub use pros_math::{feedforward::MotorFeedforwardController, pid::PidController};
    #[cfg(feature = "sync")]
    pub use pros_sync::{sync_robot, SyncRobot};
}
