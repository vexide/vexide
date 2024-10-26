//! Peripheral Access
//!
//! This module is the gateway to all of your Brain’s available I/O — ports, hardware, and devices.
//! If you want to create a device like a sensor or motor or read from a controller, you are going to
//! need something off a struct in this module.
//!
//! This module provides safe access to underlying hardware by treating physical ports as unique
//! resources. The [`Peripherals`] struct stores ownership tokens for each hardware interface:
//!
//! - 21 Smart Ports for V5 devices
//! - 8 ADI ports for legacy devices
//! - [`Display`] instance.
//! - Instances for the primary and partner [`Controller`]s.
//!
//! These tokens can only be claimed once, preventing multiple parts of code from
//! accidentally controlling the same hardware.
//!
//! # Overview
//!
//! The peripherals system uses a singleton pattern to ensure safe hardware access:
//! - Only one instance of [`Peripherals`] can exist at a time.
//! - Each port may only be claimed (owned by a device) once.
//! - Once claimed, a port cannot be used again until explicitly released.
//! - Ports cannot be cloned or copied. New ports cannot be safely created.
//!
//! By extension of this, only one mutable reference to a piece of hardware may exist. This
//! pattern of treating peripherals as a singleton is fairly common in the Rust embedded scene,
//! and is extensively coevered in [The Embedded Rust Book](https://docs.rust-embedded.org/book/peripherals/singletons.html).
//!
//! # Usage
//!
//! The [`Peripherals`] struct provides compile-time guarantees for exclusive port ownership.
//! This is best for when you know your port assignments at compile time.
//!
//! In vexide programs, a pre-initialized instance of this [`Peripherals`] struct is passed to your
//! program's entrypoint function:
//!
//! ```
//! #![no_std]
//! #![no_main]
//!
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main(peripherals: Peripherals) {
//!     println!("o.o what's this? {:?}", peripherals);
//! }
//! ```
//!
//! You can then move ports or other peripherals out of this struct to create your devices:
//!
//! ```
//! #![no_std]
//! #![no_main]
//!
//! use vexide::prelude::*;
//!
//! #[vexide::main]
//! async fn main(peripherals: Peripherals) {
//!     let mut screen = peripherals.screen;
//!     let my_motor = Motor::new(
//!         peripherals.port_1,
//!         Gearset::Green,
//!         Direction::Forward,
//!     );
//! }
//! ```
//!
//! # Dynamic Peripherals
//!
//! The [`DynamicPeripherals`] struct provides a more "flexible" way to claim ports at runtime while
//! maintaining safety guarantees. Instead of statically assigning ports at compile time, you can
//! request ports by number (e.g. port 1-21) during program execution. This is useful when:
//!
//! - You want to store unclaimed peripherals after claiming something, or pass your peripherals by value
//!   after taking something from it ([`Peripherals`] prevents this due to partial-move rules).
//! - Port assignments need to be configurable without recompiling.
//! - Port numbers need to be determined programmatically.
//!
//! The system still ensures only one device can use a port at a time, but handles the
//! bookkeeping at runtime rather than compile time. This trades a small performance cost
//! for increased flexibility, but is generally preferable to use the static [`Peripherals`]
//! struct at runtime.

use core::sync::atomic::AtomicBool;

use crate::{
    adi::AdiPort,
    controller::{Controller, ControllerId},
    display::Display,
    smart::SmartPort,
};

static PERIPHERALS_TAKEN: AtomicBool = AtomicBool::new(false);

/// Singleton Peripheral Access
///
/// Contains an instance of a Brain’s available I/O, including ports, hardware, and devices.
///
/// A Brain often has many external devices attached to it. We call these devices *peripherals*, and this
/// struct is the "gateway" to all of these. [`Peripherals`] is intended to be used as a singleton, and you
/// will typically only get one of these in your program's execution. This guarantees **at compile time** that
/// each port is only used once.
///
/// Because of the fact that this checks at compile time, it cannot be copied, cloned, or moved once
/// it has been used to create a device.
///
/// If you need to store a peripherals struct for use in multiple functions, use [`DynamicPeripherals`] instead.
/// This struct is always preferrable to [`DynamicPeripherals`] when possible.
#[derive(Debug)]
pub struct Peripherals {
    /// Brain display
    pub display: Display,

    /// Primary ("Master") Controller
    pub primary_controller: Controller,

    /// Partner Controller
    pub partner_controller: Controller,

    /// Smart Port 1 on the Brain
    pub port_1: SmartPort,
    /// Smart Port 2 on the Brain
    pub port_2: SmartPort,
    /// Smart Port 3 on the Brain
    pub port_3: SmartPort,
    /// Smart Port 4 on the Brain
    pub port_4: SmartPort,
    /// Smart Port 5 on the Brain
    pub port_5: SmartPort,
    /// Smart Port 6 on the Brain
    pub port_6: SmartPort,
    /// Smart Port 7 on the Brain
    pub port_7: SmartPort,
    /// Smart Port 8 on the Brain
    pub port_8: SmartPort,
    /// Smart Port 9 on the Brain
    pub port_9: SmartPort,
    /// Smart Port 10 on the Brain
    pub port_10: SmartPort,
    /// Smart Port 11 on the Brain
    pub port_11: SmartPort,
    /// Smart Port 12 on the Brain
    pub port_12: SmartPort,
    /// Smart Port 13 on the Brain
    pub port_13: SmartPort,
    /// Smart Port 14 on the Brain
    pub port_14: SmartPort,
    /// Smart Port 15 on the Brain
    pub port_15: SmartPort,
    /// Smart Port 16 on the Brain
    pub port_16: SmartPort,
    /// Smart Port 17 on the Brain
    pub port_17: SmartPort,
    /// Smart Port 18 on the Brain
    pub port_18: SmartPort,
    /// Smart Port 19 on the Brain
    pub port_19: SmartPort,
    /// Smart Port 20 on the Brain
    pub port_20: SmartPort,
    /// Smart Port 21 on the Brain
    pub port_21: SmartPort,

    /// Adi port A on the Brain.
    pub adi_a: AdiPort,
    /// Adi port B on the Brain.
    pub adi_b: AdiPort,
    /// Adi port C on the Brain.
    pub adi_c: AdiPort,
    /// Adi port D on the Brain.
    pub adi_d: AdiPort,
    /// Adi port E on the Brain.
    pub adi_e: AdiPort,
    /// Adi port F on the Brain.
    pub adi_f: AdiPort,
    /// Adi port G on the Brain.
    pub adi_g: AdiPort,
    /// Adi port H on the Brain.
    pub adi_h: AdiPort,
}

impl Peripherals {
    // SAFETY: caller must ensure that the SmartPorts and AdiPorts created are unique
    unsafe fn new() -> Self {
        // SAFETY: caller must ensure that this function is only called once
        unsafe {
            Self {
                display: Display::new(),

                primary_controller: Controller::new(ControllerId::Primary),
                partner_controller: Controller::new(ControllerId::Partner),

                port_1: SmartPort::new(1),
                port_2: SmartPort::new(2),
                port_3: SmartPort::new(3),
                port_4: SmartPort::new(4),
                port_5: SmartPort::new(5),
                port_6: SmartPort::new(6),
                port_7: SmartPort::new(7),
                port_8: SmartPort::new(8),
                port_9: SmartPort::new(9),
                port_10: SmartPort::new(10),
                port_11: SmartPort::new(11),
                port_12: SmartPort::new(12),
                port_13: SmartPort::new(13),
                port_14: SmartPort::new(14),
                port_15: SmartPort::new(15),
                port_16: SmartPort::new(16),
                port_17: SmartPort::new(17),
                port_18: SmartPort::new(18),
                port_19: SmartPort::new(19),
                port_20: SmartPort::new(20),
                port_21: SmartPort::new(21),

                adi_a: AdiPort::new(1, None),
                adi_b: AdiPort::new(2, None),
                adi_c: AdiPort::new(3, None),
                adi_d: AdiPort::new(4, None),
                adi_e: AdiPort::new(5, None),
                adi_f: AdiPort::new(6, None),
                adi_g: AdiPort::new(7, None),
                adi_h: AdiPort::new(8, None),
            }
        }
    }

    /// Attempts to create a new [`Peripherals`] struct, returning `None` if one has already been created.
    ///
    /// After calling this function, future calls to [`Peripherals::take`] will return `None`.
    pub fn take() -> Option<Self> {
        if PERIPHERALS_TAKEN.swap(true, core::sync::atomic::Ordering::AcqRel) {
            None
        } else {
            Some(unsafe { Self::new() })
        }
    }

    /// Creates a new [`Peripherals`] struct without ensuring that is the only unique instance.
    ///
    /// After calling this function, future calls to [`Peripherals::take`] will return `None`.
    ///
    /// # Safety
    ///
    /// Creating new [`SmartPort`]s and [`Peripherals`] instances is unsafe due to the possibility of constructing
    /// more than one device on the same port index.
    ///
    /// The caller must ensure that a given peripheral is not mutated concurrently as a result of more than one
    /// instance existing.
    pub unsafe fn steal() -> Self {
        PERIPHERALS_TAKEN.store(true, core::sync::atomic::Ordering::Release);
        // SAFETY: caller must ensure that this call is safe
        unsafe { Self::new() }
    }
}

/// Runtime-enforced Singleton Peripheral Access
///
/// A flexible alternative to the statically checked [`Peripherals`], that instead verifies singleton
/// access to ports and peripherals at *runtime*, allowing you to move this struct around after taking
/// a port or device.
///
/// This is useful in cases where:
///
/// - You want to store unclaimed peripherals after claiming something, or pass this struct by value after
///   taking something from it ([`Peripherals`] prevents this due to partial-move rules).
/// - Port assignments need to be configurable without recompiling.
/// - Port numbers need to be determined programmatically.
///
/// The system still ensures only one device can use a port at a time, but handles the
/// bookkeeping at runtime rather than compile time. This trades a small performance cost
/// for increased flexibility, but is generally preferable to use the static [`Peripherals`]
/// struct at runtime.
#[derive(Debug)]
pub struct DynamicPeripherals {
    display: bool,
    smart_ports: [bool; 21],
    adi_slots: [bool; 8],
}
impl DynamicPeripherals {
    /// Creates a new dynamic peripherals
    ///
    /// In order to guarantee that no new ports are created by this struct,
    /// this function requires a pre-existing [`Peripherals`] instance.
    ///
    /// This guarantees safety because [`Peripherals`] cannot be passed by value
    /// after it has been used to create devices.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(_peripherals: Peripherals) -> Self {
        let smart_ports = [false; 21];
        let adi_slots = [false; 8];
        Self {
            display: false,
            smart_ports,
            adi_slots,
        }
    }

    /// Creates a [`SmartPort`] only if one has not been created on the given port before.
    ///
    /// # Panics
    ///
    /// This function panics if the provided port is outside the range 1-21.
    /// Ports outside of this range are invalid and cannot be created.
    pub fn take_smart_port(&mut self, port_number: u8) -> Option<SmartPort> {
        let port_index = port_number as usize - 1;
        if self.smart_ports[port_index] {
            return None;
        };
        self.smart_ports[port_index] = true;
        Some(unsafe { SmartPort::new(port_number) })
    }

    /// Creates an [`AdiPort`] only if one has not been created on the given slot before.
    ///
    /// # Panics
    ///
    /// This function panics if the provided port is outside the range 1-8.
    /// Slots outside of this range are invalid and cannot be created.
    pub fn take_adi_port(&mut self, port_number: u8) -> Option<AdiPort> {
        let port_number = port_number as usize - 1;
        if self.adi_slots[port_number] {
            return None;
        }
        self.smart_ports[port_number] = true;
        Some(unsafe { AdiPort::new(port_number as u8 + 1, None) })
    }

    /// Creates a [`Display`] only if one has not been created before.
    pub fn take_display(&mut self) -> Option<Display> {
        if self.display {
            return None;
        }
        self.display = true;
        Some(unsafe { Display::new() })
    }
}
impl From<Peripherals> for DynamicPeripherals {
    fn from(peripherals: Peripherals) -> Self {
        Self::new(peripherals)
    }
}
