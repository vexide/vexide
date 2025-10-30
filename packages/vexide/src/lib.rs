//! # vexide
//!
//! Open-source Rust runtime for VEX V5 robots. vexide provides a runtime, async executor, hardware
//! APIs, and more for the VEX V5 Brain!
//!
//! ## Getting Started
//!
//! If you're just getting started, we recommend going through our [docs](https://vexide.dev/docs/), which provide step-by-step instructions for setting up a development environment and using vexide's common features.
//!
//! # Usage
//!
//! To to get a program running, use the `#[vexide::main]` attribute on your main function.
//!
//! ```
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main(_peripherals: Peripherals) {
//!     println!("Hello, world!");
//! }
//! ```
//!
//! Check out our [examples](https://github.com/vexide/vexide/tree/main/examples/) for more examples
//! of different features.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(html_logo_url = "https://vexide.dev/images/logo.svg")]

/// Async runtime & executor.
#[cfg(feature = "async")]
pub mod runtime {
    #[doc(inline)]
    pub use vexide_async::block_on;
}

#[doc(inline)]
#[cfg(feature = "sync")]
pub use vexide_async::sync;
#[doc(inline)]
#[cfg(feature = "async")]
pub use vexide_async::task;

/// Utilities for tracking time.
///
/// This module provides types for measuring time and executing code after a set periods of time.
///
/// - [`Sleep`] is a future that does no work and completes at a specific [`Instant`] in time.
///
/// - [`sleep`] and [`sleep_until`] provide ways to yield control away from a future for or until a
///   specific instant in time.
///
/// [`sleep`]: vexide_async::time::sleep
/// [`sleep_until`]: vexide_async::time::sleep_until
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
#[cfg(feature = "backtrace")]
pub use vexide_core::backtrace;
#[doc(inline)]
#[cfg(feature = "core")]
pub use vexide_core::{competition, os, program};
#[doc(inline)]
#[cfg(feature = "devices")]
pub use vexide_devices::{adi, battery, color, controller, display, math, peripherals, smart};
#[doc(inline)]
#[cfg(all(
    feature = "macros",
    feature = "core",
    feature = "async",
    feature = "startup",
    feature = "devices"
))]
pub use vexide_macro::{main, test};
#[cfg(all(
    feature = "macros",
    not(all(
        feature = "core",
        feature = "async",
        feature = "startup",
        feature = "devices"
    ))
))]
pub use vexide_macro::{main_fail as main, test_fail as test};
#[doc(inline)]
#[cfg(feature = "startup")]
pub use vexide_startup as startup;
#[doc(inline)]
#[cfg(feature = "allocator")]
pub use vexide_startup::allocator;

/// Commonly used features of vexide.
///
/// This module is meant to be glob imported.
pub mod prelude {
    #[cfg(feature = "core")]
    pub use crate::competition::{Compete, CompeteExt};
    #[cfg(feature = "devices")]
    pub use crate::{
        adi::{
            AdiDevice,
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
        },
        controller::Controller,
        display::Display,
        math::Direction,
        peripherals::Peripherals,
        smart::{
            SmartDevice,
            ai_vision::AiVisionSensor,
            distance::DistanceSensor,
            electromagnet::Electromagnet,
            expander::AdiExpander,
            gps::GpsSensor,
            imu::InertialSensor,
            link::{LinkType, RadioLink},
            motor::{Gearset, Motor},
            optical::OpticalSensor,
            rotation::RotationSensor,
            serial::SerialPort,
            vision::VisionSensor,
        },
    };
    #[cfg(feature = "async")]
    pub use crate::{
        runtime::block_on,
        task::{spawn, task_local},
        time::{sleep, sleep_until},
    };
}
