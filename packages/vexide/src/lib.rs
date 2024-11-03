//! # vexide
//!
//! Open-source Rust runtime for VEX V5 robots. vexide provides a `no_std` Rust runtime,
//! async executor, device API, and more for the VEX V5 Brain!
//!
//! vexide is the successor to [pros-rs](https://github.com/vexide/pros-rs) which are a set of unmaintained API using bindings over [PROS](https://github.com/purduesigbots/pros).
//!
//! ## Getting Started
//!
//! If you're just getting started, we recommend going through our [docs](https://vexide.dev/docs/), which provide step-by-step instructions for setting up a development environment
//! with [vexide-template](https://github.com/vexide/vexide-template).
//!
//! ## Project Structure
//!
//! The vexide runtime is split into 7 sub-crates. The one you're looking at right now re-exports each of these crates into a single package.
//!
//! - [`vexide-core`](https://docs.rs/vexide_core) provides low-level core functionality for programs, such as allocators, synchronization primitives, serial printing, I/O and timers.
//! - [`vexide-devices`](https://docs.rs/vexide_devices) contains all device-related bindings for things like motors and sensors.
//! - [`vexide-async`](https://docs.rs/vexide_async) implements our cooperative async runtime as well as several important async futures.
//! - [`vexide-startup`](https://docs.rs/vexide_startup) contains bare-metal startup code required to get freestanding user programs running on the Brain.
//! - [`vexide-panic`](https://docs.rs/vexide_panic) contains our [panic handler](https://doc.rust-lang.org/nomicon/panic-handler.html).
//! - [`vexide-graphics`](https://docs.rs/vexide_graphics) implements graphics drivers for some popular embedded Rust graphics libraries like [`slint`] and [`embedded-graphics`].
//! - [`vexide-macro`](https://docs.rs/vexide_macro) contains the source code for the `#[vexide::main]` proc-macro.
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
//! Check out our [docs](https://vexide.dev/docs/) for more in-depth usage guides.

#![no_std]
#![doc(html_logo_url = "https://vexide.dev/images/logo.svg")]

#[doc(inline)]
#[cfg(feature = "async")]
pub use vexide_async as async_runtime;
#[doc(inline)]
#[cfg(feature = "core")]
pub use vexide_core as core;
#[doc(inline)]
#[cfg(feature = "devices")]
pub use vexide_devices as devices;
#[doc(inline)]
#[cfg(feature = "graphics")]
pub use vexide_graphics as graphics;
#[doc(inline)]
#[cfg(feature = "macro")]
pub use vexide_macro as r#macro;
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
/// This module is meant to be glob imported.
pub mod prelude {
    #[cfg(feature = "async")]
    pub use vexide_async::{
        block_on,
        task::{spawn, Task},
        time::{sleep, sleep_until},
    };
    #[cfg(feature = "core")]
    pub use vexide_core::{
        competition::{Compete, CompeteExt, CompetitionRuntime},
        dbg,
        float::Float,
        io::{BufRead, Read, Seek, Write},
        print, println,
    };
    #[cfg(feature = "devices")]
    pub use vexide_devices::{
        adi::{
            accelerometer::{AdiAccelerometer, Sensitivity},
            addrled::AdiAddrLed,
            analog::AdiAnalogIn,
            digital::{AdiDigitalIn, AdiDigitalOut},
            encoder::AdiEncoder,
            light_sensor::AdiLightSensor,
            line_tracker::AdiLineTracker,
            motor::AdiMotor,
            potentiometer::{AdiPotentiometer, PotentiometerType},
            pwm::AdiPwmOut,
            range_finder::AdiRangeFinder,
            solenoid::AdiSolenoid,
            AdiDevice, AdiPort,
        },
        battery,
        controller::Controller,
        display::Display,
        peripherals::{DynamicPeripherals, Peripherals},
        position::Position,
        rgb::Rgb,
        smart::{
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
    #[cfg(all(feature = "graphics", feature = "embedded-graphics"))]
    pub use vexide_graphics::embedded_graphics::BrainDisplay;
    #[cfg(all(feature = "graphics", feature = "slint"))]
    pub use vexide_graphics::slint::initialize_slint_platform;
}
