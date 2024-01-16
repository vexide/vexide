use core::sync::atomic::AtomicBool;

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
}

impl Peripherals {
    pub const unsafe fn new() -> Self {
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
        }
    }

    pub fn take() -> Option<Self> {
        if PERIPHERALS_TAKEN.swap(true, core::sync::atomic::Ordering::AcqRel) {
            None
        } else {
            Some(unsafe { Self::new() })
        }
    }
}
