//! Peripherals implementations.
//!
//! Peripherals are the best way to create devices because they allow you to do it safely.
//! Both kinds of peripherals, [`Peripherals`] and [`DynamicPeripherals`], guarentee that a given port is only used to create one device.
//! This is important because creating multiple devices on the same port can cause bugs and unexpected behavior.
//! Devices can still be created unsafely without using peripherals, but it isn't recommended.
//!
//! ## Examples
//!
//! ### Using [`Peripherals`]
//! ```rust
//! # use vexide::prelude::*;
//! let mut peripherals = Peripherals::take().unwrap();
//! let motor = Motor::new(peripherals.port_1);
//! let adi_digital_in = AdiDigitalIn::new(peripherals.adi_d);
//! ```
//! ### Using [`DynamicPeripherals`]
//! ```rust
//! # use vexide::prelude::*;
//! let mut peripherals = DynamicPeripherals::new(Peripherals::take().unwrap());
//! let motor = peripherals.take_smart_port(1).unwrap();
//! let adi_digital_in = peripherals.take_adi_port(4).unwrap();
//! ```

use core::sync::atomic::AtomicBool;

use crate::{
    adi::AdiPort,
    controller::{Controller, ControllerId},
    screen::Screen,
    smart::SmartPort,
};

static PERIPHERALS_TAKEN: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
/// A struct that contains all ports on the V5 Brain
/// and guarentees **at compile time** that each port is only used once.
/// Because of the fact that this checks at compile time, it cannot be moved once it has been used to create a device.
/// If you need to store a peripherals struct for use in multiple functions, use [`DynamicPeripherals`] instead.
/// This struct is always preferred over [`DynamicPeripherals`] when possible.
pub struct Peripherals {
    /// Brain screen
    pub screen: Screen,

    /// Primary ("Master") Controller
    pub primary_controller: Controller,

    /// Partner Controller
    pub partner_controller: Controller,

    /// Smart port 1 on the brain
    pub port_1: SmartPort,
    /// Smart port 2 on the brain
    pub port_2: SmartPort,
    /// Smart port 3 on the brain
    pub port_3: SmartPort,
    /// Smart port 4 on the brain
    pub port_4: SmartPort,
    /// Smart port 5 on the brain
    pub port_5: SmartPort,
    /// Smart port 6 on the brain
    pub port_6: SmartPort,
    /// Smart port 7 on the brain
    pub port_7: SmartPort,
    /// Smart port 8 on the brain
    pub port_8: SmartPort,
    /// Smart port 9 on the brain
    pub port_9: SmartPort,
    /// Smart port 10 on the brain
    pub port_10: SmartPort,
    /// Smart port 11 on the brain
    pub port_11: SmartPort,
    /// Smart port 12 on the brain
    pub port_12: SmartPort,
    /// Smart port 13 on the brain
    pub port_13: SmartPort,
    /// Smart port 14 on the brain
    pub port_14: SmartPort,
    /// Smart port 15 on the brain
    pub port_15: SmartPort,
    /// Smart port 16 on the brain
    pub port_16: SmartPort,
    /// Smart port 17 on the brain
    pub port_17: SmartPort,
    /// Smart port 18 on the brain
    pub port_18: SmartPort,
    /// Smart port 19 on the brain
    pub port_19: SmartPort,
    /// Smart port 20 on the brain
    pub port_20: SmartPort,
    /// Smart port 21 on the brain
    pub port_21: SmartPort,

    /// Adi port A on the brain.
    pub adi_a: AdiPort,
    /// Adi port B on the brain.
    pub adi_b: AdiPort,
    /// Adi port C on the brain.
    pub adi_c: AdiPort,
    /// Adi port D on the brain.
    pub adi_d: AdiPort,
    /// Adi port E on the brain.
    pub adi_e: AdiPort,
    /// Adi port F on the brain.
    pub adi_f: AdiPort,
    /// Adi port G on the brain.
    pub adi_g: AdiPort,
    /// Adi port H on the brain.
    pub adi_h: AdiPort,
}

impl Peripherals {
    // SAFETY: caller must ensure that the SmartPorts and AdiPorts created are unique
    unsafe fn new() -> Self {
        // SAFETY: caller must ensure that this function is only called once
        unsafe {
            Self {
                screen: Screen::new(),

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
    /// Creating new [`SmartPort`]s and [`Peripherals`] instances is inherently unsafe due to the possibility of constructing more than
    /// one device on the same port index and allowing multiple mutable references to the same hardware device.
    /// The caller must ensure that only one mutable reference to each port is used.
    pub unsafe fn steal() -> Self {
        PERIPHERALS_TAKEN.store(true, core::sync::atomic::Ordering::Release);
        // SAFETY: caller must ensure that this call is safe
        unsafe { Self::new() }
    }
}

/// Guarentees that ports are only used once **at runtime**
/// This is useful for when you want to store a peripherals struct for use in multiple functions.
/// When possible, use [`Peripherals`] instead.
#[derive(Debug)]
pub struct DynamicPeripherals {
    screen: bool,
    smart_ports: [bool; 21],
    adi_slots: [bool; 8],
}
impl DynamicPeripherals {
    /// Creates a new dynamic peripherals
    /// In order to guarentee that no ports created by this struct,
    /// this function takes a [`Peripherals`].
    /// This guarentees safety because [`Peripherals`] cannot be passed by value
    /// after they have been used to create devices.
    pub fn new(_peripherals: Peripherals) -> Self {
        let smart_ports = [false; 21];
        let adi_slots = [false; 8];
        Self {
            screen: false,
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

    /// Creates a [`Screen`] only if one has not been created before.
    pub fn take_screen(&mut self) -> Option<Screen> {
        if self.screen {
            return None;
        }
        self.screen = true;
        Some(unsafe { Screen::new() })
    }
}
impl From<Peripherals> for DynamicPeripherals {
    fn from(peripherals: Peripherals) -> Self {
        Self::new(peripherals)
    }
}
