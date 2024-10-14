use alloc::collections::BTreeMap;
use core::task::Waker;

use vexide_core::time::Instant;

pub struct Sleepers {
    sleepers: BTreeMap<Instant, Waker>,
}

impl Sleepers {
    pub fn push(&mut self, waker: Waker, instant: Instant) {
        self.sleepers.insert(instant, waker);
    }

    pub fn pop(&mut self) -> Option<Waker> {
        self.sleepers.pop_first().map(|(_, waker)| waker)
    }
}

pub struct Reactor {
    pub(crate) sleepers: Sleepers,
}

impl Reactor {
    pub const fn new() -> Self {
        Self {
            sleepers: Sleepers {
                sleepers: BTreeMap::new(),
            },
        }
    }

    pub fn tick(&mut self) {
        if let Some(sleeper) = self.sleepers.pop() {
            sleeper.wake();
        }
    }
}
