use alloc::collections::BTreeMap;
use core::task::Waker;

pub struct Sleepers {
    sleepers: BTreeMap<u32, Waker>,
}

impl Sleepers {
    pub fn push(&mut self, waker: Waker, target: u32) {
        self.sleepers.insert(target, waker);
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
            sleeper.wake()
        }
    }
}
