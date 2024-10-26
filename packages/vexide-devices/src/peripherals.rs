//! Peripherals implementations.
//!
//! Peripherals are the best way to create devices because they allow you to do it safely.
//! Both kinds of peripherals, [`Peripherals`] and [`DynamicPeripherals`], guarantee that a given port is only used to create one device.
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
    display::Display,
    smart::SmartPort,
};

static PERIPHERALS_TAKEN: AtomicBool = AtomicBool::new(false);

/// Contains an instance of a brain’s available I/O, including ports, hardware, and devices.
///
/// A brain often has many external devices attached to it. We call these devices *peripherals*, and this
/// struct is the "gateway" to all of these. [`Peripherals`] is intended to be used as a singleton, and you
/// will typically only get one of these in your program's execution. This guarantees **at compile time** that
/// each port is only used once.
///
/// Because of the fact that this checks at compile time, it cannot be copied, cloned, or moved once
/// it has been used to create a device.
///
/// If you need to store a peripherals struct for use in multiple functions, use [`DynamicPeripherals`] instead.
/// This struct is always preferred over [`DynamicPeripherals`] when possible.
#[derive(Debug)]
pub struct Peripherals {
    /// Brain display
    pub display: Display,

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
    /// Creating new [`SmartPort`]s and [`Peripherals`] instances is inherently unsafe due to the possibility of constructing more than
    /// one device on the same port index and allowing multiple mutable references to the same hardware device.
    /// The caller must ensure that only one mutable reference to each port is used.
    pub unsafe fn steal() -> Self {
        PERIPHERALS_TAKEN.store(true, core::sync::atomic::Ordering::Release);
        // SAFETY: caller must ensure that this call is safe
        unsafe { Self::new() }
    }
}

/// Guarantees that ports are only used once **at runtime**
/// This is useful for when you want to store a peripherals struct for use in multiple functions.
/// When possible, use [`Peripherals`] instead.
#[derive(Debug)]
pub struct DynamicPeripherals {
    display: Option<Display>,
    primary_controller: Option<Controller>,
    partner_controller: Option<Controller>,
    smart_ports: [Option<SmartPort>; 21],
    adi_slots: [Option<AdiPort>; 8],
}
impl DynamicPeripherals {
    /// Creates a new dynamic peripherals
    /// In order to guarantee that no ports created by this struct,
    /// this function takes a [`Peripherals`].
    /// This guarantees safety because [`Peripherals`] cannot be passed by value
    /// after they have been used to create devices.
    #[must_use]
    pub fn new(peripherals: Peripherals) -> Self {
        let smart_ports = [
            peripherals.port_1.into(),
            peripherals.port_2.into(),
            peripherals.port_3.into(),
            peripherals.port_4.into(),
            peripherals.port_5.into(),
            peripherals.port_6.into(),
            peripherals.port_7.into(),
            peripherals.port_8.into(),
            peripherals.port_9.into(),
            peripherals.port_10.into(),
            peripherals.port_11.into(),
            peripherals.port_12.into(),
            peripherals.port_13.into(),
            peripherals.port_14.into(),
            peripherals.port_15.into(),
            peripherals.port_16.into(),
            peripherals.port_17.into(),
            peripherals.port_18.into(),
            peripherals.port_19.into(),
            peripherals.port_20.into(),
            peripherals.port_21.into(),
        ];
        let adi_slots = [
            peripherals.adi_a.into(),
            peripherals.adi_b.into(),
            peripherals.adi_c.into(),
            peripherals.adi_d.into(),
            peripherals.adi_e.into(),
            peripherals.adi_f.into(),
            peripherals.adi_g.into(),
            peripherals.adi_h.into(),
        ];
        Self {
            display: Some(peripherals.display),
            primary_controller: Some(peripherals.primary_controller),
            partner_controller: Some(peripherals.partner_controller),
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
        self.smart_ports[port_index].take()
    }
    /// Returns a [`SmartPort`] to the dynamic peripherals.
    ///
    /// # Panics
    ///
    /// This function panics if the provided port is outside the range 1-21.
    pub fn return_smart_port(&mut self, port: SmartPort) {
        let port_index = port.number() as usize - 1;
        self.smart_ports[port_index] = Some(port);
    }

    /// Creates an [`AdiPort`] only if one has not been created on the given slot before.
    ///
    /// # Panics
    ///
    /// This function panics if the provided port is outside the range 1-8.
    /// Slots outside of this range are invalid and cannot be created.
    pub fn take_adi_port(&mut self, port_number: u8) -> Option<AdiPort> {
        let port_number = port_number as usize - 1;
        self.adi_slots[port_number].take()
    }
    /// Returns an [`AdiPort`] to the dynamic peripherals.
    ///
    /// # Panics
    ///
    /// This function panics if the provided port is outside the range 1-8.
    pub fn return_adi_port(&mut self, port: AdiPort) {
        let port_number = port.number() as usize - 1;
        self.adi_slots[port_number] = Some(port);
    }

    /// Creates a [`Display`] only if one has not been created before.
    pub fn take_display(&mut self) -> Option<Display> {
        self.display.take()
    }
    /// Returns a [`Display`] to the dynamic peripherals.
    pub fn return_display(&mut self, display: Display) {
        self.display = Some(display);
    }

    /// Creates a primary controller only if one has not been created before.
    pub fn take_primary_controller(&mut self) -> Option<Controller> {
        self.primary_controller.take()
    }
    /// Returns the primary controller to the dynamic peripherals.
    pub fn return_primary_controller(&mut self, controller: Controller) {
        self.primary_controller = Some(controller);
    }

    /// Creates a partner controller only if one has not been created before.
    pub fn take_partner_controller(&mut self) -> Option<Controller> {
        self.partner_controller.take()
    }
    /// Returns the partner controller to the dynamic peripherals.
    pub fn return_partner_controller(&mut self, controller: Controller) {
        self.partner_controller = Some(controller);
    }
}
impl From<Peripherals> for DynamicPeripherals {
    fn from(peripherals: Peripherals) -> Self {
        Self::new(peripherals)
    }
}
