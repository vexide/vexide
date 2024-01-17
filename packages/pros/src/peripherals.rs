use core::sync::atomic::AtomicBool;

use crate::adi::AdiSlot;

static PERIPHERALS_TAKEN: AtomicBool = AtomicBool::new(false);

pub struct SmartPort {
    pub port_index: u8,
}

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

    pub adi_slot_a: AdiSlot,
    pub adi_slot_b: AdiSlot,
    pub adi_slot_c: AdiSlot,
    pub adi_slot_d: AdiSlot,
    pub adi_slot_e: AdiSlot,
    pub adi_slot_f: AdiSlot,
    pub adi_slot_g: AdiSlot,
    pub adi_slot_h: AdiSlot,
}

impl Peripherals {
    const unsafe fn new() -> Self {
        Self {
            smart_port_1: SmartPort { port_index: 1 },
            smart_port_2: SmartPort { port_index: 2 },
            smart_port_3: SmartPort { port_index: 3 },
            smart_port_4: SmartPort { port_index: 4 },
            smart_port_5: SmartPort { port_index: 5 },
            smart_port_6: SmartPort { port_index: 6 },
            smart_port_7: SmartPort { port_index: 7 },
            smart_port_8: SmartPort { port_index: 8 },
            smart_port_9: SmartPort { port_index: 9 },
            smart_port_10: SmartPort { port_index: 10 },
            smart_port_11: SmartPort { port_index: 11 },
            smart_port_12: SmartPort { port_index: 12 },
            smart_port_13: SmartPort { port_index: 13 },
            smart_port_14: SmartPort { port_index: 14 },
            smart_port_15: SmartPort { port_index: 15 },
            smart_port_16: SmartPort { port_index: 16 },
            smart_port_17: SmartPort { port_index: 17 },
            smart_port_18: SmartPort { port_index: 18 },
            smart_port_19: SmartPort { port_index: 19 },
            smart_port_20: SmartPort { port_index: 20 },
            smart_port_21: SmartPort { port_index: 21 },

            adi_slot_a: AdiSlot { index: 1 },
            adi_slot_b: AdiSlot { index: 2 },
            adi_slot_c: AdiSlot { index: 3 },
            adi_slot_d: AdiSlot { index: 4 },
            adi_slot_e: AdiSlot { index: 5 },
            adi_slot_f: AdiSlot { index: 6 },
            adi_slot_g: AdiSlot { index: 7 },
            adi_slot_h: AdiSlot { index: 8 },
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
        Some(SmartPort {
            port_index: port_index as u8 + 1,
        })
    }

    /// Creates an [`AdiSlot`] only if one has not been created on the given slot before.
    ///
    /// # Panics
    ///
    /// This function panics if the provided port is outside the range 1-8.
    /// Slots outside of this range are invalid and cannot be created.
    pub fn take_adi_slot(&mut self, slot_index: u8) -> Option<AdiSlot> {
        let slot_index = slot_index as usize - 1;
        if self.adi_slots[slot_index] {
            return None;
        }
        self.smart_ports[slot_index] = true;
        Some(unsafe { AdiSlot::new(slot_index as u8 + 1)? })
    }
}
impl From<Peripherals> for DynamicPeripherals {
    fn from(peripherals: Peripherals) -> Self {
        Self::new(peripherals)
    }
}
