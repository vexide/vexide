use core::sync::atomic::AtomicBool;

use crate::devices::{adi::AdiPort, smart::SmartPort};

static PERIPHERALS_TAKEN: AtomicBool = AtomicBool::new(false);

pub struct Peripherals {
    pub smart_port_1: SmartPort,
    pub smart_port_2: SmartPort,
    pub smart_port_3: SmartPort,
    pub smart_port_4: SmartPort,
    pub smart_port_5: SmartPort,
    pub smart_port_6: SmartPort,
    pub smart_port_7: SmartPort,
    pub smart_port_8: SmartPort,
    pub smart_port_9: SmartPort,
    pub smart_port_10: SmartPort,
    pub smart_port_11: SmartPort,
    pub smart_port_12: SmartPort,
    pub smart_port_13: SmartPort,
    pub smart_port_14: SmartPort,
    pub smart_port_15: SmartPort,
    pub smart_port_16: SmartPort,
    pub smart_port_17: SmartPort,
    pub smart_port_18: SmartPort,
    pub smart_port_19: SmartPort,
    pub smart_port_20: SmartPort,
    pub smart_port_21: SmartPort,

    pub adi_port_a: AdiPort,
    pub adi_port_b: AdiPort,
    pub adi_port_c: AdiPort,
    pub adi_port_d: AdiPort,
    pub adi_port_e: AdiPort,
    pub adi_port_f: AdiPort,
    pub adi_port_g: AdiPort,
    pub adi_port_h: AdiPort,
}

impl Peripherals {
    const unsafe fn new() -> Self {
        Self {
            smart_port_1: SmartPort::new(1),
            smart_port_2: SmartPort::new(2),
            smart_port_3: SmartPort::new(3),
            smart_port_4: SmartPort::new(4),
            smart_port_5: SmartPort::new(5),
            smart_port_6: SmartPort::new(6),
            smart_port_7: SmartPort::new(7),
            smart_port_8: SmartPort::new(8),
            smart_port_9: SmartPort::new(9),
            smart_port_10: SmartPort::new(10),
            smart_port_11: SmartPort::new(11),
            smart_port_12: SmartPort::new(12),
            smart_port_13: SmartPort::new(13),
            smart_port_14: SmartPort::new(14),
            smart_port_15: SmartPort::new(15),
            smart_port_16: SmartPort::new(16),
            smart_port_17: SmartPort::new(17),
            smart_port_18: SmartPort::new(18),
            smart_port_19: SmartPort::new(19),
            smart_port_20: SmartPort::new(20),
            smart_port_21: SmartPort::new(21),

            adi_port_a: AdiPort::new(1, None),
            adi_port_b: AdiPort::new(2, None),
            adi_port_c: AdiPort::new(3, None),
            adi_port_d: AdiPort::new(4, None),
            adi_port_e: AdiPort::new(5, None),
            adi_port_f: AdiPort::new(6, None),
            adi_port_g: AdiPort::new(7, None),
            adi_port_h: AdiPort::new(8, None),
        }
    }

    pub fn take() -> Option<Self> {
        if PERIPHERALS_TAKEN.swap(true, core::sync::atomic::Ordering::AcqRel) {
            None
        } else {
            Some(unsafe { Self::new() })
        }
    }

    pub unsafe fn steal() -> Self {
        PERIPHERALS_TAKEN.store(true, core::sync::atomic::Ordering::Release);
        Self::new()
    }
}

/// Guarentees that ports are only used once **at runtime**
/// This is useful for when you want to store a peripherals struct for use in multiple functions.
/// When possible, use [`Peripherals`] instead.
pub struct DynamicPeripherals {
    smart_ports: [bool; 21],
    adi_slots: [bool; 8],
}
impl DynamicPeripherals {
    /// Creates a new dynamic peripherals
    /// In order to guarentee that no ports created by this struct,
    /// this function takes a [`Peripherals`].
    /// This guarentees safety because [`Peripherals`] cannot be passed by value
    /// after they have been used to create devices.
    pub const fn new(_peripherals: Peripherals) -> Self {
        let smart_ports = [false; 21];
        let adi_slots = [false; 8];
        Self {
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
    pub fn take_smart_port(&mut self, port_index: u8) -> Option<SmartPort> {
        let port_index = port_index as usize - 1;
        if self.smart_ports[port_index] {
            return None;
        };
        self.smart_ports[port_index] = true;
        Some(unsafe { SmartPort::new(port_index as u8 + 1) })
    }

    /// Creates an [`AdiSlot`] only if one has not been created on the given slot before.
    ///
    /// # Panics
    ///
    /// This function panics if the provided port is outside the range 1-8.
    /// Slots outside of this range are invalid and cannot be created.
    pub fn take_adi_port(&mut self, port_index: u8) -> Option<AdiPort> {
        let port_index = port_index as usize - 1;
        if self.adi_slots[port_index] {
            return None;
        }
        self.smart_ports[port_index] = true;
        Some(unsafe { AdiPort::new(port_index as u8 + 1, None) })
    }
}
impl From<Peripherals> for DynamicPeripherals {
    fn from(peripherals: Peripherals) -> Self {
        Self::new(peripherals)
    }
}
